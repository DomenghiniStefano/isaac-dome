//! Which folders under `online_logs\` hold a run.
//!
//! **The layout was measured on 2026-09-13 and is not what a flat reading would expect:**
//!
//! ```text
//! online_logs\sessions\MM_DD_YYYY__HH_MM_SS\log.txt              26 on this machine
//! online_logs\desyncs\MM_DD_YYYY__HH_MM_SS__Name\desync_log.txt  a crash report, not a run
//! online_logs\desyncs\sessions\MM_DD_YYYY__HH_MM_SS\log.txt      2 more sessions, nested
//! ```
//!
//! So a walk of the first level finds two directories and no sessions, and a walk that takes
//! any folder under `desyncs\` files crash reports as runs. Both are wrong in a way that looks
//! like it worked.

use std::path::{Path, PathBuf};

/// Every session folder under `online_logs`, in a stable order. A folder counts only if it
/// holds a `log.txt`: the folder's own name is its identity, and the file is the run.
pub fn sessions(online_logs: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = [
        online_logs.join("sessions"),
        online_logs.join("desyncs").join("sessions"),
    ]
    .iter()
    .flat_map(|holder| session_folders(holder))
    .collect();
    found.sort();
    found
}

/// The folders directly under `holder` that hold a `log.txt`. A holder that is not there is no
/// folders and not an error: a machine that never played online has neither.
fn session_folders(holder: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(holder)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|dir| dir.join("log.txt").is_file())
        .collect()
}
