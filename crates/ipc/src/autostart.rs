//! Starting with Windows: what the login entry is called, what it carries, and what a launch
//! carrying it should do.
//!
//! The registry is the only source of truth and it lives in `app` — `Settings` gains no field
//! and `settings.json` keeps its shape, because a mirrored boolean would read "on" for a login
//! that never happens the first time somebody uses Task Manager's Startup tab.

use serde::Serialize;

/// The name of the value under `Run`, and what Task Manager shows in its Startup list.
///
/// **It is written into the registry of everyone who ever turns the switch on.** Renaming this
/// constant does not rename what is already there: it orphans it, leaving an entry nothing in
/// the app can see, turn off, or explain.
pub const AUTOSTART_ENTRY: &str = "IsaacDome";

/// The argument the login entry carries, and the only thing that tells a login launch from a
/// double-click.
///
/// Same warning as [`AUTOSTART_ENTRY`], and worse: it is stored inside the value, so a rename
/// gives every existing user a window at every login, with no error anywhere.
pub const SILENT_ARG: &str = "--silent";

/// What a launch should do about a window. Not a wire type: `app` is the only caller, and the
/// frontend never learns how the process was started.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchIntent {
    /// Open a window, or bring one forward.
    Window,
    /// Tray and archive, no window: Windows started this, not a person.
    Silent,
}

/// What this launch means, given the arguments after the executable's own.
///
/// **Anything we do not know means `Window`**, on either side of the one we do know. The only
/// way to see this app is a window, and a launch that shows nothing because of a typo in a
/// registry value is a launch that looks like a crash.
pub fn launch_intent(args: &[String]) -> LaunchIntent {
    if !args.is_empty() && args.iter().all(|a| a == SILENT_ARG) {
        LaunchIntent::Silent
    } else {
        LaunchIntent::Window
    }
}

/// Why the switch cannot be offered.
///
/// No variant carries a field, so this is a **bare camelCase string** on the wire and the
/// TypeScript mirrors it as a union of values — the rule `CLAUDE.md` states with zero
/// exceptions. The design asked for it tagged by analogy with `SettingsReason` and
/// `StoreReason`, which are tagged because a variant of each carries data; this one has none,
/// and a tag would add a key per answer to say what the value already says.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum AutostartReason {
    /// No autostart in this build. A development build registers no plugin on purpose —
    /// `current_exe()` there is `target\debug\app.exe`, and a switch flipped once while testing
    /// leaves that path in the developer's login, surviving `cargo clean` and failing silently
    /// at every boot — and the platforms this compiles for but does not ship land here too.
    NotSupported,
    /// The manager is there and the registry would not say what it holds. The switch has no
    /// honest position, so it is not offered: showing "off" would be a guess, and turning it on
    /// would be a write over something nobody could read.
    RegistryUnreadable,
}

/// Why turning the switch did not take. Two answers because they are two different things for
/// the user to do, and the app can tell them apart: the plugin says whether the write itself
/// was refused, and the registry says whether it survived.
///
/// Fieldless, so a bare camelCase string on the wire, like [`AutostartReason`] — and a type of
/// its own rather than more variants on that one, because the two sets never meet: the view
/// answers why the switch cannot be *offered*, this answers why it would not *move*, and a
/// `switch` over either should not carry branches that cannot happen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum AutostartFailure {
    /// The write was refused outright — a policy, an antivirus, a profile that cannot write
    /// under `HKCU`. Nothing was changed.
    WriteRefused,
    /// The write was accepted and the registry still says the opposite.
    ///
    /// **Named for what was observed, not for the cause that fits best.** The Startup tab is
    /// the first place to look: the plugin's `is_enabled()` is `value && approved`, so an entry
    /// a user switched off there reads as off however well the value was written — and no
    /// write from here can turn it back on. But the two halves are not exposed separately, so
    /// calling this "turned off in Windows" would be naming a cell from a guess.
    WriteIgnored,
}

/// The switch, as the registry answers it right now.
///
/// Read when the Background screen mounts and not once at startup: the app sits in the tray for
/// days, and the Startup tab can have changed underneath it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct AutostartView {
    /// What the registry says, never what was asked for.
    pub enabled: bool,
    /// `None` when the switch can be offered. The design had a bare `available: bool` here,
    /// which made a development build and a registry that would not answer the same answer
    /// with two different causes.
    pub unavailable: Option<AutostartReason>,
}

/// What `set_autostart` answers (card #81, V1: this was decided inside the command). `wrote` is
/// whether the plugin accepted the write — `None` when there is no plugin to ask. The answer is
/// **the read-back**, never what was asked for; when the two disagree, whether the write was
/// accepted is what tells a system that undid it from a plugin that refused it.
///
/// `disable()` on a value that is not there is an error in the plugin and means nothing here:
/// the read-back says off, which is what was asked for — so a failed disable counts as accepted.
pub fn autostart_answer(
    on: bool,
    wrote: Option<bool>,
    view: AutostartView,
) -> Result<AutostartView, crate::IpcError> {
    let accepted = wrote.is_some_and(|ok| ok || !on);
    match view.unavailable {
        // The switch cannot be offered, and saying so is a better answer than a write error
        // about a registry nobody could read in the first place.
        Some(_) => Ok(view),
        None if view.enabled == on => Ok(view),
        None => Err(crate::IpcError::AutostartNotWritable {
            reason: if accepted {
                AutostartFailure::WriteIgnored
            } else {
                AutostartFailure::WriteRefused
            },
        }),
    }
}
