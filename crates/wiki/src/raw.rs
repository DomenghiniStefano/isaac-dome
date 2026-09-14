//! Reading `dataset/raw/`, the raw snapshot downloaded from the wiki: the crate's only
//! I/O. Layout: `index.json` (title → kind, pageid, revid, timestamp), a `pages/<kind>/`
//! folder with a `.wikitext` per page, `cargo/<table>.json` with the cargoquery rows.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::page::PageKind;
use crate::resolver::{Row, Tables};

/// An `index.json` entry: what the wiki says about the page besides its text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexEntry {
    pub kind: PageKind,
    pub pageid: u64,
    pub revid: u64,
    pub timestamp: String,
}

/// A page as it is on disk: title, index entry and wikitext.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawPage {
    pub title: String,
    pub index: IndexEntry,
    pub text: String,
}

/// The whole raw snapshot: pages (in title order), Cargo tables, and the `version` table
/// with the game's patches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Raw {
    pub pages: Vec<RawPage>,
    pub tables: Tables,
    pub versions: Vec<Row>,
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
    let mut out = String::with_capacity(title.len());
    for c in title.chars() {
        match c {
            ' ' => out.push('_'),
            '?' | ':' | '*' | '"' | '<' | '>' | '|' | '/' | '\\' | '%' => {
                out.push_str(&format!("%{:02X}", c as u32));
            }
            _ => out.push(c),
        }
    }
    out
}

/// Reads a text file; `Missing` if it doesn't exist, `Unreadable` for every other error.
fn read_text(path: &Path) -> Result<String, RawError> {
    std::fs::read_to_string(path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => RawError::Missing(path.to_path_buf()),
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

impl Raw {
    /// Reads the snapshot in `dir`. Cargo tables other than `collectible` may be missing
    /// (the wiki might not have them): the vector stays empty. A page listed in the index
    /// but absent on disk is an error: the snapshot is incomplete.
    pub fn load(dir: &Path) -> Result<Raw, RawError> {
        let index_path = dir.join("index.json");
        let index: BTreeMap<String, IndexEntry> = serde_json::from_str(&read_text(&index_path)?)
            .map_err(|e| RawError::BadIndex(e.to_string()))?;
        let mut pages = Vec::with_capacity(index.len());
        for (title, entry) in index {
            let path = dir
                .join("pages")
                .join(entry.kind.dir())
                .join(format!("{}.wikitext", page_file_name(&title)));
            pages.push(RawPage {
                title,
                index: entry,
                text: read_text(&path)?,
            });
        }
        let cargo = dir.join("cargo");
        let table = |name: &str| -> Result<Vec<Row>, RawError> {
            Ok(read_table(&cargo.join(format!("{name}.json")))?.unwrap_or_default())
        };
        let collectible_path = cargo.join("collectible.json");
        let collectible =
            read_table(&collectible_path)?.ok_or(RawError::Missing(collectible_path))?;
        let tables = Tables {
            collectible,
            trinket: table("trinket")?,
            achievement: table("achievement")?,
            entity: table("entity")?,
            challenge: table("challenge")?,
            player: table("player")?,
            transformation: table("transformation")?,
            pickup: table("pickup")?,
        };
        Ok(Raw {
            pages,
            tables,
            versions: table("version")?,
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
        std::fs::remove_file(p.join("pages/collectible/Breakfast.wikitext")).unwrap();
        assert!(matches!(Raw::load(p), Err(RawError::Missing(_))));
    }
}
