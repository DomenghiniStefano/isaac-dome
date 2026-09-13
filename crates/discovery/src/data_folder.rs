//! The game's own folder under Documents: `log.txt`, `online_logs\`, `save_backups\`.
//!
//! This is not where the saves are — with Steam Cloud on they are under `userdata\` — and that
//! is exactly why it needs finding on its own. `saves::scan_documents` walks the same two
//! folders looking for `.dat` files and reports the folder only when it finds one, so on the
//! ordinary machine nothing named the place the log lives.

use std::path::{Path, PathBuf};

use crate::GameDataFolder;

/// The two spellings, newest first: a machine that upgraded has both, and the `+` one is the
/// one the game writes to now.
const FOLDERS: [&str; 2] = [
    "Binding of Isaac Repentance+",
    "Binding of Isaac Repentance",
];

/// The folder, if one of the two is there. Each of the three things inside is reported only if
/// it exists: a fresh install has none of them and that is not a failure.
pub(crate) fn scan_game_data(documents: &Path) -> Option<GameDataFolder> {
    for folder in FOLDERS {
        let dir = documents.join("My Games").join(folder);
        if !dir.is_dir() {
            continue;
        }
        return Some(GameDataFolder {
            log: existing_file(&dir, "log.txt"),
            online_logs: existing_dir(&dir, "online_logs"),
            save_backups: existing_dir(&dir, "save_backups"),
            dir,
        });
    }
    None
}

fn existing_file(dir: &Path, name: &str) -> Option<PathBuf> {
    let p = dir.join(name);
    p.is_file().then_some(p)
}

fn existing_dir(dir: &Path, name: &str) -> Option<PathBuf> {
    let p = dir.join(name);
    p.is_dir().then_some(p)
}
