//! The `fetch` command: downloads every page, Cargo table, redirect and template default
//! from the wiki into `dataset/raw/` (`crate::store` writes them, `crate::http` talks to
//! the network). `crate::main`'s `build` command reads what this writes; it never runs
//! from here. Where a downloaded page is filed is `crate::admit`'s decision, not this
//! module's: this one drives the request loops and reports what they did.

use std::collections::BTreeMap;
use std::path::Path;

use wiki::{page_file_name, ArticleCategory, IndexEntry, PageKind, Row, CONTENT_TEMPLATES};

use crate::admit::{file_page, KindFetch};
use crate::api::{
    cargo_url, pages_url, parse_cargo, parse_pages, sort_rows, Pending, ROWS_PER_REQUEST, TABLES,
};
use crate::namespace::{
    allpages_url, embeddedin_titles_url, parse_embeddedin_titles, parse_redirects, redirects_url,
    template_wikitext_url,
};
use crate::store::{prune, write_if_changed};
use crate::{io_error, Failure, Outcome};

/// A page's text as it goes to disk: LF line endings, because the repo forces LF in the
/// working copy and a downloaded CRLF would get rewritten on every `fetch`. `pub(crate)`:
/// `crate::admit::file_page` writes a page's own text the same way.
pub(crate) fn page_bytes(text: &str) -> Vec<u8> {
    text.replace("\r\n", "\n").into_bytes()
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

/// The five infobox templates decision 2 declines to read: an article that transcludes one
/// is tagged with the matching category, so the landing can tell a card from a mechanic
/// page without reading the infobox's own parameters. **Most specific wins**: the list is
/// ordered by specificity, card and rune through version down to pickup (which the card and
/// rune templates transclude), and the first match in the fetched categories is kept — later
/// visits to the same title do not overwrite it.
const CATEGORY_TEMPLATES: &[(&str, ArticleCategory)] = &[
    ("Template:Infobox card", ArticleCategory::Card),
    ("Template:Infobox rune", ArticleCategory::Rune),
    ("Template:Infobox stage", ArticleCategory::Stage),
    ("Template:Infobox version", ArticleCategory::Version),
    ("Template:Infobox pickup", ArticleCategory::Pickup),
];

/// Every title transcluding one of `CATEGORY_TEMPLATES`, title → category. Titles only
/// (`embeddedin_titles_url`): the category is all `index.json` keeps of these five
/// infoboxes, so their wikitext is never fetched. A title that transcludes more than one
/// (the normal case for card and rune, which transclude pickup) keeps the first match from
/// `CATEGORY_TEMPLATES` — an explicit first-wins, which makes specificity matter.
fn fetch_categories() -> Result<BTreeMap<String, ArticleCategory>, Failure> {
    let mut categories = BTreeMap::new();
    for (template, category) in CATEGORY_TEMPLATES {
        let mut cont = BTreeMap::new();
        loop {
            let body = crate::http::get(&embeddedin_titles_url(template, &cont))
                .map_err(Failure::Error)?;
            let batch = parse_embeddedin_titles(&body).map_err(Failure::Error)?;
            for title in batch.titles {
                categories.entry(title).or_insert(*category);
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
/// A non-zero `already_filed` — only ever `fetch_allpages`'s — adds the fourth line.
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
    if fetched.already_filed > 0 {
        println!(
            "{label}: {} pages already filed under another kind",
            fetched.already_filed
        );
    }
    Ok(())
}

/// Writes `index.json`, the same way `fetch` does at the end: called after every kind's
/// pages are fetched and its directory pruned, not only once `fetch` as a whole succeeds.
/// A kind's directory on disk and `index.json`'s entries for it are written by two
/// different steps (`report_fetch` prunes, this writes the map); an error partway through a
/// *later* kind — or the tables, redirects, or template fetch that follow — must not leave
/// the directories of every *earlier* one ahead of an `index.json` that still describes the
/// snapshot before this run.
fn write_index(out: &Path, index: &BTreeMap<String, IndexEntry>) -> Outcome {
    let path = out.join("index.json");
    write_if_changed(&path, &pretty_json(index)?).map_err(|e| io_error(&path, e))?;
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
        write_index(out, &index)?;
    }

    let categories = fetch_categories()?;
    let article_dir = out.join("pages").join(PageKind::Article.dir());
    let fetched = fetch_allpages(&article_dir, &mut index, &categories)?;
    report_fetch(PageKind::Article.dir(), &article_dir, &fetched)?;
    write_index(out, &index)?;

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

    for title in CONTENT_TEMPLATES {
        let wikitext = fetch_template_wikitext(&format!("Template:{title}"))?;
        let template_path = out
            .join("templates")
            .join(format!("{}.wikitext", page_file_name(title)));
        let changed = write_if_changed(&template_path, &page_bytes(&wikitext))
            .map_err(|e| io_error(&template_path, e))?;
        println!(
            "templates/{title}.wikitext: {}",
            if changed { "written" } else { "unchanged" }
        );
    }

    write_index(out, &index)?;
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

    #[test]
    fn category_first_match_wins_not_last() {
        // Simulates a title transcluding both card and pickup infoboxes (e.g., "0 - The Fool").
        // Since card template transcludes pickup template, both appear in the fetched results.
        // The first match in CATEGORY_TEMPLATES should win: card should be kept, not overwritten
        // by pickup.
        let fool = "0 - The Fool";
        let mut categories_card_first = BTreeMap::new();
        categories_card_first.insert(fool.to_string(), ArticleCategory::Card);
        categories_card_first
            .entry(fool.to_string())
            .or_insert(ArticleCategory::Pickup);
        assert_eq!(
            categories_card_first.get(fool),
            Some(&ArticleCategory::Card)
        );

        // Verify the opposite order with `or_insert` also results in Card being kept
        // (because `or_insert` only inserts if the key is absent).
        let mut categories_pickup_first = BTreeMap::new();
        categories_pickup_first.insert(fool.to_string(), ArticleCategory::Pickup);
        categories_pickup_first
            .entry(fool.to_string())
            .or_insert(ArticleCategory::Card);
        assert_eq!(
            categories_pickup_first.get(fool),
            Some(&ArticleCategory::Pickup)
        );

        // The actual fetch_categories iterates CATEGORY_TEMPLATES in order (card before pickup),
        // so the real scenario corresponds to the first case above.
    }
}
