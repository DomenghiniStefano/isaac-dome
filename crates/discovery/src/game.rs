//! Game folder and edition from the appmanifest.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::edition::{dlcs_from_appids, edition_from_appids};
use crate::{Diagnostic, GameInstall, Options, SteamInstall};

const APPID: u32 = 250900;
/// Canonical install folder for appid 250900 (the same on any machine).
const CANONICAL_INSTALLDIR: &str = "The Binding of Isaac Rebirth";

pub(crate) struct Manifest {
    pub installdir: String,
    pub dlc_appids: BTreeSet<u32>,
    /// `LastUpdated` from the `AppState`: Unix epoch in seconds of the last
    /// update installed by Steam. Absent if the key is missing or is not an
    /// integer.
    pub last_updated: Option<u64>,
}

/// Parses an appmanifest ACF file and extracts `installdir`, DLC appids and `LastUpdated`.
pub(crate) fn parse_manifest(text: &str) -> Option<Manifest> {
    let partial = keyvalues_parser::Parser::new()
        .literal_special_chars(true)
        .parse(text)
        .ok()?;
    let vdf: keyvalues_parser::Vdf<'_> = partial.into();

    let app_state = vdf.value.get_obj()?;

    let installdir = app_state
        .get("installdir")
        .and_then(|vals| vals.first())
        .and_then(|v| v.get_str())?
        .to_owned();

    let mut dlc_appids = BTreeSet::new();
    if let Some(depots_vals) = app_state.get("InstalledDepots") {
        if let Some(depots_obj) = depots_vals.first().and_then(|v| v.get_obj()) {
            for depot_vals in depots_obj.values() {
                if let Some(depot_obj) = depot_vals.first().and_then(|v| v.get_obj()) {
                    if let Some(dlcappid_str) = depot_obj
                        .get("dlcappid")
                        .and_then(|vals| vals.first())
                        .and_then(|v| v.get_str())
                    {
                        if let Ok(id) = dlcappid_str.parse::<u32>() {
                            dlc_appids.insert(id);
                        }
                    }
                }
            }
        }
    }

    let last_updated = app_state
        .get("LastUpdated")
        .and_then(|vals| vals.first())
        .and_then(|v| v.get_str())
        .and_then(|s| s.parse::<u64>().ok());

    Some(Manifest {
        installdir,
        dlc_appids,
        last_updated,
    })
}

pub(crate) fn find_game(
    opts: &Options,
    steam: Option<&SteamInstall>,
) -> (Option<GameInstall>, Vec<Diagnostic>) {
    if let Some(dir) = &opts.game_dir {
        return (
            Some(game_from_dir(dir.clone(), dir.clone(), None)),
            Vec::new(),
        );
    }

    let Some(steam) = steam else {
        return (None, vec![Diagnostic::GameNotFound]);
    };

    let mut diags = Vec::new();
    for library in &steam.libraries {
        let manifest = library
            .join("steamapps")
            .join(format!("appmanifest_{APPID}.acf"));
        let text = match std::fs::read_to_string(&manifest) {
            Ok(text) => text,
            // Not there: this library does not hold the game, which is normal.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            // There and unreadable (card #80, R7): said, like `saves.rs` says a folder it
            // cannot list, and then the same as a manifest that does not parse.
            Err(e) => {
                diags.push(Diagnostic::UnreadablePath {
                    path: manifest,
                    kind: e.kind(),
                });
                if let Some(game) = canonical_game(library) {
                    return (Some(game), diags);
                }
                continue;
            }
        };
        let Some(parsed) = parse_manifest(&text) else {
            diags.push(Diagnostic::MalformedManifest { path: manifest });
            if let Some(game) = canonical_game(library) {
                return (Some(game), diags);
            }
            continue;
        };
        let dir = library
            .join("steamapps")
            .join("common")
            .join(&parsed.installdir);
        // Steam's record says installed; the disk decides. A library on a drive that
        // isn't plugged in, or an uninstall that left the manifest behind, would
        // otherwise hand back a path that doesn't exist — and the rest of the app would
        // go looking for archives inside it and report a failed extraction, when what
        // actually happened is that the game isn't there. Keep looking instead: another
        // library may hold it, and if none does, the `GameNotFound` below says so.
        // The malformed-manifest fallback above already checks the same thing.
        if !dir.is_dir() {
            continue;
        }
        return (
            Some(game_from_dir(
                dir,
                library.clone(),
                Some((manifest, parsed)),
            )),
            diags,
        );
    }

    diags.push(Diagnostic::GameNotFound);
    (None, diags)
}

/// Where the game is when the manifest says nothing — unreadable or malformed: the standard
/// installdir for appid 250900, which doesn't depend on the machine. Only if it is there.
fn canonical_game(library: &Path) -> Option<GameInstall> {
    let canonical = library
        .join("steamapps")
        .join("common")
        .join(CANONICAL_INSTALLDIR);
    canonical
        .is_dir()
        .then(|| game_from_dir(canonical, library.to_path_buf(), None))
}

fn game_from_dir(
    dir: PathBuf,
    library: PathBuf,
    parsed: Option<(PathBuf, Manifest)>,
) -> GameInstall {
    match parsed {
        Some((manifest, m)) => GameInstall {
            dir,
            library,
            manifest,
            edition: Some(edition_from_appids(&m.dlc_appids)),
            dlcs: dlcs_from_appids(&m.dlc_appids),
            updated_unix: m.last_updated,
        },
        None => GameInstall {
            manifest: PathBuf::new(),
            edition: None,
            dlcs: Vec::new(),
            updated_unix: None,
            dir,
            library,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::parse_manifest;

    #[test]
    fn manifest_reads_last_updated() {
        let m = parse_manifest("\"AppState\"\n{\n\t\"installdir\"\t\t\"The Binding of Isaac Rebirth\"\n\t\"LastUpdated\"\t\t\"1757000000\"\n}\n").unwrap();
        assert_eq!(m.last_updated, Some(1757000000));
        let m = parse_manifest("\"AppState\"\n{\n\t\"installdir\"\t\t\"X\"\n}\n").unwrap();
        assert_eq!(m.last_updated, None);
    }
}
