//! Where a fetched page is filed: `KindFetch` is what one kind's run has written and kept
//! so far, `admit` is the precedence decision (translation, own-wiki page, already another
//! kind's, a case-insensitive file-name clash, or admitted), and `file_page` writes the page
//! and updates the index accordingly. Split out of `crate::fetch`, which keeps the request
//! loops that drive this decision and reports what it did.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use wiki::{page_file_name, ArticleCategory, IndexEntry, PageKind};

use crate::api::{is_translation_subpage, FetchedPage};
use crate::fetch::page_bytes;
use crate::namespace::is_own_wiki_page;
use crate::store::write_if_changed;
use crate::{io_error, Outcome};

/// What fetching one kind leaves behind: how many files it wrote, and the names it wrote or
/// confirmed — the ones the directory's `prune` keeps, and whose count is the kind's page
/// count. It also remembers each file name's lowercase spelling, for [`admit`].
#[derive(Debug, Default)]
pub(crate) struct KindFetch {
    pub(crate) written: usize,
    pub(crate) keep: BTreeSet<String>,
    /// `page_file_name` is injective, but Windows's filesystem is case-insensitive: two
    /// titles that collide would keep the first one and warn about it. Lowercase file name →
    /// the title that took it.
    titles_by_lower_name: BTreeMap<String, String>,
    /// Pages `fetch_allpages` met that a template-fetched kind had already claimed: the
    /// expected case, since `allpages` walks every page of namespace 0 and meets all of
    /// them, never one page's own defect. Counted here instead of printed once per page
    /// ([`file_page`]), and reported as a single summary line (`crate::fetch::report_fetch`).
    pub(crate) already_filed: usize,
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

/// Writes one page and files it in the index, or says why it is not filed. `category` is
/// `Some` only when filing an `Article` that transcludes one of the four infobox templates
/// `crate::fetch::CATEGORY_TEMPLATES` declines to read.
pub(crate) fn file_page(
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
            if kind == PageKind::Article {
                // `fetch_allpages` walks every page of namespace 0, so meeting one a
                // template-fetched kind already claimed is the normal case (`admit`'s doc
                // comment), not a defect — counted, not printed per page.
                fetched.already_filed += 1;
            } else {
                // Two *template*-fetched kinds disagree on the same title: unseen on the
                // wiki so far (`fetch_categories`'s doc comment on the same situation for
                // the four declined infoboxes), worth a warning every time it happens.
                eprintln!(
                    "  warning: «{}» is already of kind {}; ignored as {}",
                    page.title,
                    prev.dir(),
                    kind.dir()
                );
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(kind: PageKind) -> IndexEntry {
        IndexEntry {
            kind,
            pageid: 1,
            revid: 1,
            timestamp: String::new(),
            category: None,
        }
    }

    fn page(title: &str) -> FetchedPage {
        FetchedPage {
            title: title.to_string(),
            pageid: 1,
            revid: 1,
            timestamp: "2026-01-01T00:00:00Z".into(),
            text: "text".into(),
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
    /// runs last of all (`crate::fetch::fetch`'s doc comment on the `Article` skip).
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
        file_page(
            page("0 - The Fool"),
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

    /// `fetch_allpages`'s expected case — a page a template-fetched kind already claimed —
    /// is tallied in `already_filed` and leaves the index entry as that kind filed it,
    /// never re-filed as `Article`.
    #[test]
    fn file_page_counts_an_article_already_claimed_by_another_kind_instead_of_warning() {
        let dir = tempfile::tempdir().unwrap();
        let mut index = BTreeMap::new();
        index.insert("Tonsil".to_string(), entry(PageKind::Collectible));
        let mut fetched = KindFetch::default();
        file_page(
            page("Tonsil"),
            PageKind::Article,
            dir.path(),
            &mut index,
            &mut fetched,
            None,
        )
        .unwrap();
        assert_eq!(fetched.already_filed, 1);
        assert_eq!(
            index.get("Tonsil").map(|e| e.kind),
            Some(PageKind::Collectible)
        );
    }

    /// Two *template*-fetched kinds disagreeing on the same title is not `fetch_allpages`'s
    /// expected case, so it is never tallied — only the `Article` path is (the test above).
    #[test]
    fn file_page_does_not_count_a_conflict_between_two_template_fetched_kinds() {
        let dir = tempfile::tempdir().unwrap();
        let mut index = BTreeMap::new();
        index.insert("Tonsil".to_string(), entry(PageKind::Collectible));
        let mut fetched = KindFetch::default();
        file_page(
            page("Tonsil"),
            PageKind::Entity,
            dir.path(),
            &mut index,
            &mut fetched,
            None,
        )
        .unwrap();
        assert_eq!(fetched.already_filed, 0);
        assert_eq!(
            index.get("Tonsil").map(|e| e.kind),
            Some(PageKind::Collectible)
        );
    }
}
