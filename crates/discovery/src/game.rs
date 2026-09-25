//! Game folder and edition from the appmanifest.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use keyvalues_parser::{Obj, Value};

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

    Some(Manifest {
        installdir: first_str(app_state, "installdir")?.to_owned(),
        dlc_appids: dlc_appids(app_state),
        last_updated: first_str(app_state, "LastUpdated").and_then(|s| s.parse().ok()),
    })
}

/// The first value under `key`, when it is a string. A VDF key can repeat, and the first one
/// is the one Steam reads.
fn first_str<'a>(obj: &'a Obj<'_>, key: &str) -> Option<&'a str> {
    obj.get(key)?.first()?.get_str()
}

/// The DLC appids of `InstalledDepots`: every depot that names one it can parse. A depot with
/// no `dlcappid` is the base game's, and one that doesn't parse is skipped rather than guessed.
fn dlc_appids(app_state: &Obj<'_>) -> BTreeSet<u32> {
    app_state
        .get("InstalledDepots")
        .and_then(|values| values.first())
        .and_then(Value::get_obj)
        .map(|depots| depots.values().filter_map(|d| dlc_appid(d)).collect())
        .unwrap_or_default()
}

fn dlc_appid(depot: &[Value<'_>]) -> Option<u32> {
    first_str(depot.first()?.get_obj()?, "dlcappid")?
        .parse()
        .ok()
}

pub(crate) fn find_game(
    opts: &Options,
    steam: Option<&SteamInstall>,
) -> (Option<GameInstall>, Vec<Diagnostic>) {
    if let Some(dir) = &opts.game_dir {
        return (Some(game_from_dir(dir.clone(), None)), Vec::new());
    }

    let Some(steam) = steam else {
        return (None, vec![Diagnostic::GameNotFound]);
    };

    // The libraries are asked in order, and the first that holds the game ends the search: what
    // a later library would have said is never read, so it is never reported either.
    let mut diags = Vec::new();
    for library in &steam.libraries {
        let (game, diag) = look_in(library);
        diags.extend(diag);
        if let Some(game) = game {
            return (Some(game), diags);
        }
    }
    diags.push(Diagnostic::GameNotFound);
    (None, diags)
}

/// One library: the game if it holds it, and what was wrong with its manifest if anything was.
fn look_in(library: &Path) -> (Option<GameInstall>, Option<Diagnostic>) {
    let manifest = library
        .join("steamapps")
        .join(format!("appmanifest_{APPID}.acf"));
    let text = match std::fs::read_to_string(&manifest) {
        Ok(text) => text,
        // Not there: this library does not hold the game, which is normal.
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return (None, None),
        // There and unreadable: said, like `saves.rs` says a folder it
        // cannot list, and then the same as a manifest that does not parse.
        Err(e) => {
            let diag = Diagnostic::UnreadablePath {
                path: manifest,
                kind: e.kind(),
            };
            return (canonical_game(library), Some(diag));
        }
    };
    let Some(parsed) = parse_manifest(&text) else {
        let diag = Diagnostic::MalformedManifest { path: manifest };
        return (canonical_game(library), Some(diag));
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
    // library may hold it, and if none does, the `GameNotFound` the caller adds says so.
    // The malformed-manifest fallback above already checks the same thing.
    let game = dir
        .is_dir()
        .then(|| game_from_dir(dir, Some((manifest, parsed))));
    (game, None)
}

/// Where the game is when the manifest says nothing — unreadable or malformed: the standard
/// installdir for appid 250900, which doesn't depend on the machine. Only if it is there.
fn canonical_game(library: &Path) -> Option<GameInstall> {
    let canonical = library
        .join("steamapps")
        .join("common")
        .join(CANONICAL_INSTALLDIR);
    canonical.is_dir().then(|| game_from_dir(canonical, None))
}

fn game_from_dir(dir: PathBuf, parsed: Option<(PathBuf, Manifest)>) -> GameInstall {
    match parsed {
        Some((manifest, m)) => GameInstall {
            dir,
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
