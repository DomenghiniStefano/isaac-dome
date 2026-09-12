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
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            active_profile_id: None,
            scale: default_scale(),
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
}
