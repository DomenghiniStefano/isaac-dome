use serde::Serialize;

use crate::{AutostartFailure, SaveReason, SettingsReason, StoreReason};

/// Error that crosses the IPC boundary. Tagged, not a string: the UI must be able to
/// tell "no active profile" apart from "unreadable file" without parsing text.
#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum IpcError {
    /// Not an error to display: the UI goes to the selection screen.
    NoActiveProfile,
    UnknownProfile {
        id: String,
    },
    UnreadableSave {
        reason: SaveReason,
    },
    SettingsNotWritable {
        reason: SettingsReason,
    },
    /// A goal's target doesn't exist in the catalog: it isn't saved.
    UnknownTarget,
    /// The catalog isn't there: a target can't be verified, and without verification
    /// nothing gets saved. Different from `UnknownTarget`: here the id could actually be
    /// valid. Today this only fires when the game is absent, because `ResourceSet::open`
    /// and `Catalog::build` never fail: an unreadable `packed` folder yields an empty
    /// catalog, hence `UnknownTarget`.
    CatalogUnavailable,
    /// The app's database won't open: goals can neither be read nor written.
    StoreUnavailable {
        reason: StoreReason,
    },
    /// The embedded dataset failed to load: `wiki_entry` can't answer;
    /// `ExtractionReport.wiki` says why.
    WikiUnavailable,
    /// The session document offered is past `MAX_SESSION_BYTES`. Nothing the user did: a
    /// frontend bug, reported rather than truncated, because half a document is not a session.
    SessionTooLarge,
    /// The switch would not move. The reason is the two cases the app can actually tell apart,
    /// and they are two different things for the user to do: a write nothing accepted, and a
    /// write that went in while the registry kept saying the opposite.
    AutostartNotWritable {
        reason: AutostartFailure,
    },
    /// Install was asked for while there are no verified bytes to install.
    ///
    /// **The one case here that is a frontend defect and not a thing that happened to the
    /// user**: the button only exists in `Ready`, so reaching this means a window acted on a
    /// phase it no longer had. It is an `Err` for that reason — everything the user can
    /// actually run into, an endpoint that will not answer included, travels in the payload as
    /// a phase.
    UpdateNotReady,
}
