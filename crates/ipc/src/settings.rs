use serde::{Deserialize, Serialize};

use crate::ProfileId;

/// Persisted settings. The type and its serialization live here;
/// reading and writing the file live in the app crate.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub active_profile_id: Option<ProfileId>,
}
