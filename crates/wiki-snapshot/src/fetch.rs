//! The `fetch` command: downloads every page, Cargo table, redirect and template default
//! from the wiki into `dataset/raw/` (`crate::store` writes them, `crate::http` talks to
//! the network). `crate::main`'s `build` command reads what this writes; it never runs
//! from here.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use wiki::{page_file_name, ArticleCategory, IndexEntry, PageKind, Row};

use crate::api::{
    cargo_url, is_translation_subpage, pages_url, parse_cargo, parse_pages, sort_rows, FetchedPage,
    Pending, ROWS_PER_REQUEST, TABLES,
};
use crate::namespace::{
    allpages_url, embeddedin_titles_url, is_own_wiki_page, parse_embeddedin_titles,
    parse_redirects, redirects_url, template_wikitext_url,
};
use crate::store::{prune, write_if_changed};
use crate::{io_error, Failure, Outcome};

/// A page's text as it goes to disk: LF line endings, because the repo forces LF in the
/// working copy and a downloaded CRLF would get rewritten on every `fetch`.
fn page_bytes(text: &str) -> Vec<u8> {
    text.replace("\r\n", "\n").into_bytes()
}

/// What fetching one kind leaves behind: how many files it wrote, and the names it wrote or
/// confirmed — the ones the directory's `prune` keeps, and whose count is the kind's page
/// count. It also remembers each file name's lowercase spelling, for [`admit`].
#[derive(Debug, Default)]
struct KindFetch {
    written: usize,
    keep: BTreeSet<String>,
    /// `page_file_name` is injective, but Windows's filesystem is case-insensitive: two
    /// titles that collide would keep the first one and warn about it. Lowercase file name →
    /// the title that took it.
    titles_by_lower_name: BTreeMap<String, String>,
}

/// Whether a page that arrived with text is filed under the kind being fetched.
#[derive(Debug, PartialEq, Eq)]
enum Admission {
    /// Filed, under this file name.
    Admit(String),
    /// A translation (`Steven/de`): it transcludes the same infobox, and it is not a page of ours.
    Translation,
    /// One of the wiki's own pages (`is_own_wiki_page`): a portal, not a page about the game.
    OwnWikiPage,
    /// Already filed under another kind, which keeps it — the precedence decision 1 asks
    /// for: whichever kind claims a title first is the one it stays under, so a page
    /// transcluding both a collectible infobox and an entity one is the collectible it was
    /// filed as first, never re-filed as the entity `PageKind::ALL` reaches afterward.
    OtherKind(PageKind),
    /// Another title already took the same file name on a case-insensitive filesystem.
    SameFileAs(String),
}

/// Where a page goes, from what is already filed. Pure: the warnings and the writes are the
/// caller's. A page that reappears with text within the same kind is admitted again and
/// silently replaces its entry; the same page under a different kind stays with the first —
/// which kind runs first over `PageKind::ALL` (or, for `Article`, running last of all, after
/// `fetch`'s whole per-template pass) is what decides precedence.
fn admit(
    title: &str,
    kind: PageKind,
    index: &BTreeMap<String, IndexEntry>,
    fetched: &KindFetch,
) -> Admission {
    if is_translation_subpage(title) {
        return Admission::Translation;
    }
    if is_own_wiki_page(title) {
        return Admission::OwnWikiPage;
    }
    if let Some(prev) = index.get(title).filter(|prev| prev.kind != kind) {
        return Admission::OtherKind(prev.kind);
    }
    let name = format!("{}.wikitext", page_file_name(title));
    match fetched
        .titles_by_lower_name
        .get(&name.to_lowercase())
        .filter(|first| *first != title)
    {
        Some(first) => Admission::SameFileAs(first.clone()),
        None => Admission::Admit(name),
    }
}

/// Downloads all pages of one kind, writes them and updates the index.
///
/// One generator run per template, and `Pending` belongs to a run: a page the server lists
/// without text arrives in a later batch *of that same run*, so each template has to answer
/// for its own before the next one starts (B45: the characters have two).
fn fetch_kind(
    kind: PageKind,
    dir: &Path,
    index: &mut BTreeMap<String, IndexEntry>,
) -> Result<KindFetch, Failure> {
    let mut fetched = KindFetch::default();
    for template in kind.templates() {
        fetch_template(template, kind, dir, index, &mut fetched)?;
    }
    Ok(fetched)
}

/// One template's generator run, batch after batch.
///
/// Pages listed without text: the server relists in every batch the ones already delivered
/// too, and in a truncated batch it lists ahead of time the ones that will arrive later. Only
/// at the end of the run do we know if any are still missing.
fn fetch_template(
    template: &str,
    kind: PageKind,
    dir: &Path,
    index: &mut BTreeMap<String, IndexEntry>,
    fetched: &mut KindFetch,
) -> Outcome {
    let mut pending = Pending::default();
    let mut cont = BTreeMap::new();
    loop {
        let body = crate::http::get(&pages_url(template, &cont)).map_err(Failure::Error)?;
        let batch = parse_pages(&body).map_err(Failure::Error)?;
        for (pageid, title) in &batch.without_revision {
            pending.seen_without(*pageid, title);
        }
        for page in batch.pages {
            pending.delivered(page.pageid);
            // No kind fetched by template carries a `category`: it exists only for
            // `Article`, filed by `fetch_allpages` below.
            file_page(page, kind, dir, index, fetched, None)?;
        }
        match batch.cont {
            Some(c) => cont = c,
            None => break,
        }
    }
    let unresolved = pending.unresolved();
    if !unresolved.is_empty() {
        return Err(Failure::Error(format!(
            "{}: {} pages listed by {template} but never arrived with text: {}",
            kind.dir(),
            unresolved.len(),
            unresolved.join(", ")
        )));
    }
    Ok(())
}

/// The four infobox templates decision 2 declines to read: an article that transcludes one
/// is tagged with the matching category, so the landing can tell a card from a mechanic
/// page without reading the infobox's own parameters.
const CATEGORY_TEMPLATES: &[(&str, ArticleCategory)] = &[
    ("Template:Infobox card", ArticleCategory::Card),
    ("Template:Infobox rune", ArticleCategory::Rune),
    ("Template:Infobox pickup", ArticleCategory::Pickup),
    ("Template:Infobox stage", ArticleCategory::Stage),
];

/// Every title transcluding one of `CATEGORY_TEMPLATES`, title → category. Titles only
/// (`embeddedin_titles_url`): the category is all `index.json` keeps of these four
/// infoboxes, so their wikitext is never fetched. A title that transcludes two of the four
/// (unseen on the wiki so far) keeps whichever `CATEGORY_TEMPLATES` visits last — an
/// explicit last-wins, not a crash, because none of the four is more "correct" than another.
fn fetch_categories() -> Result<BTreeMap<String, ArticleCategory>, Failure> {
    let mut categories = BTreeMap::new();
    for (template, category) in CATEGORY_TEMPLATES {
        let mut cont = BTreeMap::new();
        loop {
            let body = crate::http::get(&embeddedin_titles_url(template, &cont))
                .map_err(Failure::Error)?;
            let batch = parse_embeddedin_titles(&body).map_err(Failure::Error)?;
            for title in batch.titles {
                categories.insert(title, *category);
            }
            match batch.cont {
                Some(c) => cont = c,
                None => break,
            }
        }
    }
    Ok(categories)
}

/// Every page of namespace 0 not already claimed by a template-fetched kind: the whole-
/// namespace fetch (decision 1), run once `fetch` has finished every `PageKind::ALL` kind
/// but `Article` — so `admit`'s `OtherKind` precedence has something to check against.
/// `categories` tags the ones that transclude a declined infobox.
fn fetch_allpages(
    dir: &Path,
    index: &mut BTreeMap<String, IndexEntry>,
    categories: &BTreeMap<String, ArticleCategory>,
) -> Result<KindFetch, Failure> {
    let mut fetched = KindFetch::default();
    let mut pending = Pending::default();
    let mut cont = BTreeMap::new();
    loop {
        let body = crate::http::get(&allpages_url(&cont)).map_err(Failure::Error)?;
        let batch = parse_pages(&body).map_err(Failure::Error)?;
        for (pageid, title) in &batch.without_revision {
            pending.seen_without(*pageid, title);
        }
        for page in batch.pages {
            pending.delivered(page.pageid);
            let category = categories.get(&page.title).copied();
            file_page(page, PageKind::Article, dir, index, &mut fetched, category)?;
        }
        match batch.cont {
            Some(c) => cont = c,
            None => break,
        }
    }
    let unresolved = pending.unresolved();
    if !unresolved.is_empty() {
        return Err(Failure::Error(format!(
            "article: {} pages listed by allpages but never arrived with text: {}",
            unresolved.len(),
            unresolved.join(", ")
        )));
    }
    Ok(fetched)
}

/// Every redirect of namespace 0, `from` → `to`, both canonical titles, later than the
/// last: `parse_redirects` already canonicalizes each pair, so a `from` seen twice (should
/// never happen — the wiki lists a redirect page once) keeps its last answer, same as any
/// other map insert.
fn fetch_redirects() -> Result<BTreeMap<String, String>, Failure> {
    let mut redirects = BTreeMap::new();
    let mut cont = BTreeMap::new();
    loop {
        let body = crate::http::get(&redirects_url(&cont)).map_err(Failure::Error)?;
        let batch = parse_redirects(&body).map_err(Failure::Error)?;
        redirects.extend(batch.redirects);
        match batch.cont {
            Some(c) => cont = c,
            None => break,
        }
    }
    Ok(redirects)
}

/// A namespace-10 template's own wikitext, fetched once (decision 4: the base-stat
/// defaults `Template:Infobox character` declares). A single title never truncates across
/// batches — there is only one page to list — so a response with none is the error.
fn fetch_template_wikitext(title: &str) -> Result<String, Failure> {
    let body = crate::http::get(&template_wikitext_url(title)).map_err(Failure::Error)?;
    let batch = parse_pages(&body).map_err(Failure::Error)?;
    batch
        .pages
        .into_iter()
        .next()
        .map(|p| p.text)
        .ok_or_else(|| Failure::Error(format!("{title}: no page in the response")))
}

/// Writes one page and files it in the index, or says why it is not filed. `category` is
/// `Some` only when filing an `Article` that transcludes one of `CATEGORY_TEMPLATES`.
fn file_page(
    page: FetchedPage,
    kind: PageKind,
    dir: &Path,
    index: &mut BTreeMap<String, IndexEntry>,
    fetched: &mut KindFetch,
    category: Option<ArticleCategory>,
) -> Outcome {
    let name = match admit(&page.title, kind, index, fetched) {
        Admission::Admit(name) => name,
        Admission::Translation | Admission::OwnWikiPage => return Ok(()),
        Admission::OtherKind(prev) => {
            eprintln!(
                "  warning: «{}» is already of kind {}; ignored as {}",
                page.title,
                prev.dir(),
                kind.dir()
            );
            return Ok(());
        }
        Admission::SameFileAs(first) => {
            eprintln!(
                "  warning: «{}» and «{}» have the same file name on a case-insensitive \
                 filesystem; keeping the first one",
                first, page.title
            );
            return Ok(());
        }
    };
    fetched
        .titles_by_lower_name
        .insert(name.to_lowercase(), page.title.clone());
    let path = dir.join(&name);
    if write_if_changed(&path, &page_bytes(&page.text)).map_err(|e| io_error(&path, e))? {
        fetched.written += 1;
    }
    fetched.keep.insert(name);
    index.insert(
        page.title,
        IndexEntry {
            kind,
            pageid: page.pageid,
            revid: page.revid,
            timestamp: page.timestamp,
            category,
        },
    );
    Ok(())
}

/// Downloads a whole Cargo table, in pages of `ROWS_PER_REQUEST` rows.
fn fetch_table(table: &str, fields: &str) -> Result<Vec<Row>, Failure> {
    let mut rows = Vec::new();
    let mut offset = 0;
    loop {
        let body = crate::http::get(&cargo_url(table, fields, offset)).map_err(Failure::Error)?;
        let batch = parse_cargo(&body).map_err(Failure::Error)?;
        let n = batch.len();
        rows.extend(batch);
        if n < ROWS_PER_REQUEST {
            break;
        }
        offset += ROWS_PER_REQUEST;
    }
    sort_rows(&mut rows);
    Ok(rows)
}

fn pretty_json<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, Failure> {
    let mut s = serde_json::to_string_pretty(value)
        .map_err(|e| Failure::Error(format!("serialization: {e}")))?;
    s.push('\n');
    Ok(s.into_bytes())
}

/// Prunes `dir` to `fetched.keep` and prints the three counts every kind's fetch ends with
/// (pages kept, written, deleted): the one report every kind gives, template-fetched or not.
fn report_fetch(label: &str, dir: &Path, fetched: &KindFetch) -> Outcome {
    let removed = prune(dir, &fetched.keep).map_err(|e| io_error(dir, e))?;
    for name in &removed {
        println!("  deleted {}", name);
    }
    println!(
        "{label}: {} pages, {} written, {} deleted",
        fetched.keep.len(),
        fetched.written,
        removed.len()
    );
    Ok(())
}

/// The `fetch` command: every kind's pages, the Cargo tables, the redirect map and the
/// template default, all into `out`.
pub fn fetch(out: &Path) -> Outcome {
    println!("snapshot at {}", out.display());
    let mut index = BTreeMap::new();
    for kind in PageKind::ALL {
        // `Article` has no template of its own (`PageKind::templates`'s doc comment): it is
        // claimed below by `fetch_allpages`, once every template-fetched kind has had first
        // claim on its own pages — that order is `admit`'s `OtherKind` precedence.
        if kind == PageKind::Article {
            continue;
        }
        let dir = out.join("pages").join(kind.dir());
        let fetched = fetch_kind(kind, &dir, &mut index)?;
        report_fetch(kind.dir(), &dir, &fetched)?;
    }

    let categories = fetch_categories()?;
    let article_dir = out.join("pages").join(PageKind::Article.dir());
    let fetched = fetch_allpages(&article_dir, &mut index, &categories)?;
    report_fetch(PageKind::Article.dir(), &article_dir, &fetched)?;

    for (table, fields) in TABLES {
        let rows = fetch_table(table, fields)?;
        let path = out.join("cargo").join(format!("{table}.json"));
        write_if_changed(&path, &pretty_json(&rows)?).map_err(|e| io_error(&path, e))?;
        println!("{table}: {} rows", rows.len());
    }

    let redirects = fetch_redirects()?;
    let redirects_path = out.join("redirects.json");
    write_if_changed(&redirects_path, &pretty_json(&redirects)?)
        .map_err(|e| io_error(&redirects_path, e))?;
    println!("redirects.json: {} redirects", redirects.len());

    let wikitext = fetch_template_wikitext("Template:Infobox character")?;
    let template_path = out
        .join("templates")
        .join(format!("{}.wikitext", page_file_name("Infobox character")));
    let changed = write_if_changed(&template_path, &page_bytes(&wikitext))
        .map_err(|e| io_error(&template_path, e))?;
    println!(
        "templates/Infobox character.wikitext: {}",
        if changed { "written" } else { "unchanged" }
    );

    let path = out.join("index.json");
    write_if_changed(&path, &pretty_json(&index)?).map_err(|e| io_error(&path, e))?;
    println!("index.json: {} pages", index.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_bytes_normalise_line_endings() {
        assert_eq!(page_bytes("a\r\nb\n"), b"a\nb\n");
    }

    fn entry(kind: PageKind) -> IndexEntry {
        IndexEntry {
            kind,
            pageid: 1,
            revid: 1,
            timestamp: String::new(),
            category: None,
        }
    }

    /// A page is filed under the kind being fetched unless it is a translation, belongs to
    /// another kind already, or would share a file name with another title on a
    /// case-insensitive filesystem. The same title twice is the server relisting it, and it
    /// is filed again.
    #[test]
    fn admission_decides_where_a_page_goes() {
        let mut index = BTreeMap::new();
        index.insert("Cain".to_string(), entry(PageKind::Character));
        let mut fetched = KindFetch::default();
        fetched
            .titles_by_lower_name
            .insert("the_d6.wikitext".into(), "The D6".into());

        let kind = PageKind::Collectible;
        assert_eq!(
            admit("Steven/de", kind, &index, &fetched),
            Admission::Translation
        );
        assert_eq!(
            admit("Cain", kind, &index, &fetched),
            Admission::OtherKind(PageKind::Character)
        );
        assert_eq!(
            admit("The d6", kind, &index, &fetched),
            Admission::SameFileAs("The D6".into())
        );
        assert_eq!(
            admit("The D6", kind, &index, &fetched),
            Admission::Admit("The_D6.wikitext".into())
        );
        assert_eq!(
            admit("Cain", PageKind::Character, &index, &fetched),
            Admission::Admit("Cain.wikitext".into())
        );
    }

    #[test]
    fn a_wiki_own_page_is_never_admitted() {
        let index = BTreeMap::new();
        let fetched = KindFetch::default();
        assert_eq!(
            admit(
                "Binding of Isaac: Rebirth Wiki",
                PageKind::Article,
                &index,
                &fetched
            ),
            Admission::OwnWikiPage
        );
        assert_eq!(
            admit(
                "Binding of Isaac: Rebirth Wiki/Rules",
                PageKind::Article,
                &index,
                &fetched
            ),
            Admission::OwnWikiPage
        );
    }

    /// Decision 1's precedence: a page that transcludes both a collectible infobox and an
    /// entity one — `Entity`'s two templates run last of `PageKind::ALL`'s template-fetched
    /// kinds — keeps the collectible it was filed as first when `Entity`'s own run reaches
    /// the same title, and the same holds one step further out, against `Article`, which
    /// runs last of all (`fetch`'s doc comment on the `Article` skip).
    #[test]
    fn a_page_of_two_infoboxes_keeps_the_first_kind_that_claimed_it() {
        let mut index = BTreeMap::new();
        index.insert("Tonsil".to_string(), entry(PageKind::Collectible));
        let fetched = KindFetch::default();
        assert_eq!(
            admit("Tonsil", PageKind::Entity, &index, &fetched),
            Admission::OtherKind(PageKind::Collectible)
        );
        assert_eq!(
            admit("Tonsil", PageKind::Article, &index, &fetched),
            Admission::OtherKind(PageKind::Collectible)
        );
    }

    /// `file_page` writes an `Article`'s `category` into its index entry; every other kind
    /// files with `None`, since only `fetch_allpages` ever passes `Some`.
    #[test]
    fn file_page_records_an_articles_category() {
        let dir = tempfile::tempdir().unwrap();
        let mut index = BTreeMap::new();
        let mut fetched = KindFetch::default();
        let page = FetchedPage {
            title: "0 - The Fool".to_string(),
            pageid: 1,
            revid: 1,
            timestamp: "2026-01-01T00:00:00Z".into(),
            text: "text".into(),
        };
        file_page(
            page,
            PageKind::Article,
            dir.path(),
            &mut index,
            &mut fetched,
            Some(ArticleCategory::Card),
        )
        .unwrap();
        assert_eq!(
            index.get("0 - The Fool").and_then(|e| e.category),
            Some(ArticleCategory::Card)
        );
    }
}
