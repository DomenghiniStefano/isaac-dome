//! The two rules files and their join. `requirements.json` is generated from the wiki
//! snapshot and holds no id of ours; `corrections.json` is written by hand. Nothing here
//! knows about the user's catalog: resolution happens later, in `graph::build`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use wiki::Target;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Requirements {
    pub schema_version: u32,
    pub generated_from: GeneratedFrom,
    pub achievements: BTreeMap<u32, AchievementRefs>,
    /// Every target that doesn't reduce to an achievement on its own: the form the
    /// curation fills in, not a list anyone has to invent.
    pub targets: Vec<TargetRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedFrom {
    pub snapshot_at: String,
    pub max_revid: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AchievementRefs {
    pub refs: Vec<RefRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefRow {
    pub target: Target,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetRow {
    pub key: String,
    pub label: String,
    pub uses: u32,
    /// Whether a verdict is always consulted for this target, and therefore has to exist.
    ///
    /// It is false for entities only. An entity ref usually resolves to a boss by name —
    /// and then no verdict is read — but whether it resolves depends on the catalog the
    /// user has, which this file deliberately knows nothing about. So the file-level check
    /// covers the kinds that can never resolve (stage, room, pickup, transformation), and
    /// the stronger check — nothing left unknown against the real catalog — belongs to the
    /// tests that have the game. An entity that neither resolves nor has a verdict still
    /// ends up `Unknown`: the honesty chain doesn't depend on this flag.
    pub verdict_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Corrections {
    pub schema_version: u32,
    #[serde(default)]
    pub aliases: BTreeMap<String, String>,
    #[serde(default)]
    pub verdicts: BTreeMap<String, Verdict>,
}

/// Exactly three verdicts, and no fourth. A target with no verdict is not "no
/// prerequisite": it stays unknown, and the node that carries it drops to `Partial`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Verdict {
    /// Fought on night one: Satan, Mom, Isaac.
    AlwaysAvailable(bool),
    /// Sits behind an achievement of the graph.
    Behind { achievement: u32 },
    /// Not a prerequisite at all: a pickup that appears in the sentence.
    NotAPrerequisite(bool),
    /// Judged, and the answer is that the model can't say it: gated, but by something
    /// other than one achievement — three Guppy items, "a tainted character", "all
    /// endings". At runtime it behaves exactly like no verdict (the node drops to
    /// `Partial`), and it exists to separate *judged and inexpressible* from *nobody
    /// looked yet*. Without it, curation has to lie in one of two directions.
    Unknown { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RulesError {
    SchemaMismatch { found: u32, expected: u32 },
    Malformed { reason: String },
}

impl std::fmt::Display for RulesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RulesError::SchemaMismatch { found, expected } => {
                write!(f, "rules schema {found}, expected {expected}")
            }
            RulesError::Malformed { reason } => write!(f, "invalid rules: {reason}"),
        }
    }
}

impl std::error::Error for RulesError {}

pub struct Rules {
    requirements: Requirements,
    corrections: Corrections,
}

/// Written by hand, and it prints counts. The derived version would dump 641 achievements
/// with all their refs into whatever message asked for it — a panic line, an `expect_err`,
/// a log — which is unreadable rather than dangerous, but unreadable is enough.
impl std::fmt::Debug for Rules {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Rules")
            .field("snapshot", &self.requirements.generated_from.snapshot_at)
            .field("achievements", &self.requirements.achievements.len())
            .field("targets", &self.requirements.targets.len())
            .field("aliases", &self.corrections.aliases.len())
            .field("verdicts", &self.corrections.verdicts.len())
            .finish()
    }
}

impl Rules {
    pub fn build(r: Requirements, c: Corrections) -> Result<Rules, RulesError> {
        for found in [r.schema_version, c.schema_version] {
            if found != SCHEMA_VERSION {
                return Err(RulesError::SchemaMismatch {
                    found,
                    expected: SCHEMA_VERSION,
                });
            }
        }
        Ok(Rules {
            requirements: r,
            corrections: c,
        })
    }

    /// The label the catalog is searched by. The wiki writes "Jacob and Esau", the game
    /// writes "Jacob & Esau": one alias carries fifteen refs.
    pub fn alias<'a>(&'a self, label: &'a str) -> &'a str {
        self.corrections
            .aliases
            .get(label)
            .map(String::as_str)
            .unwrap_or(label)
    }

    pub fn verdict(&self, key: &str) -> Option<&Verdict> {
        self.corrections.verdicts.get(key)
    }

    /// Every key a verdict was written for. Used to catch verdicts left behind by a
    /// snapshot that dropped their target.
    pub fn verdict_keys(&self) -> impl Iterator<Item = &str> {
        self.corrections.verdicts.keys().map(String::as_str)
    }

    pub fn targets(&self) -> &[TargetRow] {
        &self.requirements.targets
    }

    pub fn refs(&self, achievement: u32) -> &[RefRow] {
        self.requirements
            .achievements
            .get(&achievement)
            .map(|a| a.refs.as_slice())
            .unwrap_or(&[])
    }

    pub fn generated_from(&self) -> &GeneratedFrom {
        &self.requirements.generated_from
    }
}

static EMBEDDED: std::sync::OnceLock<Result<Rules, RulesError>> = std::sync::OnceLock::new();

/// The rules compiled into the binary. There is deliberately no runtime path for "rules
/// missing": there is no file that can be missing. Rules read from disk at runtime would
/// buy updatability-without-recompiling at the cost of one more failure mode — a trade
/// worth making for `log-watch`, whose patterns chase the game's patches, and not here,
/// where they chase a wiki snapshot that is already inside the binary.
pub fn embedded() -> Result<&'static Rules, &'static RulesError> {
    EMBEDDED
        .get_or_init(|| {
            let r: Requirements = serde_json::from_str(include_str!("../rules/requirements.json"))
                .map_err(|e| RulesError::Malformed {
                    reason: e.to_string(),
                })?;
            let c: Corrections = serde_json::from_str(include_str!("../rules/corrections.json"))
                .map_err(|e| RulesError::Malformed {
                    reason: e.to_string(),
                })?;
            Rules::build(r, c)
        })
        .as_ref()
}

/// The key a target is addressed by in `corrections.json`: `kind:label`. The label is the
/// bridge, not the id — the wiki's entity ids and the catalog's boss ids are two different
/// numbering spaces (Gish is entity 43 and boss 19).
pub fn target_key(t: &Target, label: &str) -> String {
    let kind = match t {
        Target::Item { .. } => "item",
        Target::Trinket { .. } => "trinket",
        Target::Character { .. } => "character",
        Target::Achievement { .. } => "achievement",
        Target::Challenge { .. } => "challenge",
        Target::Entity { .. } => "entity",
        Target::Transformation { .. } => "transformation",
        Target::Stage { .. } => "stage",
        Target::Room { .. } => "room",
        Target::Pickup { .. } => "pickup",
    };
    format!("{kind}:{label}")
}
