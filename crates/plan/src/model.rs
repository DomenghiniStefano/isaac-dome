//! The queue and its rows. The order **is** the position in the array: there is no
//! sequence column, and therefore no way to write an order that contradicts itself.

use graph::AchievementId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    pub achievement: AchievementId,
    /// You asked for this one, for itself.
    pub wanted: bool,
    /// Every wanted achievement whose chain passes through this row. A list, not a single
    /// value: two wishes can need the same step, and with one slot the second would be
    /// lost — visibly, on removal, when a step another wish still needs looks orphaned.
    pub origins: Vec<AchievementId>,
}

impl Row {
    /// A row with neither of the two reasons to exist is an orphan, and it goes.
    pub fn is_orphan(&self) -> bool {
        !self.wanted && self.origins.is_empty()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Queue(Vec<Row>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueueError {
    Unreadable { reason: String },
}

impl std::fmt::Display for QueueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueueError::Unreadable { reason } => write!(f, "unreadable queue: {reason}"),
        }
    }
}

impl std::error::Error for QueueError {}

impl Queue {
    pub fn from_rows(rows: Vec<Row>) -> Queue {
        Queue(rows)
    }

    pub fn rows(&self) -> &[Row] {
        &self.0
    }

    pub fn position(&self, achievement: AchievementId) -> Option<usize> {
        self.0.iter().position(|r| r.achievement == achievement)
    }

    pub fn to_json(&self) -> String {
        // A `Vec<Row>` of plain integers and bools: serialization cannot fail. The
        // fallback is an empty document rather than an `unwrap`, and it is unreachable.
        serde_json::to_string(self).unwrap_or_else(|_| "[]".to_string())
    }

    pub fn from_json(s: &str) -> Result<Queue, QueueError> {
        serde_json::from_str(s).map_err(|e| QueueError::Unreadable {
            reason: e.to_string(),
        })
    }
}
