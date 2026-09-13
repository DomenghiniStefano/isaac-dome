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
    let mut out = Vec::new();
    for holder in [
        online_logs.join("sessions"),
        online_logs.join("desyncs").join("sessions"),
    ] {
        let Ok(entries) = std::fs::read_dir(&holder) else {
            // Not there is not an error: a machine that never played online has neither folder.
            continue;
        };
        for entry in entries.flatten() {
            let dir = entry.path();
            if dir.join("log.txt").is_file() {
                out.push(dir);
            }
        }
    }
    out.sort();
    out
}
