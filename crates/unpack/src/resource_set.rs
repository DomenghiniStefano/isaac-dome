//! Resolving a resource across multiple archives.
//!
//! The caller knows a resource's logical path — `gfx/items/collectibles/x.png` — and
//! shouldn't have to know which `.a` it lives in or under which root. This is where the
//! two things the caller can't guess are concentrated:
//!
//! 1. **DLC precedence.** The same path exists in multiple archives, and the most
//!    recent one wins: Repentance overrides Afterbirth+, which overrides Afterbirth,
//!    and so on.
//! 2. **The root.** Repentance uses `resources-dlc3/`, all the others `resources/`.
//!    Getting it wrong gives no error: the index only holds hashes, so a wrong root
//!    produces silence, not a visible failure.

use std::path::Path;

use crate::arch::{Archive, CompressionMode};

/// Precedence order: the first one that has the resource wins. A missing archive is skipped.
const PRECEDENCE: [&str; 8] = [
    "repentance.a",
    "afterbirthp.a",
    "afterbirth.a",
    "graphics.a",
    "animations.a",
    "music.a",
    "fonts.a",
    "config.a",
];

/// Known roots, tried in this order on every archive.
const ROOTS: [&str; 2] = ["resources-dlc3", "resources"];

/// What we know about an open archive, for diagnostics.
#[derive(Debug, Clone)]
pub struct ArchiveInfo {
    pub name: String,
    pub mode: CompressionMode,
    pub entries: usize,
}

/// An installation's archives, queryable by logical path.
pub struct ResourceSet {
    archives: Vec<(String, Archive)>,
    info: Vec<ArchiveInfo>,
}

impl ResourceSet {
    /// Opens the archives present in `packed_dir`, in precedence order.
    ///
    /// Degrades: an archive that fails to open is skipped silently, because its
    /// absence is normal (different editions of the game have a different subset).
    pub fn open(packed_dir: &Path) -> ResourceSet {
        let mut archives = Vec::new();
        let mut info = Vec::new();
        for name in PRECEDENCE {
            let Ok(a) = Archive::open(&packed_dir.join(name)) else {
                continue;
            };
            info.push(ArchiveInfo {
                name: name.to_string(),
                mode: a.mode(),
                entries: a.entries().len(),
            });
            archives.push((name.to_string(), a));
        }
        ResourceSet { archives, info }
    }

    /// The open archives, in precedence order.
    pub fn archives(&self) -> &[ArchiveInfo] {
        &self.info
    }

    /// Reads a resource from its logical path, without a root or archive name.
    pub fn read(&self, logical: &str) -> Option<Vec<u8>> {
        self.read_with_source(logical).map(|(bytes, _)| bytes)
    }

    /// Like [`read`](Self::read), but also says which archive the resource comes from.
    /// Used for diagnostics: knowing *who* won the precedence.
    pub fn read_with_source(&self, logical: &str) -> Option<(Vec<u8>, &str)> {
        if logical.is_empty() {
            return None;
        }
        let logical = logical.trim_start_matches('/');
        for (name, archive) in &self.archives {
            for root in ROOTS {
                if let Some(bytes) = archive.read(&format!("{root}/{logical}")) {
                    return Some((bytes, name));
                }
            }
        }
        None
    }

    /// Whether the resource exists, without extracting it.
    pub fn contains(&self, logical: &str) -> bool {
        if logical.is_empty() {
            return false;
        }
        let logical = logical.trim_start_matches('/');
        self.archives.iter().any(|(_, a)| {
            ROOTS
                .iter()
                .any(|root| a.contains(&format!("{root}/{logical}")))
        })
    }
}
