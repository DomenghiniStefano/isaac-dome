//! Reading `dataset/raw/`, the raw snapshot downloaded from the wiki: the crate's only
//! I/O. Layout: `index.json` (title → kind, pageid, revid, timestamp), a `pages/<kind>/`
//! folder with a `.wikitext` per page, `cargo/<table>.json` with the cargoquery rows,
//! `redirects.json` (`from` → `to`, canonical titles) and `templates/<name>.wikitext`
//! (a template's own wikitext, for the defaults it declares). The last two come with
//! the whole-namespace fetch and are absent from a snapshot taken without it: both
//! degrade to empty rather than fail.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::page::PageKind;
use crate::resolver::{Row, Tables};

/// The infobox template an article was found to transclude, when it transcludes one of the
/// five whose parameters this sub-project declines to read (design decision 2): a card, a
/// rune, a pickup, a stage, or a version. It is not a `PageKind` — the page is still filed
/// as `Article` — but it is how the landing tells a card from a mechanic page.
///
/// Crosses the IPC as part of `Infobox::Article` (`model.rs`): fieldless, so a bare
/// camelCase string, same as every variant name here since each is one word.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum ArticleCategory {
    Card,
    Rune,
    Pickup,
    Stage,
    Version,
}

/// An `index.json` entry: what the wiki says about the page besides its text. `category`
/// exists only for an `Article`; `#[serde(default)]` reads it as `None` on every entry a
/// snapshot taken without the whole-namespace fetch wrote without the field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexEntry {
    pub kind: PageKind,
    pub pageid: u64,
    pub revid: u64,
    pub timestamp: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<ArticleCategory>,
}

/// A page as it is on disk: title, index entry and wikitext.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawPage {
    pub title: String,
    pub index: IndexEntry,
    pub text: String,
}

/// The whole raw snapshot: pages (in title order), Cargo tables, the `version` table with
/// the game's patches, the redirect map, and a template's own wikitext for its declared
/// defaults.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Raw {
    pub pages: Vec<RawPage>,
    pub tables: Tables,
    pub versions: Vec<Row>,
    /// `from` → `to`, both canonical titles (`title::canonical_title`), sorted. Empty when
    /// `redirects.json` is absent — a snapshot has none until the whole-namespace fetch runs.
    pub redirects: BTreeMap<String, String>,
    /// `Template:Infobox character`'s own wikitext, read once for the base-stat defaults it
    /// declares (design decision 4). `None` until the template-defaults fetch runs; the
    /// parser that reads the defaults is a later step, not this one.
    pub template_infobox_character: Option<String>,
}

#[derive(Debug)]
pub enum RawError {
    /// A file that must exist doesn't: `index.json`, a listed page, `collectible.json`.
    Missing(PathBuf),
    /// The file exists but doesn't read, or isn't the expected JSON.
    Unreadable(PathBuf, String),
    /// `index.json` isn't the expected map.
    BadIndex(String),
}

impl fmt::Display for RawError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RawError::Missing(p) => write!(f, "missing file: {}", p.display()),
            RawError::Unreadable(p, reason) => {
                write!(f, "unreadable file: {} ({reason})", p.display())
            }
            RawError::BadIndex(reason) => write!(f, "invalid index.json: {reason}"),
        }
    }
}

impl std::error::Error for RawError {}

/// A page's file name: spaces → `_`, characters forbidden on Windows or ambiguous in a
/// path (`? : * " < > | / \ %`) → `%XX`. Reversible and stable across systems.
pub fn page_file_name(title: &str) -> String {
    title.chars().map(file_name_char).collect()
}

/// One character of a title as a file name spells it.
fn file_name_char(c: char) -> String {
    match c {
        ' ' => "_".to_string(),
        '?' | ':' | '*' | '"' | '<' | '>' | '|' | '/' | '\\' | '%' => format!("%{:02X}", c as u32),
        _ => c.to_string(),
    }
}

/// Reads a text file; `Missing` if it doesn't exist, `Unreadable` for every other error.
#[allow(clippy::wildcard_enum_match_arm)] // a foreign enum; the reason is at the wildcard arm
fn read_text(path: &Path) -> Result<String, RawError> {
    std::fs::read_to_string(path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => RawError::Missing(path.to_path_buf()),
        // `ErrorKind` is `#[non_exhaustive]` and not ours: "every other error" is the contract.
        _ => RawError::Unreadable(path.to_path_buf(), e.to_string()),
    })
}

/// Reads a Cargo table, `Vec<Row>`. A missing file gives `None`: it's up to the caller to
/// decide whether the table is mandatory.
fn read_table(path: &Path) -> Result<Option<Vec<Row>>, RawError> {
    let text = match read_text(path) {
        Ok(t) => t,
        Err(RawError::Missing(_)) => return Ok(None),
        Err(e) => return Err(e),
    };
    serde_json::from_str(&text)
        .map(Some)
        .map_err(|e| RawError::Unreadable(path.to_path_buf(), e.to_string()))
}

/// A Cargo table that may be missing: the wiki might not have it, and the vector stays empty.
fn optional_table(cargo: &Path, name: &str) -> Result<Vec<Row>, RawError> {
    Ok(read_table(&cargo.join(format!("{name}.json")))?.unwrap_or_default())
}

/// Every page `index.json` lists, with its text. A listed page absent on disk is an error:
/// the snapshot is incomplete.
fn load_pages(dir: &Path) -> Result<Vec<RawPage>, RawError> {
    let index: BTreeMap<String, IndexEntry> =
        serde_json::from_str(&read_text(&dir.join("index.json"))?)
            .map_err(|e| RawError::BadIndex(e.to_string()))?;
    index
        .into_iter()
        .map(|(title, index)| {
            let path = dir
                .join("pages")
                .join(index.kind.dir())
                .join(format!("{}.wikitext", page_file_name(&title)));
            Ok(RawPage {
                text: read_text(&path)?,
                title,
                index,
            })
        })
        .collect()
}

/// The tables the resolver reads. `collectible` is the one that must exist.
fn load_tables(cargo: &Path) -> Result<Tables, RawError> {
    let collectible_path = cargo.join("collectible.json");
    let collectible = read_table(&collectible_path)?.ok_or(RawError::Missing(collectible_path))?;
    Ok(Tables {
        collectible,
        trinket: optional_table(cargo, "trinket")?,
        achievement: optional_table(cargo, "achievement")?,
        entity: optional_table(cargo, "entity")?,
        challenge: optional_table(cargo, "challenge")?,
        player: optional_table(cargo, "player")?,
        transformation: optional_table(cargo, "transformation")?,
        pickup: optional_table(cargo, "pickup")?,
    })
}

/// `redirects.json` (`{ "from": "to" }`) if present; a missing file degrades to an empty
/// map, same as an absent Cargo table — the fetch that writes it is new, and an old
/// snapshot never wrote it.
fn load_redirects(dir: &Path) -> Result<BTreeMap<String, String>, RawError> {
    let path = dir.join("redirects.json");
    match read_text(&path) {
        Ok(text) => serde_json::from_str(&text)
            .map_err(|e| RawError::Unreadable(path.clone(), e.to_string())),
        Err(RawError::Missing(_)) => Ok(BTreeMap::new()),
        Err(e) => Err(e),
    }
}

/// A template's own wikitext (`templates/<name>.wikitext`), if the fetch downloaded it.
/// A missing file is `None`, not an error: the template-defaults fetch is new, and the
/// parser that reads it treats absence like every other "the game/wiki didn't say" case.
fn load_template(dir: &Path, title: &str) -> Result<Option<String>, RawError> {
    let path = dir
        .join("templates")
        .join(format!("{}.wikitext", page_file_name(title)));
    match read_text(&path) {
        Ok(text) => Ok(Some(text)),
        Err(RawError::Missing(_)) => Ok(None),
        Err(e) => Err(e),
    }
}

impl Raw {
    /// Reads the snapshot in `dir`. Cargo tables other than `collectible` may be missing
    /// (the wiki might not have them): the vector stays empty. A page listed in the index
    /// but absent on disk is an error: the snapshot is incomplete. `redirects.json` and the
    /// template wikitext degrade the same way the optional Cargo tables do.
    pub fn load(dir: &Path) -> Result<Raw, RawError> {
        let cargo = dir.join("cargo");
        Ok(Raw {
            pages: load_pages(dir)?,
            tables: load_tables(&cargo)?,
            versions: optional_table(&cargo, "version")?,
            redirects: load_redirects(dir)?,
            template_infobox_character: load_template(dir, "Infobox character")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_names() {
        assert_eq!(page_file_name("False PHD"), "False_PHD");
        assert_eq!(page_file_name("??? (Boss)"), "%3F%3F%3F_(Boss)");
        assert_eq!(page_file_name("Mom's Knife"), "Mom's_Knife");
        assert_eq!(
            page_file_name("Achievements/Rebirth 1"),
            "Achievements%2FRebirth_1"
        );
        assert_eq!(page_file_name("100% Fun"), "100%25_Fun");
    }

    #[test]
    fn load_reads_index_pages_and_tables() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        std::fs::create_dir_all(p.join("pages/collectible")).unwrap();
        std::fs::create_dir_all(p.join("cargo")).unwrap();
        std::fs::write(
            p.join("index.json"),
            r#"{"Breakfast":{"kind":"collectible","pageid":1,"revid":7,"timestamp":"2026-01-01T00:00:00Z"}}"#,
        )
        .unwrap();
        std::fs::write(
            p.join("pages/collectible/Breakfast.wikitext"),
            "{{infobox passive collectible|id=25}}",
        )
        .unwrap();
        std::fs::write(
            p.join("cargo/collectible.json"),
            r#"[{"_pageName":"Breakfast","id":"25","alias":"Breakfast"}]"#,
        )
        .unwrap();
        let raw = Raw::load(p).unwrap();
        assert_eq!(raw.pages.len(), 1);
        assert_eq!(raw.pages[0].title, "Breakfast");
        assert_eq!(raw.tables.collectible.len(), 1);
        assert!(raw.tables.trinket.is_empty());
        // Neither file exists in this fixture: both degrade to empty rather than fail,
        // the same as an optional Cargo table — a snapshot taken without the
        // whole-namespace fetch has neither.
        assert!(raw.redirects.is_empty());
        assert!(raw.template_infobox_character.is_none());
        std::fs::remove_file(p.join("pages/collectible/Breakfast.wikitext")).unwrap();
        assert!(matches!(Raw::load(p), Err(RawError::Missing(_))));
    }

    /// `redirects.json` and the template's wikitext, when the fetch wrote them: read back
    /// exactly what was written, keyed and valued as canonical titles (the writer's job, not
    /// this reader's — `raw.rs` trusts what is on disk).
    #[test]
    fn load_reads_redirects_and_the_template_default_when_present() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        std::fs::create_dir_all(p.join("cargo")).unwrap();
        std::fs::create_dir_all(p.join("templates")).unwrap();
        std::fs::write(p.join("index.json"), "{}").unwrap();
        std::fs::write(
            p.join("cargo/collectible.json"),
            r#"[{"_pageName":"Breakfast","id":"25"}]"#,
        )
        .unwrap();
        std::fs::write(
            p.join("redirects.json"),
            r#"{"Soul Hearts":"Health","Tears Up":"Tears"}"#,
        )
        .unwrap();
        std::fs::write(
            p.join("templates/Infobox_character.wikitext"),
            "{{{damage|3.5}}}",
        )
        .unwrap();
        let raw = Raw::load(p).unwrap();
        assert_eq!(
            raw.redirects.get("Soul Hearts").map(String::as_str),
            Some("Health")
        );
        assert_eq!(raw.redirects.len(), 2);
        assert_eq!(
            raw.template_infobox_character.as_deref(),
            Some("{{{damage|3.5}}}")
        );
    }

    /// An `Article`'s `category` round-trips through `index.json`; every other kind's stays
    /// absent from the JSON (`skip_serializing_if`) and reads back as `None`.
    #[test]
    fn an_articles_category_round_trips_and_is_absent_for_other_kinds() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        std::fs::create_dir_all(p.join("pages/article")).unwrap();
        std::fs::create_dir_all(p.join("pages/collectible")).unwrap();
        std::fs::create_dir_all(p.join("cargo")).unwrap();
        std::fs::write(
            p.join("index.json"),
            r#"{
                "0 - The Fool": {"kind":"article","pageid":1,"revid":1,"timestamp":"2026-01-01T00:00:00Z","category":"card"},
                "Breakfast": {"kind":"collectible","pageid":2,"revid":1,"timestamp":"2026-01-01T00:00:00Z"}
            }"#,
        )
        .unwrap();
        std::fs::write(p.join("pages/article/0_-_The_Fool.wikitext"), "text").unwrap();
        std::fs::write(
            p.join("pages/collectible/Breakfast.wikitext"),
            "{{infobox passive collectible|id=25}}",
        )
        .unwrap();
        std::fs::write(
            p.join("cargo/collectible.json"),
            r#"[{"_pageName":"Breakfast","id":"25"}]"#,
        )
        .unwrap();
        let raw = Raw::load(p).unwrap();
        let by_title = |t: &str| raw.pages.iter().find(|p| p.title == t).unwrap();
        assert_eq!(
            by_title("0 - The Fool").index.category,
            Some(ArticleCategory::Card)
        );
        assert_eq!(by_title("Breakfast").index.category, None);
        // Serializing keeps the field out entirely when it is `None`.
        let s = serde_json::to_string(&by_title("Breakfast").index).unwrap();
        assert!(!s.contains("category"), "{s}");
    }
}
