//! Filename parsing and scanning of the save folders.

use std::path::Path;

use crate::{Diagnostic, SaveCandidate, SavePrefix, SaveSource};

/// `rep+persistentgamedata1.dat` → `(RepPlus, 1)`; `rep_persistentgamedata2.dat` → `(Rep, 2)`.
/// Requires the exact prefix, `persistentgamedata`, a single slot digit 1..=9, then `.dat`.
/// Dated backups (`2025….dat`) and any other name don't match.
pub(crate) fn parse_save_filename(name: &str) -> Option<(SavePrefix, u8)> {
    let stem = name.strip_suffix(".dat")?;
    let (prefix, rest) = if let Some(rest) = stem.strip_prefix("rep+") {
        (SavePrefix::RepPlus, rest)
    } else if let Some(rest) = stem.strip_prefix("rep_") {
        (SavePrefix::Rep, rest)
    } else {
        return None;
    };
    let digits = rest.strip_prefix("persistentgamedata")?;
    if digits.len() != 1 {
        return None;
    }
    let slot: u8 = digits.parse().ok()?;
    if slot == 0 {
        return None;
    }
    Some((prefix, slot))
}

/// Scans a single folder for valid save files.
pub(crate) fn scan_dir(
    dir: &Path,
    source: &dyn Fn() -> SaveSource,
) -> (Vec<SaveCandidate>, Vec<Diagnostic>) {
    let mut saves = Vec::new();
    let mut diags = Vec::new();

    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) => {
            if err.kind() != std::io::ErrorKind::NotFound {
                diags.push(Diagnostic::UnreadablePath {
                    path: dir.to_path_buf(),
                    kind: err.kind(),
                });
            }
            return (saves, diags);
        }
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        let Some((prefix, slot)) = parse_save_filename(name) else {
            continue;
        };
        let meta = entry.metadata().ok();
        saves.push(SaveCandidate {
            path: entry.path(),
            slot,
            prefix,
            source: source(),
            modified: meta.as_ref().and_then(|m| m.modified().ok()),
            size: meta.as_ref().map(|m| m.len()).unwrap_or(0),
        });
    }

    (saves, diags)
}

/// For each account in `<steam_root>/userdata/*`, scans `250900/remote/`.
pub(crate) fn scan_userdata(steam_root: &Path) -> (Vec<SaveCandidate>, Vec<Diagnostic>) {
    let userdata = steam_root.join("userdata");
    let mut saves = Vec::new();
    let mut diags = Vec::new();

    let entries = match std::fs::read_dir(&userdata) {
        Ok(entries) => entries,
        Err(err) => {
            if err.kind() != std::io::ErrorKind::NotFound {
                diags.push(Diagnostic::UnreadablePath {
                    path: userdata,
                    kind: err.kind(),
                });
            }
            return (saves, diags);
        }
    };

    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let account_id = entry.file_name().to_string_lossy().into_owned();
        let remote = entry.path().join("250900").join("remote");
        let (mut c, mut d) = scan_dir(&remote, &|| SaveSource::SteamCloud {
            account_id: account_id.clone(),
        });
        saves.append(&mut c);
        diags.append(&mut d);
    }

    (saves, diags)
}

/// Scans the two possible folders in Documents (with and without the `+`).
pub(crate) fn scan_documents(documents: &Path) -> (Vec<SaveCandidate>, Vec<Diagnostic>) {
    let mut saves = Vec::new();
    let mut diags = Vec::new();

    for folder in [
        "Binding of Isaac Repentance+",
        "Binding of Isaac Repentance",
    ] {
        let dir = documents.join("My Games").join(folder);
        let dir_for_source = dir.clone();
        let (mut c, mut d) = scan_dir(&dir, &|| SaveSource::Documents {
            folder: dir_for_source.clone(),
        });
        saves.append(&mut c);
        diags.append(&mut d);
    }

    (saves, diags)
}
