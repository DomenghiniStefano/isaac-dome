use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::room::RoomKind;

/// Which secret room a rule is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Target {
    Secret,
    SuperSecret,
    UltraSecret,
}

/// What a rule asks of a cell. `Unmodelled` is the honest variant: a sentence the wiki states
/// and this grid cannot evaluate. It is carried, never dropped, because a dropped constraint
/// reads as "nothing in the way".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Constraint {
    NeighbourCount {
        allowed: Vec<u8>,
        rank: u8,
    },
    /// A count that is switched off by a better cell. The wiki states the two secret-room
    /// counts in different shapes — 2 neighbours is possible "even when there are locations
    /// with 3+ neighbors available", 1 neighbour "can only happen if there are no valid 3+
    /// neighbor locations" — and encoding both as a plain rank would assert what the second
    /// sentence denies. `superseded_by_at_least` is the 3 that sentence states, not a band
    /// anybody named.
    NeighbourCountFallback {
        allowed: Vec<u8>,
        rank: u8,
        superseded_by_at_least: u8,
    },
    ForbiddenNeighbour {
        kinds: Vec<RoomKind>,
    },
    NeighbourNotSpecial,
    DeadEndDistanceRank {
        rank: u8,
    },
    Unmodelled {
        note: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub target: Target,
    pub quote: String,
    pub url: String,
    pub constraint: Constraint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rules {
    pub version: u32,
    pub license: String,
    pub read: String,
    rules: Vec<Rule>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulesError {
    pub message: String,
}

impl Rules {
    pub fn parse(text: &str) -> Result<Self, RulesError> {
        serde_json::from_str(text).map_err(|e| RulesError {
            message: e.to_string(),
        })
    }

    /// The file committed next to this crate, parsed once. An error here is a build-time
    /// mistake the tests catch, so it is reported rather than unwrapped.
    pub fn embedded() -> Result<&'static Rules, RulesError> {
        static ONCE: OnceLock<Result<Rules, RulesError>> = OnceLock::new();
        ONCE.get_or_init(|| Rules::parse(include_str!("../rules/placement.json")))
            .as_ref()
            .map_err(Clone::clone)
    }

    pub fn all(&self) -> impl Iterator<Item = &Rule> {
        self.rules.iter()
    }

    pub fn for_target(&self, target: Target) -> impl Iterator<Item = &Rule> {
        self.rules.iter().filter(move |r| r.target == target)
    }
}

/// The room kinds the wiki calls Special Rooms. Sourced in §4 of the rules report, from the
/// `Rooms` page's own section hierarchy — a structural citation, not a sentence about one.
/// `Start` is deliberately absent: that page files it under neither heading, so its membership
/// is unstated and the solver says so instead of choosing.
pub const SPECIAL_KINDS: &[RoomKind] = &[
    RoomKind::Boss,
    RoomKind::Treasure,
    RoomKind::Shop,
    RoomKind::Curse,
    RoomKind::Challenge,
    RoomKind::Sacrifice,
    RoomKind::Arcade,
    RoomKind::Library,
    RoomKind::Miniboss,
    RoomKind::Secret,
    RoomKind::SuperSecret,
    RoomKind::UltraSecret,
];

pub fn is_special(kind: RoomKind) -> bool {
    SPECIAL_KINDS.contains(&kind)
}
