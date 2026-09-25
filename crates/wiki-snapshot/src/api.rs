//! The wiki's MediaWiki API URLs and response parsing. A pure module: no
//! networking, so request and response shapes can be verified with a test.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;
use wiki::{Row, HOST};

/// The Cargo tables to download, with the fields the `wiki` crate reads. Every one of them is a
/// table `Raw::load` opens: `stage` was downloaded here for months and read by nothing (card
/// #82, D1), which is a download that costs requests and a file that looks like data.
pub const TABLES: &[(&str, &str)] = &[
    (
        "collectible",
        "_pageName,id,dlc,alias,is_activated,unlocked_by",
    ),
    ("trinket", "_pageName,id,dlc,alias,unlocked_by"),
    (
        "achievement",
        "_pageName,id,name,alias,dlc,description,requirements,notes",
    ),
    (
        "entity",
        "_pageName,id,variant,subtype,type,alias,dlc,unlocked_by",
    ),
    (
        "challenge",
        "_pageName,number,alias,dlc,unlocked_by,unlocks",
    ),
    ("player", "_pageName,id,alias,dlc,parent"),
    (
        "transformation",
        "_pageName,id,alias,dlc,requirement,items,description,target,appearance",
    ),
    ("pickup", "_pageName,id,alias,type,dlc"),
    ("version", "_pageName,number,name,date,dlc"),
];

/// How many pages an `embeddedin` request asks for: the maximum allowed for an anonymous user.
const PAGES_PER_REQUEST: u32 = 50;
/// How many rows a `cargoquery` request asks for: the maximum allowed for an anonymous user.
pub const ROWS_PER_REQUEST: usize = 500;

/// The `continue` map of a response: the keys to pass back in the next request.
pub type Continue = BTreeMap<String, String>;

/// A page as returned by `action=query`: title, revision identifiers,
/// and wikitext.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchedPage {
    pub title: String,
    pub pageid: u64,
    pub revid: u64,
    pub timestamp: String,
    pub text: String,
}

/// A translation subpage: the title ends with `/` followed by exactly two lowercase
/// letters (`Steven/de`). `Achievements/Rebirth 1` and `20/20` are not. The `embeddedin`
/// generator lists them alongside the real pages, but the dataset is in English.
pub fn is_translation_subpage(title: &str) -> bool {
    match title.rsplit_once('/') {
        Some((page, code)) => {
            !page.is_empty() && code.len() == 2 && code.chars().all(|c| c.is_ascii_lowercase())
        }
        None => false,
    }
}

/// Sorts rows by (`_pageName`, `id`) and, when those tie, by the whole row: an entity's
/// variants arrive in the wiki's SQL order, which can change between one `fetch` and the next.
pub fn sort_rows(rows: &mut [Row]) {
    rows.sort_by(|a, b| {
        let key = |r: &Row| (r.get("_pageName").cloned(), r.get("id").cloned());
        key(a).cmp(&key(b)).then_with(|| a.cmp(b))
    });
}

/// Percent-encodes everything except alphanumerics, `-`, `_`, `.`, `~`.
pub fn url_encode(s: &str) -> String {
    s.bytes().map(url_encode_byte).collect()
}

fn url_encode_byte(b: u8) -> String {
    match b {
        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
            char::from(b).to_string()
        }
        _ => format!("%{b:02X}"),
    }
}

/// The URL that lists pages transcluding `template`, with the text of the latest revision;
/// `cont` is the `continue` map from the previous response (empty on the first request).
pub fn pages_url(template: &str, cont: &Continue) -> String {
    let base = format!(
        "{HOST}/api.php?action=query&generator=embeddedin&geititle={}&geinamespace=0\
         &geilimit={PAGES_PER_REQUEST}&prop=revisions&rvprop=content%7Cids%7Ctimestamp\
         &rvslots=main&format=json&formatversion=2&maxlag=5",
        url_encode(template)
    );
    let continuation: String = cont
        .iter()
        .map(|(k, v)| format!("&{}={}", url_encode(k), url_encode(v)))
        .collect();
    base + &continuation
}

/// The URL for a page of `ROWS_PER_REQUEST` rows of a Cargo table.
pub fn cargo_url(table: &str, fields: &str, offset: usize) -> String {
    format!(
        "{HOST}/api.php?action=cargoquery&tables={}&fields={}&limit={ROWS_PER_REQUEST}\
         &offset={offset}&format=json&maxlag=5",
        url_encode(table),
        url_encode(fields)
    )
}

/// Parses the JSON and rejects a response carrying `error` (`maxlag`, bad
/// parameters, …): the message includes the wiki's code and text.
fn parse_response(json: &str) -> Result<Value, String> {
    let v: Value = serde_json::from_str(json).map_err(|e| format!("response is not JSON: {e}"))?;
    if let Some(err) = v.get("error") {
        let code = err.get("code").and_then(Value::as_str).unwrap_or("?");
        let info = err.get("info").and_then(Value::as_str).unwrap_or("");
        return Err(format!("wiki error: {code}: {info}"));
    }
    Ok(v)
}

fn str_field(v: &Value, name: &str, what: &str) -> Result<String, String> {
    v.get(name)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("{what}: field `{name}` missing or not a string"))
}

fn u64_field(v: &Value, name: &str, what: &str) -> Result<u64, String> {
    v.get(name)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("{what}: field `{name}` missing or not an integer"))
}

/// A `pages_url` response, already parsed.
///
/// With `prop=revisions` on a generator, MediaWiki caps the batch size: when the text
/// doesn't all fit, it truncates and defers the remaining pages to the next round
/// (`rvcontinue`). Every batch, including the last one, can also include pages
/// **without** `revisions`: pages already delivered in an earlier batch (the generator
/// relists the whole set and attaches revisions only from `rvcontinue` onward), or pages
/// that were cut off and will arrive later. A single batch can't tell the two cases
/// apart: the parser lists them in `without_revision` and leaves the decision to
/// [`Pending`], which tracks the history for the whole kind.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PageBatch {
    /// The pages that arrived with text.
    pub pages: Vec<FetchedPage>,
    /// The pages listed without `revisions`: `(pageid, title)`.
    pub without_revision: Vec<(u64, String)>,
    /// The `continue` map, if there's another batch.
    pub cont: Option<Continue>,
}

/// The pages from a `pages_url` response, with and without text, plus the `continue` map.
/// A page without `revisions` is never an error here: that's decided by whoever sees all the batches.
pub fn parse_pages(json: &str) -> Result<PageBatch, String> {
    let v = parse_response(json)?;
    let cont = v
        .get("continue")
        .and_then(Value::as_object)
        .map(continue_map)
        .transpose()?;
    let mut batch = PageBatch {
        cont,
        ..PageBatch::default()
    };
    // With no results, the wiki omits `query` entirely.
    let raw_pages = match v.get("query").and_then(|q| q.get("pages")) {
        Some(p) => p
            .as_array()
            .ok_or_else(|| "query.pages is not a list".to_string())?,
        None => return Ok(batch),
    };
    for p in raw_pages {
        let title = str_field(p, "title", "page")?;
        let what = format!("page «{title}»");
        let pageid = u64_field(p, "pageid", &what)?;
        let Some(rev) = p
            .get("revisions")
            .and_then(Value::as_array)
            .and_then(|r| r.first())
        else {
            batch.without_revision.push((pageid, title));
            continue;
        };
        let text = rev
            .get("slots")
            .and_then(|s| s.get("main"))
            .and_then(|m| m.get("content"))
            .and_then(Value::as_str)
            .ok_or_else(|| format!("{what}: revision without content"))?;
        batch.pages.push(FetchedPage {
            pageid,
            revid: u64_field(rev, "revid", &what)?,
            timestamp: str_field(rev, "timestamp", &what)?,
            text: text.to_string(),
            title,
        });
    }
    Ok(batch)
}

/// The `continue` object as the next request sends it back: every value must be a string.
fn continue_map(map: &serde_json::Map<String, Value>) -> Result<Continue, String> {
    map.iter()
        .map(|(k, val)| {
            val.as_str()
                .map(|s| (k.clone(), s.to_string()))
                .ok_or_else(|| format!("continue: `{k}` is not a string"))
        })
        .collect()
}

/// The bookkeeping for one kind across its batches: who was listed without text and
/// hasn't arrived with text yet. At the end of the kind, an empty `unresolved` means the
/// snapshot is complete; otherwise pages are missing and the fetch must fail with their titles.
#[derive(Debug, Default)]
pub struct Pending {
    waiting: BTreeMap<u64, String>,
    delivered: BTreeSet<u64>,
}

impl Pending {
    /// The page showed up without `revisions`. If the text already arrived in an earlier
    /// batch, there's nothing left to wait for.
    pub fn seen_without(&mut self, pageid: u64, title: &str) {
        if !self.delivered.contains(&pageid) {
            self.waiting.insert(pageid, title.to_string());
        }
    }

    /// The page arrived with text.
    pub fn delivered(&mut self, pageid: u64) {
        self.delivered.insert(pageid);
        self.waiting.remove(&pageid);
    }

    /// The titles listed without text that never arrived, in pageid order.
    pub fn unresolved(&self) -> Vec<String> {
        self.waiting.values().cloned().collect()
    }
}

/// The rows from a `cargo_url` response: each `title` object is a row. A `null` value
/// becomes the empty string, which the `wiki` crate treats as a missing field.
pub fn parse_cargo(json: &str) -> Result<Vec<Row>, String> {
    let v = parse_response(json)?;
    let list = v
        .get("cargoquery")
        .and_then(Value::as_array)
        .ok_or_else(|| "cargoquery missing or not a list".to_string())?;
    list.iter()
        .enumerate()
        .map(|(i, item)| {
            item.get("title")
                .and_then(Value::as_object)
                .map(cargo_row)
                .ok_or_else(|| format!("row {i}: `title` missing or not an object"))
        })
        .collect()
}

/// One Cargo row: every field as text.
fn cargo_row(obj: &serde_json::Map<String, Value>) -> Row {
    obj.iter()
        .map(|(k, val)| (k.clone(), cargo_text(val)))
        .collect()
}

/// A field's value as text: `null` is the empty string, a string itself.
#[allow(clippy::wildcard_enum_match_arm)] // a foreign enum; the reason is at the wildcard arm
fn cargo_text(val: &Value) -> String {
    match val {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        // `serde_json::Value` is not ours, and every other shape is rendered as JSON text.
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The `transformation` table declares eight fields — `Special:CargoTables/transformation`,
    /// read on 2026-09-13 — and the page's own content lives in five of them. Asking for four
    /// was enough while a transformation was only a name on a link; it is not enough to state
    /// how many items it takes and which ones.
    #[test]
    fn the_transformation_query_asks_for_the_fields_that_carry_the_content() {
        let (_, fields) = TABLES
            .iter()
            .find(|(t, _)| *t == "transformation")
            .expect("the transformation table is downloaded");
        for f in [
            "requirement",
            "items",
            "description",
            "target",
            "appearance",
        ] {
            assert!(fields.contains(f), "missing field {f} in {fields}");
        }
    }

    #[test]
    fn urls() {
        let u = pages_url("Template:Infobox boss", &BTreeMap::new());
        assert!(u.starts_with("https://bindingofisaacrebirth.wiki.gg/api.php?action=query&generator=embeddedin&geititle=Template%3AInfobox%20boss"));
        assert!(
            u.contains("&geilimit=50&") && u.contains("rvslots=main") && u.contains("maxlag=5")
        );
        let mut c = BTreeMap::new();
        c.insert("geicontinue".into(), "1|2".into());
        c.insert("continue".into(), "geicontinue||".into());
        assert!(pages_url("T", &c).ends_with("&continue=geicontinue%7C%7C&geicontinue=1%7C2"));
        assert_eq!(cargo_url("collectible", "_pageName,id", 500), "https://bindingofisaacrebirth.wiki.gg/api.php?action=cargoquery&tables=collectible&fields=_pageName%2Cid&limit=500&offset=500&format=json&maxlag=5");
    }

    #[test]
    fn parse_pages_response() {
        let j = r#"{"continue":{"geicontinue":"0|Hush","continue":"geicontinue||"},"query":{"pages":[{"pageid":11419,"title":"False PHD","revisions":[{"revid":267225,"timestamp":"2026-07-30T08:21:54Z","slots":{"main":{"content":"{{infobox}}"}}}]}]}}"#;
        let b = parse_pages(j).unwrap();
        assert_eq!(b.pages.len(), 1);
        assert_eq!(b.pages[0].title, "False PHD");
        assert_eq!(b.pages[0].revid, 267225);
        assert_eq!(b.pages[0].text, "{{infobox}}");
        assert!(b.without_revision.is_empty());
        assert_eq!(
            b.cont.unwrap().get("geicontinue").map(String::as_str),
            Some("0|Hush")
        );
        let b = parse_pages(r#"{"batchcomplete":true,"query":{"pages":[]}}"#).unwrap();
        assert!(b.cont.is_none());
        assert!(parse_pages(r#"{"error":{"code":"maxlag","info":"x"}}"#).is_err());
    }

    #[test]
    fn parse_pages_without_query_is_empty_and_malformed_page_is_error() {
        let b = parse_pages(r#"{"batchcomplete":true}"#).unwrap();
        assert_eq!(b, PageBatch::default());
        let no_text = r#"{"query":{"pages":[{"pageid":1,"title":"X","revisions":[{"revid":1,"timestamp":"t","slots":{}}]}]}}"#;
        assert!(parse_pages(no_text).unwrap_err().contains("X"));
        assert!(parse_pages("not json").is_err());
    }

    #[test]
    fn pages_without_revisions_are_listed_apart_in_any_batch() {
        // Truncated batch: «Hush» is listed without `revisions`, it will arrive with text later.
        let truncated = r#"{"continue":{"rvcontinue":"20260101|1","continue":"||"},"query":{"pages":[{"pageid":11419,"title":"False PHD","revisions":[{"revid":1,"timestamp":"2026-07-30T08:21:54Z","slots":{"main":{"content":"a"}}}]},{"pageid":5,"title":"Hush"}]}}"#;
        let b = parse_pages(truncated).unwrap();
        assert_eq!(b.pages.len(), 1);
        assert_eq!(b.without_revision, vec![(5, "Hush".to_string())]);
        assert!(b.cont.is_some());
        // Last batch: a page without `revisions` isn't an error by itself — it's the
        // normal case of a page already delivered in an earlier batch. `Pending` decides.
        let last = r#"{"batchcomplete":true,"query":{"pages":[{"pageid":5,"title":"Hush"},{"pageid":1,"title":"X","revisions":[]}]}}"#;
        let b = parse_pages(last).unwrap();
        assert!(b.pages.is_empty());
        assert_eq!(
            b.without_revision,
            vec![(5, "Hush".to_string()), (1, "X".to_string())]
        );
        assert!(b.cont.is_none());
    }

    #[test]
    fn pending_resolves_pages_across_batches_of_one_kind() {
        // The sequence observed on the wiki for characters: batch 1 with all pages
        // carrying text and a `continue`; batch 2 (without `continue`) relisting the first nine
        // without `revisions`, because they were already delivered.
        let mut p = Pending::default();
        for id in 1..=9 {
            p.delivered(id);
        }
        for id in 1..=9 {
            p.seen_without(id, &format!("already delivered {id}"));
        }
        assert!(p.unresolved().is_empty());

        // Truncated batch: two pages listed without text; one arrives later, the other never does.
        let mut p = Pending::default();
        p.seen_without(5, "Hush");
        p.seen_without(7, "Delirium");
        p.delivered(5);
        assert_eq!(p.unresolved(), vec!["Delirium".to_string()]);
        // Relisted again without text after delivery: it stays resolved.
        p.seen_without(5, "Hush");
        assert_eq!(p.unresolved(), vec!["Delirium".to_string()]);
    }

    #[test]
    fn parse_cargo_response() {
        let rows = parse_cargo(
            r#"{"cargoquery":[{"title":{"_pageName":"False PHD","id":"654","is activated":"0"}}]}"#,
        )
        .unwrap();
        assert_eq!(rows[0].get("id").map(String::as_str), Some("654"));
        assert_eq!(rows[0].get("is activated").map(String::as_str), Some("0"));
        let rows = parse_cargo(r#"{"cargoquery":[{"title":{"alias":null}}]}"#).unwrap();
        assert_eq!(rows[0].get("alias").map(String::as_str), Some(""));
        assert!(parse_cargo(r#"{"error":{"code":"badtable","info":"x"}}"#).is_err());
    }

    /// A value that is neither a string nor null is kept as its JSON text; a row that is not an
    /// object under `title` is an error naming its position; the rows keep the response's order.
    #[test]
    fn parse_cargo_keeps_other_values_as_json_text_and_names_a_malformed_row() {
        let rows = parse_cargo(
            r#"{"cargoquery":[{"title":{"n":5,"b":true,"l":[1]}},{"title":{"n":"x"}}]}"#,
        )
        .unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].get("n").map(String::as_str), Some("5"));
        assert_eq!(rows[0].get("b").map(String::as_str), Some("true"));
        assert_eq!(rows[0].get("l").map(String::as_str), Some("[1]"));
        assert_eq!(rows[1].get("n").map(String::as_str), Some("x"));
        assert_eq!(
            parse_cargo(r#"{"cargoquery":[{"title":{}},{"title":3}]}"#),
            Err("row 1: `title` missing or not an object".to_string())
        );
    }

    /// Every byte outside the unreserved set is encoded on its own, a multi-byte character too.
    #[test]
    fn url_encode_encodes_each_reserved_byte() {
        assert_eq!(url_encode("aZ9-_.~"), "aZ9-_.~");
        assert_eq!(url_encode("a b/é"), "a%20b%2F%C3%A9");
    }

    /// A `continue` value that is not a string is an error naming its key.
    #[test]
    fn parse_pages_refuses_a_continue_value_that_is_not_a_string() {
        assert_eq!(
            parse_pages(r#"{"continue":{"geicontinue":"1","n":2}}"#),
            Err("continue: `n` is not a string".to_string())
        );
        let b = parse_pages(r#"{"continue":{"a":"1","b":"2"}}"#).unwrap();
        let expected: Continue = [("a".into(), "1".into()), ("b".into(), "2".into())].into();
        assert_eq!(b.cont, Some(expected));
    }

    #[test]
    fn translation_subpages_end_with_a_two_letter_code() {
        assert!(is_translation_subpage("Death's List/de"));
        assert!(is_translation_subpage("Steven/fr"));
        assert!(is_translation_subpage("A/B/es"));
        assert!(!is_translation_subpage("Achievements/Rebirth 1"));
        assert!(!is_translation_subpage("Steven"));
        assert!(!is_translation_subpage("20/20"));
        assert!(!is_translation_subpage("Page/DE"));
        assert!(!is_translation_subpage("Page/deu"));
        assert!(!is_translation_subpage("Page/d"));
        assert!(!is_translation_subpage("/de"));
        assert!(!is_translation_subpage("Page/"));
    }

    #[test]
    fn rows_sort_by_page_and_id_with_the_whole_row_as_tiebreak() {
        let row = |pairs: &[(&str, &str)]| -> Row {
            pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect()
        };
        // Two variants of the same entity, in the wiki's SQL order (1 before 0).
        let mut rows = vec![
            row(&[
                ("_pageName", "Ultra Greed"),
                ("id", "406"),
                ("variant", "1"),
            ]),
            row(&[
                ("_pageName", "Ultra Greed"),
                ("id", "406"),
                ("variant", "0"),
            ]),
            row(&[("_pageName", "Mom"), ("id", "45"), ("variant", "0")]),
            row(&[("id", "1")]),
        ];
        sort_rows(&mut rows);
        let variants: Vec<Option<&str>> = rows
            .iter()
            .map(|r| r.get("variant").map(String::as_str))
            .collect();
        assert_eq!(rows[0].get("_pageName"), None);
        assert_eq!(rows[1].get("_pageName").map(String::as_str), Some("Mom"));
        assert_eq!(variants[2..], [Some("0"), Some("1")]);
        // Same input in a different order, same output.
        let mut again = vec![
            rows[3].clone(),
            rows[1].clone(),
            rows[2].clone(),
            rows[0].clone(),
        ];
        sort_rows(&mut again);
        assert_eq!(again, rows);
    }

    #[test]
    fn encoding_keeps_unreserved_and_escapes_the_rest() {
        assert_eq!(url_encode("a-b_c.d~e"), "a-b_c.d~e");
        assert_eq!(url_encode("Mom's Knife?"), "Mom%27s%20Knife%3F");
        assert_eq!(url_encode("è"), "%C3%A8");
    }
}
