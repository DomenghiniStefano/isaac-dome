//! discovery — locates Steam, the Isaac game (250900) and the saves, on any PC.

use std::path::PathBuf;
use std::time::SystemTime;

use serde::Serialize;

mod data_folder;
mod edition;
mod game;
mod saves;
mod steam;

/// Overrides supplied by the caller/UI for manual fallback. All optional.
#[derive(Debug, Default, Clone)]
pub struct Options {
    pub steam_root: Option<PathBuf>,
    pub game_dir: Option<PathBuf>,
    pub save_dir: Option<PathBuf>,
}

/// Not `Serialize`, and deliberately: it holds a `PathBuf`, which the IPC boundary forbids —
/// a path under `userdata\` carries the Steam account id, and always the Windows username.
/// `ipc::SetupState` is what crosses, and it carries only a path's last component.
#[derive(Debug, Clone)]
pub struct Discovery {
    pub steam: Option<SteamInstall>,
    pub game: Option<GameInstall>,
    pub saves: Vec<SaveCandidate>,
    /// Where the game writes its logs, when Documents has the folder at all.
    pub game_data: Option<GameDataFolder>,
    pub diagnostics: Vec<Diagnostic>,
}

/// The game's folder under Documents — where `log.txt` and `online_logs\` live, which is **not**
/// where the saves live when Steam Cloud is on.
///
/// Not `Serialize`, for `Discovery`'s own reason: these are paths, and a path under Documents
/// carries the Windows username. What crosses the IPC about the logs is the runs view, which
/// names a session by its folder's name and never by its path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameDataFolder {
    pub dir: PathBuf,
    /// The log the game rewrites on every launch.
    pub log: Option<PathBuf>,
    /// One folder per online session. Not flat: see `log_watch::sessions`.
    pub online_logs: Option<PathBuf>,
    /// The game's own dated backups. Not read by this sub-project; named here because this is
    /// the one place that knows where it is.
    pub save_backups: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SteamInstall {
    pub root: PathBuf,
    pub source: SteamSource,
    pub libraries: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SteamSource {
    SteamLocate,
    Registry,
    Override,
}

#[derive(Debug, Clone, Serialize)]
pub struct GameInstall {
    pub dir: PathBuf,
    pub library: PathBuf,
    pub manifest: PathBuf,
    pub edition: Edition,
    pub dlcs: Vec<Dlc>,
    /// Unix epoch (seconds) of the last update installed, from the
    /// appmanifest's `LastUpdated`. `None` without a manifest or without the key.
    pub updated_unix: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum Edition {
    Rebirth,
    Afterbirth,
    AfterbirthPlus,
    Repentance,
    RepentancePlus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[ts(rename = "InstalledDlc")]
#[serde(rename_all = "snake_case")]
pub enum Dlc {
    Afterbirth,
    AfterbirthPlus,
    Repentance,
    RepentancePlus,
}

#[derive(Debug, Clone, Serialize)]
pub struct SaveCandidate {
    pub path: PathBuf,
    pub slot: u8,
    pub source: SaveSource,
    pub prefix: SavePrefix,
    pub modified: Option<SystemTime>,
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum SaveSource {
    SteamCloud { account_id: String },
    Documents { folder: PathBuf },
    Override,
}

/// The two save-file name prefixes, `rep_` and `rep+`. A domain enum: it keeps its own
/// `snake_case` on the wire, unlike the ones the boundary defines for itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum SavePrefix {
    Rep,
    RepPlus,
}

/// Same as `Discovery`: not `Serialize`, because `UnreadablePath` and `MalformedManifest`
/// name a full path and a path never crosses the boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Diagnostic {
    SteamNotFound,
    GameNotFound,
    NoSavesFound,
    /// The `io::ErrorKind` and not the message: the message is written by the OS, is not
    /// translatable, and on some platforms repeats the path it was given.
    UnreadablePath {
        path: PathBuf,
        kind: std::io::ErrorKind,
    },
    MalformedManifest {
        path: PathBuf,
    },
}

/// Entry point. Enumerates everything it finds; never chooses; never returns `Err`.
pub fn discover(opts: &Options) -> Discovery {
    let mut diagnostics = Vec::new();

    let (steam, mut steam_diags) = steam::find_steam(opts);
    diagnostics.append(&mut steam_diags);

    let (game, mut game_diags) = game::find_game(opts, steam.as_ref());
    diagnostics.append(&mut game_diags);

    let mut saves = Vec::new();

    if let Some(dir) = &opts.save_dir {
        let (mut c, mut d) = saves::scan_dir(dir, &|| SaveSource::Override);
        saves.append(&mut c);
        diagnostics.append(&mut d);
    }
    if let Some(steam) = &steam {
        let (mut c, mut d) = saves::scan_userdata(&steam.root);
        saves.append(&mut c);
        diagnostics.append(&mut d);
    }
    // The data folder is probed **whether or not** a save was found there: with Steam Cloud on
    // there never is one, and that is the ordinary machine.
    let mut game_data = None;
    if let Some(documents) = dirs::document_dir() {
        let (mut c, mut d) = saves::scan_documents(&documents);
        saves.append(&mut c);
        diagnostics.append(&mut d);
        game_data = data_folder::scan_game_data(&documents);
    }

    if saves.is_empty() {
        diagnostics.push(Diagnostic::NoSavesFound);
    }

    Discovery {
        steam,
        game,
        saves,
        game_data,
        diagnostics,
    }
}

/// Entry points that exist only so tests can reach a shape the public API doesn't build.
///
/// One module per crate, and nothing test-only anywhere else in the public surface: a name
/// in the crate's `pub use` list says "call me", which is the opposite of what these mean.
/// Nothing outside a `tests/` target may call them.
pub mod for_tests {
    use std::collections::BTreeSet;

    use crate::{Dlc, Edition};

    pub fn scan_game_data(documents: &std::path::Path) -> Option<crate::GameDataFolder> {
        crate::data_folder::scan_game_data(documents)
    }

    pub fn edition_from_appids(appids: &BTreeSet<u32>) -> Edition {
        crate::edition::edition_from_appids(appids)
    }
    pub fn dlcs_from_appids(appids: &BTreeSet<u32>) -> Vec<Dlc> {
        crate::edition::dlcs_from_appids(appids)
    }

    use crate::SavePrefix;

    pub fn parse_save_filename(name: &str) -> Option<(SavePrefix, u8)> {
        crate::saves::parse_save_filename(name)
    }

    pub fn parse_manifest_fields(text: &str) -> Option<(String, BTreeSet<u32>)> {
        crate::game::parse_manifest(text).map(|m| (m.installdir, m.dlc_appids))
    }

    use crate::{Diagnostic, GameInstall, SaveCandidate, SaveSource, SteamInstall};
    use std::path::Path;

    pub fn scan_userdata(steam_root: &Path) -> (Vec<SaveCandidate>, Vec<Diagnostic>) {
        crate::saves::scan_userdata(steam_root)
    }
    pub fn scan_documents(documents: &Path) -> (Vec<SaveCandidate>, Vec<Diagnostic>) {
        crate::saves::scan_documents(documents)
    }
    pub fn scan_override(dir: &Path) -> (Vec<SaveCandidate>, Vec<Diagnostic>) {
        crate::saves::scan_dir(dir, &|| SaveSource::Override)
    }

    pub fn find_steam(opts: &crate::Options) -> (Option<SteamInstall>, Vec<Diagnostic>) {
        crate::steam::find_steam(opts)
    }
    pub fn find_game(
        opts: &crate::Options,
        steam: Option<&SteamInstall>,
    ) -> (Option<GameInstall>, Vec<Diagnostic>) {
        crate::game::find_game(opts, steam)
    }
}
