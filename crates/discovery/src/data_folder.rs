//! The game's own folder under Documents: `log.txt`, `online_logs\`, `save_backups\`.
//!
//! This is not where the saves are — with Steam Cloud on they are under `userdata\` — and that
//! is exactly why it needs finding on its own. `saves::scan_documents` walks the same two
//! folders looking for `.dat` files and reports the folder only when it finds one, so on the
//! ordinary machine nothing named the place the log lives.

use std::path::{Path, PathBuf};

use crate::GameDataFolder;

/// The two spellings, newest first: a machine that upgraded has both, and the `+` one is the
/// one the game writes to now. **A guess, and the only one there was until B57**: which of the
/// two exists depends on a history the app cannot see.
const FOLDERS: [&str; 2] = [
    "Binding of Isaac Repentance+",
    "Binding of Isaac Repentance",
];

/// The file the game writes in its own install folder, rewritten on every launch.
const DECLARATION: &str = "savedatapath.txt";

/// The label that tells the save folder from the modding one. Both lines end in an absolute
/// path, and this is the only thing separating them.
const SAVE_LABEL: &str = "Save Data Path:";

/// Where the game says it saves, read from its install. `None` when the file is not there, holds
/// no such line, or names nothing — each of them ordinary: the file exists only where the game
/// was found, and it is informational text rather than a contract.
pub(crate) fn declared_game_data(game_dir: &Path) -> Option<PathBuf> {
    parse_save_data_path(&std::fs::read_to_string(game_dir.join(DECLARATION)).ok()?)
}

/// The folder named on the `Save Data Path:` line.
///
/// The separators are **mixed** — `C:\Users\stefa/Documents/My Games/…` — on both machines this
/// has been read on, two different installs and two different Steam library roots, which is what
/// makes it something a parser may rely on rather than one machine's accident. `Path` on Windows
/// takes both. The trailing one the game writes is dropped: kept, it makes an empty last
/// component, and every question asked about the folder's name then answers about nothing.
pub(crate) fn parse_save_data_path(contents: &str) -> Option<PathBuf> {
    let value = contents
        .lines()
        .find_map(|line| line.trim_start().strip_prefix(SAVE_LABEL))?
        .trim()
        .trim_end_matches(['/', '\\']);
    (!value.is_empty()).then(|| PathBuf::from(value))
}

/// The folder, if one is there. `declared` is the game's own answer and is tried first — it is
/// the one thing that cannot be wrong about the spelling — but it is a **candidate, not the
/// answer**: it is informational, it exists only where the game was found, and a stranger's
/// machine may have neither. A declared folder that is not on disk falls through to the search,
/// which is what every machine did before B57.
///
/// Each of the three things inside is reported only if it exists: a fresh install has none of
/// them and that is not a failure.
pub(crate) fn scan_game_data(documents: &Path, declared: Option<&Path>) -> Option<GameDataFolder> {
    let searched = FOLDERS
        .iter()
        .map(|folder| documents.join("My Games").join(folder));
    declared
        .map(Path::to_path_buf)
        .into_iter()
        .chain(searched)
        .find(|dir| dir.is_dir())
        .map(|dir| GameDataFolder {
            log: existing_file(&dir, "log.txt"),
            online_logs: existing_dir(&dir, "online_logs"),
            save_backups: existing_dir(&dir, "save_backups"),
            dir,
        })
}

fn existing_file(dir: &Path, name: &str) -> Option<PathBuf> {
    let p = dir.join(name);
    p.is_file().then_some(p)
}

fn existing_dir(dir: &Path, name: &str) -> Option<PathBuf> {
    let p = dir.join(name);
    p.is_dir().then_some(p)
}
