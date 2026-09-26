//! The whole-namespace fetch's own URLs and response parsing (design decision 1): every
//! page of namespace 0 (`allpages_url`), the redirect map (`redirects_url`), which articles
//! transclude a declined infobox (`embeddedin_titles_url`), a template's own wikitext
//! (`template_wikitext_url`), and the wiki's own pages to leave out (`is_own_wiki_page`). A
//! pure module, same as `api`, whose primitives (`with_continuation`, `parse_response`,
//! `str_field`, `response_continue`, `Continue`) it shares rather than repeats.

use serde_json::Value;
use wiki::{canonical_title, HOST};

use crate::api::{
    parse_response, response_continue, str_field, with_continuation, Continue, PAGES_PER_REQUEST,
};

/// How many results a request asking for **titles only** — no wikitext — fetches: an
/// anonymous user's cap for a plain `list` module or a generator with no `prop=revisions`
/// attached is ten times `PAGES_PER_REQUEST`, so the redirect map and the declined-infobox
/// category tallies (decisions 1 and 2) cost a tenth of the requests a content fetch would.
const TITLES_PER_REQUEST: u32 = 500;

/// The wiki's own pages: a portal introducing the wiki itself, not part of the game's
/// subject matter — decision 1 names it as the second thing the whole-namespace fetch
/// leaves out, alongside translation subpages. A short, explicit list rather than a
/// namespace test, because the main page is not in its own namespace: it is an ordinary
/// namespace-0 title that happens to be about the wiki.
const OWN_WIKI_PAGES: &[&str] = &[
    // The only portal page. Anything under it (a policy, a help page) is the wiki
    // introducing itself, never a page about the game.
    "Binding of Isaac: Rebirth Wiki",
];

/// Whether `title` is one of the wiki's own pages, or a subpage of one.
pub fn is_own_wiki_page(title: &str) -> bool {
    OWN_WIKI_PAGES.iter().any(|page| {
        title == *page
            || title
                .strip_prefix(page)
                .is_some_and(|rest| rest.starts_with('/'))
    })
}

/// The URL that lists every page of namespace 0 that is not a redirect, with the text of
/// its latest revision — the whole-namespace fetch (decision 1). Asks exactly the way
/// `crate::api::pages_url` does (same `prop`, `rvprop`, `rvslots`), because a page found
/// this way and one found by template both become a `FetchedPage` through the same
/// `crate::api::parse_pages`.
pub fn allpages_url(cont: &Continue) -> String {
    let base = format!(
        "{HOST}/api.php?action=query&generator=allpages&gapnamespace=0\
         &gapfilterredir=nonredirects&gaplimit={PAGES_PER_REQUEST}&prop=revisions\
         &rvprop=content%7Cids%7Ctimestamp&rvslots=main&format=json&formatversion=2&maxlag=5"
    );
    with_continuation(base, cont)
}

/// The URL that lists every redirect of namespace 0 together with its target: the
/// generator lists the redirect pages, `redirects=1` resolves each one and the response's
/// `query.redirects` carries the `from`/`to` pairs ([`parse_redirects`]). No `prop` is
/// asked: the map is the whole answer, so this is a titles-only request
/// (`TITLES_PER_REQUEST`), a tenth the size of a content one for the same page count.
pub fn redirects_url(cont: &Continue) -> String {
    let base = format!(
        "{HOST}/api.php?action=query&generator=allpages&gapnamespace=0\
         &gapfilterredir=redirects&gaplimit={TITLES_PER_REQUEST}&redirects=1\
         &format=json&formatversion=2&maxlag=5"
    );
    with_continuation(base, cont)
}

/// The URL that lists the titles transcluding `template`, with **no** wikitext: used for the
/// four declined infobox templates (decision 1's `category`), where only knowing "does this
/// article transclude it" is needed, never the text.
pub fn embeddedin_titles_url(template: &str, cont: &Continue) -> String {
    let base = format!(
        "{HOST}/api.php?action=query&list=embeddedin&eititle={}&einamespace=0\
         &eilimit={TITLES_PER_REQUEST}&format=json&formatversion=2&maxlag=5",
        crate::api::url_encode(template)
    );
    with_continuation(base, cont)
}

/// The URL for one template's own wikitext (namespace 10), fetched once for the base-stat
/// defaults it declares (decision 4). A single title never truncates across batches, so the
/// response is read with `crate::api::parse_pages` and its `pages` is expected to hold
/// exactly one.
pub fn template_wikitext_url(title: &str) -> String {
    format!(
        "{HOST}/api.php?action=query&titles={}&prop=revisions\
         &rvprop=content%7Cids%7Ctimestamp&rvslots=main&format=json&formatversion=2&maxlag=5",
        crate::api::url_encode(title)
    )
}

/// A [`redirects_url`] response, already parsed: every redirect this batch resolved, both
/// titles put through `wiki::canonical_title` the way `dataset/raw/redirects.json` stores
/// them, and the `continue` map for the next batch.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RedirectBatch {
    pub redirects: Vec<(String, String)>,
    pub cont: Option<Continue>,
}

/// The `from`/`to` pairs from a `redirects_url` response. A batch that resolved no redirect
/// (every title `gapcontinue` still had to walk before landing on one, or the fetch is on
/// its last, empty batch) omits `query.redirects` entirely, the same way `query` itself is
/// omitted with no results at all.
pub fn parse_redirects(json: &str) -> Result<RedirectBatch, String> {
    let v = parse_response(json)?;
    let mut batch = RedirectBatch {
        cont: response_continue(&v)?,
        ..RedirectBatch::default()
    };
    let Some(raw) = v
        .get("query")
        .and_then(|q| q.get("redirects"))
        .and_then(Value::as_array)
    else {
        return Ok(batch);
    };
    for r in raw {
        let from = canonical_title(&str_field(r, "from", "redirect")?);
        let to = canonical_title(&str_field(r, "to", "redirect")?);
        // `tofragment` (a redirect to `Page#Section`) is read and dropped on purpose: the
        // link resolves to the page, decision 1 says nothing about the section.
        batch.redirects.push((from, to));
    }
    Ok(batch)
}

/// An [`embeddedin_titles_url`] response, already parsed: titles only, plus the `continue`
/// map for the next batch.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TitleBatch {
    pub titles: Vec<String>,
    pub cont: Option<Continue>,
}

/// The titles from an `embeddedin_titles_url` response.
pub fn parse_embeddedin_titles(json: &str) -> Result<TitleBatch, String> {
    let v = parse_response(json)?;
    let mut batch = TitleBatch {
        cont: response_continue(&v)?,
        ..TitleBatch::default()
    };
    let Some(raw) = v
        .get("query")
        .and_then(|q| q.get("embeddedin"))
        .and_then(Value::as_array)
    else {
        return Ok(batch);
    };
    for item in raw {
        batch
            .titles
            .push(str_field(item, "title", "embeddedin entry")?);
    }
    Ok(batch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn allpages_and_redirects_and_embeddedin_titles_and_template_urls() {
        let u = allpages_url(&BTreeMap::new());
        assert!(u.starts_with(
            "https://bindingofisaacrebirth.wiki.gg/api.php?action=query&generator=allpages\
             &gapnamespace=0&gapfilterredir=nonredirects"
        ));
        assert!(
            u.contains("&gaplimit=50&") && u.contains("rvslots=main") && u.contains("maxlag=5")
        );
        let mut c = BTreeMap::new();
        c.insert("gapcontinue".into(), "Hush".into());
        c.insert("continue".into(), "gapcontinue||".into());
        assert!(allpages_url(&c).ends_with("&continue=gapcontinue%7C%7C&gapcontinue=Hush"));

        let u = redirects_url(&BTreeMap::new());
        assert!(u.contains("&gapfilterredir=redirects&gaplimit=500&redirects=1"));
        assert!(!u.contains("prop=revisions"), "a titles-only request: {u}");

        let u = embeddedin_titles_url("Template:Infobox card", &BTreeMap::new());
        assert!(u.contains("&list=embeddedin&eititle=Template%3AInfobox%20card"));
        assert!(u.contains("&eilimit=500") && !u.contains("prop=revisions"));

        let u = template_wikitext_url("Template:Infobox character");
        assert!(u.contains("&titles=Template%3AInfobox%20character"));
        assert!(u.contains("rvslots=main") && !u.contains("generator="));
    }

    #[test]
    fn parse_redirects_response_canonicalizes_and_drops_the_fragment() {
        let j = r#"{"continue":{"gapcontinue":"x","continue":"gapcontinue||"},"query":{"redirects":[{"from":"soul_hearts","to":"health","tofragment":"Soul Hearts"}],"pages":[{"pageid":1,"title":"Health"}]}}"#;
        let b = parse_redirects(j).unwrap();
        assert_eq!(
            b.redirects,
            vec![("Soul hearts".to_string(), "Health".to_string())]
        );
        assert!(b.cont.is_some());
        // No redirects in this batch: `query.redirects` is absent, same as `parse_pages`
        // treats an empty `query`.
        let b = parse_redirects(r#"{"batchcomplete":true,"query":{"pages":[]}}"#).unwrap();
        assert!(b.redirects.is_empty() && b.cont.is_none());
        let b = parse_redirects(r#"{"batchcomplete":true}"#).unwrap();
        assert_eq!(b, RedirectBatch::default());
        assert!(parse_redirects(r#"{"error":{"code":"maxlag","info":"x"}}"#).is_err());
    }

    #[test]
    fn parse_embeddedin_titles_response() {
        let j = r#"{"continue":{"eicontinue":"0|1","continue":"eicontinue||"},"query":{"embeddedin":[{"pageid":1,"title":"0 - The Fool"},{"pageid":2,"title":"I - The Magician"}]}}"#;
        let b = parse_embeddedin_titles(j).unwrap();
        assert_eq!(
            b.titles,
            vec!["0 - The Fool".to_string(), "I - The Magician".to_string()]
        );
        assert!(b.cont.is_some());
        let b = parse_embeddedin_titles(r#"{"batchcomplete":true}"#).unwrap();
        assert_eq!(b, TitleBatch::default());
    }

    #[test]
    fn own_wiki_pages_and_their_subpages_are_recognized() {
        assert!(is_own_wiki_page("Binding of Isaac: Rebirth Wiki"));
        assert!(is_own_wiki_page("Binding of Isaac: Rebirth Wiki/Rules"));
        assert!(!is_own_wiki_page("Binding of Isaac: Rebirth Wikipedia"));
        assert!(!is_own_wiki_page("Damage"));
    }
}
