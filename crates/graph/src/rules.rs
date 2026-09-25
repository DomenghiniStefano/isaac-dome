//! The two rules files and their join. `requirements.json` is generated from the wiki
//! snapshot and holds no id of ours; `corrections.json` is written by hand. Nothing here
//! knows about the user's catalog: resolution happens later, in `graph::build`.

use catalog::AchievementId;
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use wiki::Target;

pub const SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Requirements {
    pub schema_version: u32,
    pub generated_from: GeneratedFrom,
    pub achievements: BTreeMap<u32, AchievementRefs>,
    /// Every target that doesn't reduce to an achievement on its own: the form the
    /// curation fills in, not a list anyone has to invent.
    pub targets: Vec<TargetRow>,
    /// The transformations, by the wiki's id. `Graph::build` never sees a `Dataset` — the
    /// wiki reaches this crate only through this file — so a threshold's number and its
    /// item list have to travel here to be answerable at runtime.
    ///
    /// `default` is **not** leniency towards an older file — `SCHEMA_VERSION` rejects those
    /// outright. It is so a hand-written fixture that is not about transformations does not
    /// have to spell an empty map to parse.
    #[serde(default)]
    pub transformations: BTreeMap<u32, TransformationRow>,
}

/// One transformation: what it is called, how many of its items are needed, and which.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformationRow {
    pub label: String,
    /// `None` when the wiki page did not state a count in a form the parser reads. The row
    /// is emitted anyway: "we have this transformation and cannot read its count" is a
    /// different thing from "there is no such transformation", and only one of them is true
    /// of Adult.
    pub at_least: Option<u32>,
    /// The items and trinkets that count toward it, as the wiki names them. No id of ours:
    /// this file deliberately knows nothing about the user's catalog.
    pub items: Vec<Target>,
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

/// One target of the inventory. **Every one of them needs a verdict**, entities included: an
/// entity only escapes the verdict table when it resolves to a boss the game itself gates by
/// an achievement — 27 of 103 bosses — and the other 76 have to be judged, or a node behind
/// Delirium would read as "nothing in the way". Judging them all costs a handful of verdicts
/// that are never read; exempting entities left a silent hole, found on 2026-09-07.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetRow {
    pub key: String,
    pub label: String,
    pub uses: u32,
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

/// A closed list, and every addition to it is a decision. A target with no verdict is not
/// "no prerequisite": it stays unknown, and the node that carries it drops to `Partial`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Verdict {
    /// Fought on night one: Satan, Mom, Isaac.
    AlwaysAvailable(bool),
    /// Sits behind an achievement of the graph.
    Behind { achievement: AchievementId },
    /// Not a prerequisite at all: a pickup that appears in the sentence.
    NotAPrerequisite(bool),
    /// Judged, and the answer is that the model can't say it: gated, but by something
    /// other than one achievement — three Guppy items, "a tainted character", "all
    /// endings". At runtime it behaves exactly like no verdict (the node drops to
    /// `Partial`), and it exists to separate *judged and inexpressible* from *nobody
    /// looked yet*. Without it, curation has to lie in one of two directions.
    Unknown { reason: String },
    /// Answered by the profile rather than by another achievement: a cell of the completion
    /// matrix, a tally of section 2, or both.
    ///
    /// Both halves are optional because which one answers is decided by the *reference*,
    /// not by the target: "defeat Mother as Magdalene" asks about that character's cell,
    /// "defeat Mother" asks whether it was ever done at all. A target rarely has both
    /// located — Ultra Greedier has a cell and no tally — and the half a reference needs
    /// may be missing, which is `Unknown` like anything else this crate can't say.
    Progress {
        #[serde(default)]
        mark: Option<MarkRule>,
        #[serde(default)]
        counter: Option<CounterRule>,
    },
}

/// The twelve columns of the completion matrix, in the game's own order: the layout's own
/// enum, not a copy of it. A rules file spells a column the way the layout
/// serializes it (`"momsHeart"`), and nothing more of `core-save` is read from here — the
/// index a cell sits at stays the business of whoever answers `Profile::mark`.
pub use core_save::Column as MarkColumn;

/// A level within a cell, **named for the bit and not for a meaning**. Bit 0 is `Base`,
/// bit 1 is `Second`.
///
/// In the Greed column `Second` is Ultra Greedier, measured 2026-09-12 on three days and
/// three characters. What bit 1 means in the other eleven columns is *not* measured, so a
/// name like `Hard` would assert exactly what this repository has already paid for twice
/// (sections 3 and 6, a mark's bit 2).
///
/// Ordered, `Base` first, so a cell reached at the second level satisfies a requirement
/// for the base one by `reached >= required`.
///
/// The one definition of the two levels: a requirement carries it across the IPC as
/// `MarkLevelView`, a bare camelCase string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ts_rs::TS)]
#[ts(rename = "MarkLevelView")]
#[serde(rename_all = "camelCase")]
pub enum MarkLevel {
    Base,
    Second,
}

/// A tally of section 2, named: the layout's own enum, like the column. A rules file spells
/// it the way the layout serializes it (`"hushKills"`); the index it sits at stays
/// `core-save`'s business, since a rules file carrying an offset would be the same mistake as
/// an offset crossing the IPC.
pub use core_save::CounterKey as CounterName;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkRule {
    pub column: MarkColumn,
    pub level: MarkLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CounterRule {
    pub name: CounterName,
    pub at_least: u32,
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

/// Written by hand, and it prints counts. The derived version would dump every achievement
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
        // A verdict that answers nothing is not the same as no verdict, and at runtime the
        // two would be indistinguishable: both drop the node to `Partial`. Refusing the
        // file keeps curation honest — an empty `progress` is a half-written row, not a
        // judgement, and it must not be able to hide as one.
        for (key, v) in &c.verdicts {
            if let Verdict::Progress {
                mark: None,
                counter: None,
            } = v
            {
                return Err(RulesError::Malformed {
                    reason: format!("{key}: a progress verdict with neither half answers nothing"),
                });
            }
            // The flag is the file's way of writing a unit variant, and `true` is its only
            // meaning: `false` would still resolve to no prerequisite, the opposite of what it
            // reads as.
            if let Verdict::AlwaysAvailable(false) | Verdict::NotAPrerequisite(false) = v {
                return Err(RulesError::Malformed {
                    reason: format!("{key}: a verdict written false says nothing"),
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

    /// Every key a verdict was written for. Used by the curation tests to catch verdicts left
    /// behind by a snapshot that dropped their target.
    #[cfg(feature = "test-api")]
    pub fn verdict_keys(&self) -> impl Iterator<Item = &str> {
        self.corrections.verdicts.keys().map(String::as_str)
    }

    pub fn targets(&self) -> &[TargetRow] {
        &self.requirements.targets
    }

    /// A transformation's row, by the wiki's id. `None` means the snapshot has no such
    /// transformation, which is not the same as one whose count could not be read: that one
    /// is present with `at_least: None`.
    pub fn transformation(&self, id: u32) -> Option<&TransformationRow> {
        self.requirements.transformations.get(&id)
    }

    pub fn refs(&self, achievement: AchievementId) -> &[RefRow] {
        self.requirements
            .achievements
            .get(&achievement.0)
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
        Target::Concept { .. } => "concept",
    };
    format!("{kind}:{label}")
}
