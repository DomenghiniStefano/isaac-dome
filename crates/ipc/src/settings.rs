use serde::{Deserialize, Serialize};

use crate::ProfileId;

/// The sizes the interface is drawn at, as percentages: **Discord's zoom levels**, which are
/// Chromium's own ladder (`docs/BACKLOG.md` B26). Eleven, and no twelfth — every one of them
/// was looked at, and a size in between was not.
pub const SCALE_PERCENTS: [u16; 11] = [50, 67, 75, 80, 90, 100, 110, 125, 150, 175, 200];

/// The size everything is drawn at unless the user says otherwise: the one the design file
/// is measured in.
pub const DEFAULT_SCALE: u16 = 100;

/// A percentage read from outside, made one of ours. **Not the nearest step**: a number
/// that isn't on the ladder comes from a file we didn't write, and reading it as "about
/// 125" would put the app at a size nothing was drawn at. It reads as the default instead,
/// and the next change writes a value we know.
pub fn snap_percent(percent: u16) -> u16 {
    if SCALE_PERCENTS.contains(&percent) {
        percent
    } else {
        DEFAULT_SCALE
    }
}

fn default_scale() -> u16 {
    DEFAULT_SCALE
}

/// Persisted settings. The type and its serialization live here;
/// reading and writing the file live in the app crate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub active_profile_id: Option<ProfileId>,
    /// The interface's size as a percentage. Public because it has to serialize, but read
    /// through `scale()`: the field is whatever the file said.
    pub scale: u16,
    /// Closing the last window leaves the app in the notification area instead of ending it.
    /// **On by default**: it is the behaviour, and the switch exists for whoever doesn't want
    /// an app in their tray.
    pub stay_in_background: bool,
    /// A window born with nothing owed to it opens on the tabs of the last session.
    pub resume_tabs: bool,
    /// Whether the one-time "it's still running" notice has been shown. Written by the app,
    /// never shown to the user.
    pub background_notice_shown: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            active_profile_id: None,
            scale: default_scale(),
            stay_in_background: true,
            resume_tabs: true,
            background_notice_shown: false,
        }
    }
}

impl Settings {
    /// The size to draw at: the field, snapped to the ladder. Every caller goes through
    /// here, so a hand-edited file can't put the app at 137%.
    pub fn scale(&self) -> u16 {
        snap_percent(self.scale)
    }

    /// The same settings at another size, snapped. The other fields are kept: switching
    /// size must not forget which profile is active.
    pub fn with_scale(&self, percent: u16) -> Settings {
        Settings {
            scale: snap_percent(percent),
            ..self.clone()
        }
    }

    /// The same settings, staying in the background or not. Same reason as `with_scale`: a
    /// write must never forget the other fields.
    pub fn with_stay_in_background(&self, stay: bool) -> Settings {
        Settings {
            stay_in_background: stay,
            ..self.clone()
        }
    }

    pub fn with_resume_tabs(&self, resume: bool) -> Settings {
        Settings {
            resume_tabs: resume,
            ..self.clone()
        }
    }

    pub fn with_notice_shown(&self, shown: bool) -> Settings {
        Settings {
            background_notice_shown: shown,
            ..self.clone()
        }
    }
}

/// The largest session document the app will store. Nothing should approach it — fifty tabs
/// of route names and queries are a few kilobytes — and a database that grows without a bound
/// is how you find out that something did.
pub const MAX_SESSION_BYTES: usize = 64 * 1024;

/// Whether a document offered by the frontend is small enough to keep. Refused, never
/// truncated: half a JSON document is not a session.
pub fn session_document_fits(document: &str) -> bool {
    document.len() <= MAX_SESSION_BYTES
}

#[cfg(test)]
mod tests {
    use super::*;

    // A settings file written before this feature has none of the three keys. It must read,
    // and read as "background on": the default is the behaviour, not the opt-in.
    #[test]
    fn a_settings_file_from_before_this_feature_reads_with_the_background_on() {
        let old = r#"{"activeProfileId":null,"scale":110}"#;
        let s: Settings = serde_json::from_str(old).expect("old settings still read");
        assert_eq!(s.scale, 110);
        assert!(s.stay_in_background);
        assert!(s.resume_tabs);
        assert!(!s.background_notice_shown);
    }

    #[test]
    fn the_three_flags_serialize_as_camel_case() {
        let json = serde_json::to_string(&Settings::default()).expect("serializes");
        assert!(json.contains("\"stayInBackground\""), "{json}");
        assert!(json.contains("\"resumeTabs\""), "{json}");
        assert!(json.contains("\"backgroundNoticeShown\""), "{json}");
    }

    // The same reason `with_scale` exists: a write must never forget the other fields.
    #[test]
    fn changing_one_flag_keeps_every_other_field() {
        let s = Settings {
            scale: 125,
            resume_tabs: false,
            ..Settings::default()
        }
        .with_stay_in_background(false);
        assert_eq!(s.scale, 125);
        assert!(!s.resume_tabs);
        assert!(!s.stay_in_background);
    }

    #[test]
    fn a_session_document_is_refused_past_the_cap_and_accepted_at_it() {
        assert!(session_document_fits(&"a".repeat(MAX_SESSION_BYTES)));
        assert!(!session_document_fits(&"a".repeat(MAX_SESSION_BYTES + 1)));
    }
}
