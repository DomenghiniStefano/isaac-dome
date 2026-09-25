//! View-model of "what we managed to extract from the game's archives".
//!
//! Like everything else in `ipc`: pure logic, no I/O. The bytes are read by the caller.

use serde::Serialize;
use unpack::{ArchiveFault, ArchiveInfo, BrokenArchive, CompressionMode};

use crate::catalog_view::CatalogView;
use crate::reasons::IoReason;
use crate::wiki::WikiInfo;

/// An archive as the UI sees it.
#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveView {
    pub name: String,
    pub mode: ArchiveMode,
    pub entries: usize,
}

/// The compression mode, remapped onto an enum **of our own**.
///
/// `unpack::CompressionMode` has a newtype variant (`Unknown(u8)`), a shape the project's
/// rules forbid past the IPC boundary. Here it becomes a tagged struct variant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ArchiveMode {
    Bogocrypt1,
    Lzw,
    MiniZ,
    Bogocrypt2,
    Unknown { value: u8 },
}

/// Translates the opened archives into view-models.
pub fn archive_views(archives: &[ArchiveInfo]) -> Vec<ArchiveView> {
    archives
        .iter()
        .map(|a| ArchiveView {
            name: a.name.clone(),
            mode: match a.mode {
                CompressionMode::Bogocrypt1 => ArchiveMode::Bogocrypt1,
                CompressionMode::Lzw => ArchiveMode::Lzw,
                CompressionMode::MiniZ => ArchiveMode::MiniZ,
                CompressionMode::Bogocrypt2 => ArchiveMode::Bogocrypt2,
                CompressionMode::Unknown(v) => ArchiveMode::Unknown { value: v },
            },
            entries: a.entries,
        })
        .collect()
}

/// An archive the install has and that did not open (card #80, R6). Its name is one of the
/// game's own archive names, never a path.
#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct BrokenArchiveView {
    pub name: String,
    pub reason: ArchiveReason,
}

/// Why an archive that is there did not open: the three cases a save has (`SaveReason`),
/// because they are the three things a file can do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ArchiveReason {
    /// Shorter than an archive header.
    TooShort,
    /// The `ARCH000` signature isn't there. The bytes found are not carried.
    BadMagic,
    Io {
        reason: IoReason,
    },
}

impl From<&ArchiveFault> for ArchiveReason {
    fn from(fault: &ArchiveFault) -> Self {
        match fault {
            ArchiveFault::TooShort => ArchiveReason::TooShort,
            ArchiveFault::BadMagic => ArchiveReason::BadMagic,
            ArchiveFault::Io { kind } => ArchiveReason::Io {
                reason: (*kind).into(),
            },
        }
    }
}

/// Translates the archives that did not open into view-models.
pub fn broken_archive_views(broken: &[BrokenArchive]) -> Vec<BrokenArchiveView> {
    broken
        .iter()
        .map(|b| BrokenArchiveView {
            name: b.name.clone(),
            reason: (&b.fault).into(),
        })
        .collect()
}

/// Wraps a PNG's bytes in a `data:` URL, ready for an `<img>`.
pub fn data_url(png: &[u8]) -> String {
    format!("data:image/png;base64,{}", base64(png))
}

/// Standard base64 with padding. Twenty lines instead of one more dependency,
/// for the one use we make of it.
fn base64(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    data.chunks(3)
        .flat_map(|chunk| {
            // A chunk of `chunks(3)` is never empty; the missing bytes of the last one are
            // zero, and the characters they would have produced are padding.
            let byte = |i: usize| chunk.get(i).copied().unwrap_or(0);
            let n = u32::from_be_bytes([0, byte(0), byte(1), byte(2)]);
            (0..4).map(move |i| {
                if i <= chunk.len() {
                    let sextet = (n >> (18 - 6 * i)) & 0x3F;
                    ALPHABET[sextet as usize] as char
                } else {
                    '='
                }
            })
        })
        .collect()
}

/// Everything the verification screen shows about the extraction.
#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ExtractionReport {
    pub archives: Vec<ArchiveView>,
    /// The archives that are there and did not open: not in `archives`, not in the total.
    pub broken: Vec<BrokenArchiveView>,
    /// Total entries across the indexes of the opened archives.
    pub total_entries: usize,
    /// `None` when the catalog couldn't be read: game absent, or `items.xml` unreadable.
    pub catalog: Option<CatalogView>,
    pub sprites: Vec<SpriteView>,
    /// The state of the wiki dataset: independent of the archives, loaded or not
    /// regardless of how the extraction went.
    pub wiki: WikiInfo,
}

/// An icon already resolved: the frontend sees neither paths nor raw bytes.
#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct SpriteView {
    pub id: u32,
    pub name: String,
    pub data_url: String,
}

/// Assembles the report. The icons arrive already extracted: nothing here touches disk.
pub fn extraction_report(
    archives: Vec<ArchiveView>,
    broken: Vec<BrokenArchiveView>,
    catalog: Option<CatalogView>,
    sprites: Vec<(u32, String, String)>,
    wiki: WikiInfo,
) -> ExtractionReport {
    ExtractionReport {
        total_entries: archives.iter().map(|a| a.entries).sum(),
        archives,
        broken,
        catalog,
        sprites: sprites
            .into_iter()
            .map(|(id, name, data_url)| SpriteView { id, name, data_url })
            .collect(),
        wiki,
    }
}
