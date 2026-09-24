//! Updating the app itself: the phases a window draws, and the reading that decides how
//! often the windows are told to look again.
//!
//! The plugin, the network and the installer live in `app`. Nothing here knows that any of
//! them exist — this is a state machine and a division.
//!
//! Spec: `docs/superpowers/specs/2026-09-20-app-update-design.md`.

use serde::Serialize;
use wiki::Block;

use crate::release_notes::release_notes;

/// Why an update could not even be offered.
///
/// One variant today, and an enum rather than a `bool` for the reason the autostart design
/// recorded: a boolean makes a development build and a genuine failure the same answer with
/// two different causes. Fieldless, so a **bare camelCase string** on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum UpdateReason {
    /// No updater in this build. A development build registers no plugin on purpose — it
    /// would install a release over `target\debug` — and the platforms this compiles for but
    /// does not ship land here too.
    NotSupported,
}

/// Why an update did not happen. Five answers because they are five different things for the
/// user to do, and everything the plugin can say falls into one of them.
///
/// Fieldless, so a bare camelCase string on the wire, like [`UpdateReason`]. The moment a
/// variant gains a field the whole enum becomes tagged, TypeScript included.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum UpdateFailure {
    /// The endpoint could not be reached: no network, DNS, a proxy, GitHub down. Try later.
    Offline,
    /// Reached, and there is nothing to install: no manifest at that URL, or one that carries
    /// no entry for this platform. The ordinary answer while no release exists yet.
    NotPublished,
    /// **Reached, answered, and refused.** The signature did not verify, or it was signed for
    /// a version other than the one announced. The only failure here that is not bad luck: it
    /// means what was served is not what it claims to be, and it gets its own sentence.
    Rejected,
    /// The bytes verified and the installer would not run.
    InstallFailed,
    /// Anything else. Owed to the plugin's error being `#[non_exhaustive]`, not to laziness.
    Unknown,
}

/// Where the update stands, as a window draws it.
///
/// **There is no `Available`**: an update that is found is downloaded at once, on the
/// automatic path and on the button alike, so the phase would never rest anywhere a screen
/// could show it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum UpdatePhase {
    /// Nobody has looked yet in this launch. **Not the same as `UpToDate`**: one is "there is
    /// nothing newer", the other is "nothing was asked", and they are two sentences.
    #[default]
    Idle,
    Checking,
    /// A check answered, and this build is the newest there is.
    UpToDate,
    /// `percent` is `None` while no length has been announced: an indeterminate bar is the
    /// truth, and a bar inventing a number is not.
    Downloading {
        version: String,
        percent: Option<u8>,
    },
    /// Verified, in memory, waiting for the word.
    Ready {
        version: String,
        /// The release notes, read into blocks (`release_notes`): the manifest is not signed,
        /// so its text crosses as words and never as markup. Empty when it carried none.
        notes: Vec<Block>,
    },
    Failed {
        reason: UpdateFailure,
    },
}

/// What the `update_status` command answers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct UpdateView {
    /// What is running. The screen answers "which build is this" with no network at all, and
    /// that has to hold when everything else failed.
    pub current_version: String,
    pub phase: UpdatePhase,
    /// `None` when updating can be offered.
    pub unavailable: Option<UpdateReason>,
}

/// The phase, and the bytes counted towards the one being downloaded.
///
/// `app` holds one of these behind a mutex and calls it; every decision about what may follow
/// what is here, where it can be read and tested.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UpdateState {
    phase: UpdatePhase,
    /// Bytes of the current download. Meaningless outside `Downloading`, and reset by
    /// [`UpdateState::begin_download`] so a retry cannot start the bar halfway.
    downloaded: u64,
}

impl UpdateState {
    /// A state resting at a phase. Test-only, reachable through `for_tests`: the app only ever
    /// starts at [`UpdatePhase::Idle`] and moves through the calls below.
    #[cfg(feature = "test-api")]
    pub(crate) fn at(phase: UpdatePhase) -> Self {
        UpdateState {
            phase,
            downloaded: 0,
        }
    }

    pub fn phase(&self) -> &UpdatePhase {
        &self.phase
    }

    /// Whether something is already on its way. The button is disabled on this, and
    /// [`UpdateState::begin_check`] refuses on it.
    pub fn busy(&self) -> bool {
        matches!(
            self.phase,
            UpdatePhase::Checking | UpdatePhase::Downloading { .. }
        )
    }

    /// Starts a check, unless one is running or a download is. **`false` means "do nothing"**,
    /// never "it failed": the answer already on its way is worth more than a second question,
    /// and the bytes coming in are worth more than both.
    pub fn begin_check(&mut self) -> bool {
        if self.busy() {
            return false;
        }
        self.phase = UpdatePhase::Checking;
        true
    }

    /// A check answered, and there is nothing newer.
    pub fn up_to_date(&mut self) {
        self.phase = UpdatePhase::UpToDate;
    }

    pub fn begin_download(&mut self, version: String) {
        self.downloaded = 0;
        self.phase = UpdatePhase::Downloading {
            version,
            percent: None,
        };
    }

    /// Counts a chunk in, and answers **whether the windows are worth telling**.
    ///
    /// `true` only when the whole percentage point moved, which is what keeps a download to at
    /// most a hundred events instead of one per chunk. A chunk arriving outside a download —
    /// a callback still in flight after a failure, or after the last byte — changes nothing:
    /// a late chunk must not put the bar back on a screen that has moved on.
    pub fn advance(&mut self, chunk: usize, total: Option<u64>) -> bool {
        let UpdatePhase::Downloading { percent, .. } = &mut self.phase else {
            return false;
        };
        self.downloaded = self.downloaded.saturating_add(chunk as u64);
        // The length is the server's claim and not a measurement, so a body longer than
        // announced reads as finished rather than as 104%.
        let now = total
            .filter(|t| *t > 0)
            .map(|t| (self.downloaded.saturating_mul(100) / t).min(100) as u8);
        if now == *percent {
            return false;
        }
        *percent = now;
        true
    }

    pub fn ready(&mut self, version: String, notes: Option<String>) {
        self.phase = UpdatePhase::Ready {
            version,
            notes: notes.as_deref().map(release_notes).unwrap_or_default(),
        };
    }

    pub fn fail(&mut self, reason: UpdateFailure) {
        self.phase = UpdatePhase::Failed { reason };
    }

    pub fn view(&self, current_version: &str, unavailable: Option<UpdateReason>) -> UpdateView {
        UpdateView {
            current_version: current_version.to_string(),
            phase: self.phase.clone(),
            unavailable,
        }
    }
}
