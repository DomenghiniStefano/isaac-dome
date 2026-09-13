use serde::Serialize;

use crate::{SaveReason, SettingsReason, StoreReason};

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
}
