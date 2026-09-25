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

/// What a scan found and what got in its way, the shape every scanner here answers with.
pub(crate) type Scan = (Vec<SaveCandidate>, Vec<Diagnostic>);

/// Several scans as one, in the order they were given: the saves one after the other, and the
/// diagnostics the same way.
pub(crate) fn merge(scans: impl IntoIterator<Item = Scan>) -> Scan {
    let (saves, diags): (Vec<_>, Vec<_>) = scans.into_iter().unzip();
    (
        saves.into_iter().flatten().collect(),
        diags.into_iter().flatten().collect(),
    )
}

/// A folder that could not be listed. Not there is not a diagnostic: a machine without that
/// folder is the ordinary case, and only a folder that is there and refuses is worth saying.
fn unreadable(path: &Path, err: &std::io::Error) -> Vec<Diagnostic> {
    (err.kind() != std::io::ErrorKind::NotFound)
        .then(|| Diagnostic::UnreadablePath {
            path: path.to_path_buf(),
            kind: err.kind(),
        })
        .into_iter()
        .collect()
}

/// Scans a single folder for valid save files.
pub(crate) fn scan_dir(dir: &Path, source: &dyn Fn() -> SaveSource) -> Scan {
    match std::fs::read_dir(dir) {
        Ok(entries) => (
            entries
                .flatten()
                .filter_map(|entry| candidate(&entry, source))
                .collect(),
            Vec::new(),
        ),
        Err(err) => (Vec::new(), unreadable(dir, &err)),
    }
}

/// The entry as a save, when its name is one.
fn candidate(entry: &std::fs::DirEntry, source: &dyn Fn() -> SaveSource) -> Option<SaveCandidate> {
    let name = entry.file_name();
    let (prefix, slot) = parse_save_filename(name.to_str()?)?;
    let meta = entry.metadata().ok();
    Some(SaveCandidate {
        path: entry.path(),
        slot,
        prefix,
        source: source(),
        modified: meta.as_ref().and_then(|m| m.modified().ok()),
        size: meta.as_ref().map(|m| m.len()).unwrap_or(0),
    })
}

/// For each account in `<steam_root>/userdata/*`, scans `250900/remote/`.
pub(crate) fn scan_userdata(steam_root: &Path) -> Scan {
    let userdata = steam_root.join("userdata");
    let entries = match std::fs::read_dir(&userdata) {
        Ok(entries) => entries,
        Err(err) => return (Vec::new(), unreadable(&userdata, &err)),
    };
    merge(
        entries
            .flatten()
            .filter(|entry| entry.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .map(|entry| {
                let account_id = entry.file_name().to_string_lossy().into_owned();
                let remote = entry.path().join("250900").join("remote");
                scan_dir(&remote, &|| SaveSource::SteamCloud {
                    account_id: account_id.clone(),
                })
            }),
    )
}

/// Scans the two possible folders in Documents (with and without the `+`).
pub(crate) fn scan_documents(documents: &Path) -> Scan {
    merge(
        [
            "Binding of Isaac Repentance+",
            "Binding of Isaac Repentance",
        ]
        .into_iter()
        .map(|folder| {
            let dir = documents.join("My Games").join(folder);
            scan_dir(&dir, &|| SaveSource::Documents {
                folder: dir.clone(),
            })
        }),
    )
}
