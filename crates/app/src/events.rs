//! What the commands that write tell the other windows.
//!
//! With more than one window open, a window only ever learns of a write it made itself. These
//! events say "read again", and **carry no payload**: a payload would be a copy of state the
//! next command could contradict, and a second wire shape to keep in `camelCase` for nothing.
//! Each window answers by calling the commands it already calls.
//!
//! The three names are mirrored by hand in `ui/src/lib/window/appEvents.ts`, as the IPC types
//! are: change one and change the other.

use tauri::{AppHandle, Emitter};

/// The active profile changed: every window's indicator, gate and progress screens follow.
pub const PROFILE_CHANGED: &str = "profile-changed";

/// The settings changed — today that means the interface's size.
pub const SETTINGS_CHANGED: &str = "settings-changed";

/// The plan changed: the queue or the goals, which are one screen.
pub const PLAN_CHANGED: &str = "plan-changed";

/// Tells every window. A window closing at this instant makes the emit fail, and a failed
/// notification must never turn a successful write into an error.
pub fn announce(app: &AppHandle, event: &str) {
    let _ = app.emit(event, ());
}

/// The run archive changed: a run was imported, or the one being played moved on.
pub const RUNS_CHANGED: &str = "runs-changed";
