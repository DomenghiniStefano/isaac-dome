//! Targeted extraction of a subset of paths into a cache.

use std::path::{Component, Path, PathBuf};

use crate::arch::Archive;

#[derive(Debug, Clone)]
pub struct ExtractReport {
    pub extracted: Vec<PathBuf>,
    pub missing: Vec<String>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Diagnostic {
    DecompressFailed {
        path: String,
    },
    /// The kind and never the message (card #80, R6): the OS's sentence is not translatable,
    /// and on Windows it can repeat the path it was given.
    WriteFailed {
        path: PathBuf,
        reason: std::io::ErrorKind,
    },
}

/// Joins `rel` under `cache_dir`, accepting only normal path components.
/// Rejects (`None`) any absolute path, drive/UNC prefix, `..`, or `.`.
#[allow(clippy::wildcard_enum_match_arm)] // a foreign enum; the reason is at the wildcard arm
fn safe_join(cache_dir: &Path, rel: &str) -> Option<PathBuf> {
    let rel = rel.replace('\\', "/");
    let mut dest = cache_dir.to_path_buf();
    let mut pushed = false;
    for comp in Path::new(&rel).components() {
        match comp {
            Component::Normal(c) => {
                dest.push(c);
                pushed = true;
            }
            // RootDir, Prefix (C:), ParentDir (..), CurDir (.): only a plain name stays inside the
            // cache, so refusing by default is the rule, and a component std adds later is refused too.
            _ => return None,
        }
    }
    if pushed {
        Some(dest)
    } else {
        None
    }
}

/// Extracts into `cache_dir` only the requested paths, preserving the path structure.
/// Never fails: missing → `missing`, failed decompression → `DecompressFailed`,
/// failed write → `WriteFailed`. Never writes outside `cache_dir`.
pub fn extract_subset(archive: &Archive, wanted: &[&str], cache_dir: &Path) -> ExtractReport {
    let mut report = ExtractReport {
        extracted: Vec::new(),
        missing: Vec::new(),
        diagnostics: Vec::new(),
    };

    for &path in wanted {
        // Builds the destination path accepting ONLY Normal components.
        // Absolute paths, drives (C:), UNC, .. and . are all rejected → missing.
        let Some(dest) = safe_join(cache_dir, path) else {
            report.missing.push(path.to_string());
            continue;
        };
        if !archive.contains(path) {
            report.missing.push(path.to_string());
            continue;
        }
        let Some(data) = archive.read(path) else {
            report.diagnostics.push(Diagnostic::DecompressFailed {
                path: path.to_string(),
            });
            continue;
        };
        if let Some(parent) = dest.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                report.diagnostics.push(Diagnostic::WriteFailed {
                    path: dest.clone(),
                    reason: e.kind(),
                });
                continue;
            }
        }
        match std::fs::write(&dest, &data) {
            Ok(()) => report.extracted.push(dest),
            Err(e) => report.diagnostics.push(Diagnostic::WriteFailed {
                path: dest,
                reason: e.kind(),
            }),
        }
    }

    report
}
