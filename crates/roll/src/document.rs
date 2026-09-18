use serde::{Deserialize, Serialize};

use crate::deck::Preset;
use crate::target::Target;

/// The shape this binary writes. Bumped when a field changes meaning, never when one is added
/// with a default.
pub const DOCUMENT_VERSION: u32 = 1;

/// The draw as it was made. `deck_size` is stored and not recomputed: the deck at that moment
/// is gone once the save moves, and "drawn from 137" is a fact about then.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Drawn {
    pub target: Target,
    pub deck_size: usize,
    pub drawn_unix: i64,
}

/// One preset, always saved, with the current draw beside it. One document and not a table: a
/// second row would be a second answer to "what is the preset".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub version: u32,
    pub preset: Preset,
    pub current: Option<Drawn>,
}

impl Default for Document {
    fn default() -> Document {
        Document {
            version: DOCUMENT_VERSION,
            preset: Preset::default(),
            current: None,
        }
    }
}

/// Two different sentences on the screen, which is why they are two variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentError {
    Unreadable { reason: String },
    FromTheFuture { version: u32, supported: u32 },
}

impl std::fmt::Display for DocumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentError::Unreadable { reason } => {
                write!(f, "unreadable roll document: {reason}")
            }
            DocumentError::FromTheFuture { version, supported } => write!(
                f,
                "roll document version {version}, this binary reads {supported}"
            ),
        }
    }
}

impl std::error::Error for DocumentError {}

impl Document {
    pub fn to_json(&self) -> String {
        // Integers, bools and two small enums: serialization cannot fail. The fallback is the
        // default document rather than an `unwrap`, and it is unreachable.
        serde_json::to_string(self)
            .unwrap_or_else(|_| serde_json::to_string(&Document::default()).unwrap_or_default())
    }

    /// A document newer than us is refused rather than read through: fields we do not know are
    /// fields we would drop on the next write.
    pub fn from_json(s: &str) -> Result<Document, DocumentError> {
        let doc: Document = serde_json::from_str(s).map_err(|e| DocumentError::Unreadable {
            reason: e.to_string(),
        })?;
        if doc.version > DOCUMENT_VERSION {
            return Err(DocumentError::FromTheFuture {
                version: doc.version,
                supported: DOCUMENT_VERSION,
            });
        }
        Ok(doc)
    }
}
