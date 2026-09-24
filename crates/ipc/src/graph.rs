//! The contracts for the graph screens: Unlock, Next Steps, Plan. A single node shared
//! by all three. What the graph can't interpret travels as a declared `Partial`, never as
//! a value that looks computed — `GraphInfo::Stub` left the wire with M2, because a node
//! saying "the graph doesn't exist" would now be lying.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use wiki::{Dataset, Target};

pub use crate::goals::{Goal, GoalId, TargetKey, UnlockTarget};

/// How many "next steps" the screen shows. Presentation, not domain.
pub const STEPS: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct UnlockNode {
    pub achievement: AchievementRef,
    /// From the save, section 1: real.
    pub done: bool,
    /// From the catalog, reverse index: real. Empty if it unlocks nothing known.
    pub unlocks: Vec<UnlockTarget>,
    /// Origin DLC of the first item unlocked: real.
    pub origin: Option<OriginView>,
    /// What is still in the way, typed by the nature of the target. This is what the
    /// screen groups by: "you're missing 1 character and 2 bosses" instead of
    /// "blocked by 3".
    pub missing: Vec<RequirementView>,
    /// What the graph knows.
    pub graph: GraphInfo,
}

/// What a node is still missing. Typed because the type decides both the grouping and
/// whether there is an achievement behind it at all.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum RequirementView {
    Character {
        id: u32,
        /// Shared by the base and Tainted forms, like `UnlockTarget::Character`: the flag
        /// beside it is what tells the two apart.
        name: String,
        tainted: bool,
        /// The wiki page that says how *this* is unlocked. `None` means the dataset has no
        /// page for it: the name shows and does not link. Never "no requirement".
        page: Option<Target>,
    },
    Boss {
        id: u32,
        name: String,
        page: Option<Target>,
    },
    Challenge {
        id: u32,
        name: String,
        page: Option<Target>,
    },
    /// `itemKind` and not `kind`: the tag already took that name.
    Item {
        item_kind: ItemKindView,
        id: u32,
        name: String,
        page: Option<Target>,
    },
    /// A curated gate — stage, room, mode. The label is what the wiki calls it. No page by
    /// construction: it is a condition we chose not to resolve to an entity.
    Gate { label: String },
    /// One cell of the completion matrix: go and beat `column` with this character.
    ///
    /// No progress field, unlike `Counter`: for one cell the state is binary, and an
    /// invented percentage would be a number nobody measured.
    Mark {
        character: u32,
        character_name: String,
        column: MarkColumnView,
        level: MarkLevelView,
    },
    /// A tally and its threshold, with where the profile stands. Unlike every other
    /// requirement here, this one is not a wall: it is content already reachable.
    Counter {
        label: String,
        current: u32,
        at_least: u32,
    },
    /// A transformation: N of a set of items, in any combination. Like `Counter`, it is not
    /// a wall — it appears only while the profile is short of the count — and unlike every
    /// other member here it lists the things that would satisfy it rather than the one thing
    /// that blocks it.
    Threshold {
        transformation: u32,
        label: String,
        current: u32,
        at_least: u32,
        of: Vec<ThresholdItemView>,
        /// How many of the wiki's contributors this catalog does not have. They can only
        /// ever add to `current`, never subtract, so a non-zero value means the count shown
        /// is a floor.
        unresolved: u32,
        /// The transformation's own wiki page.
        page: Option<Target>,
    },
    /// Not interpreted. A node carrying one cannot claim "available now".
    Unknown { label: String },
}

/// One item of a threshold's set, with where the profile stands on it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ThresholdItemView {
    /// `itemKind` and not `kind`, for the same reason as `RequirementView::Item`.
    pub item_kind: ItemKindView,
    pub id: u32,
    pub name: String,
    /// Whether the profile can already find it: its achievement is done, or nothing gates it.
    pub unlocked: bool,
    pub page: Option<Target>,
}

/// The twelve columns, as a value on the wire. Fieldless, so it is a bare camelCase string
/// and the TypeScript is a union of values — the repo's rule, zero exceptions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum MarkColumnView {
    MomsHeart,
    Isaac,
    Satan,
    BossRush,
    BlueBaby,
    TheLamb,
    MegaSatan,
    Greed,
    Hush,
    Delirium,
    Mother,
    TheBeast,
}

/// A level inside a cell, named for its bit. `Second` is Ultra Greedier in the Greed
/// column, measured; what it means elsewhere is not, and `hard` would ship that claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum MarkLevelView {
    Base,
    Second,
}

/// One rule for the enums on this boundary: those whose variants carry different data are
/// tagged on `kind`; those with no fields travel as a bare string. `UnlockTarget::Item`
/// carries `item_kind`, not `kind`, because `kind` is the tag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum AchievementRef {
    Known {
        id: u32,
        text: String,
        /// **How to get it**, in one line: the game's own `unlock_condition` when
        /// `achievements.xml` states one, and the wiki's requirement when it does not.
        ///
        /// Two sources, because the file alone is not enough. Measured 2026-09-13 on the
        /// reference profile: of 637 known achievements the file answers for 283, and among
        /// the 119 unlockable *now* — what the landing page draws from — for only 16. It was
        /// called `hint` while it was only the file's; the name changed with the meaning, so
        /// that every reader had to be revisited rather than silently widened.
        ///
        /// The game's words win where it has any: the wiki is the fallback, never a rewrite.
        condition: Option<String>,
        icon_url: Option<String>,
    },
    /// In the save but not in the catalog: a patch newer than the file.
    Unknown { slot: u32 },
}

/// `Partial` carries no count of the steps still missing, on purpose: with a requirement
/// uninterpreted, or a node caught in a cycle, the transitive count is not knowable, and a zero
/// would read as "nothing in the way". A node that is `Partial` must never be drawn as
/// unlockable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum GraphInfo {
    Computed {
        available_now: bool,
        blocked_by: u32,
        fan_out: u32,
        steps_missing: u32,
    },
    /// Requirements only partly interpreted, or a node inside a cycle. It carries no
    /// `steps_missing` on purpose: with something uninterpreted the transitive count isn't
    /// knowable, and a zero would be the exact lie this variant exists to prevent.
    ///
    /// A variant and not one more field on `Computed`, because a new variant **forces**
    /// the TypeScript `switch` to deal with it while a field is ignored in silence.
    Partial {
        blocked_by: u32,
        fan_out: u32,
        unknown: u32,
    },
}

/// `catalog::Origin` doesn't cross the IPC boundary: this is its view, like `ItemKindView` for `ItemKind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum OriginView {
    Rebirth,
    Afterbirth,
    AfterbirthPlus,
    Repentance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct UnlockView {
    pub nodes: Vec<UnlockNode>,
    pub totals: UnlockTotals,
    pub diagnostics: Vec<UnlockDiagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct UnlockTotals {
    pub slots: u32,
    pub done: u32,
    pub known: u32,
    pub unknown: u32,
}

/// `NoAchievementSection` means section 1 of the save was not read: nodes and totals are
/// zero, and that does not mean "zero achievements done".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum UnlockDiagnostic {
    SlotsBeyondCatalog {
        count: u32,
    },
    CatalogBeyondSlots {
        count: u32,
    },
    NoCatalog,
    /// Section 1 of the save wasn't read: no nodes, no totals. This is not "zero
    /// achievements done", and it's the only diagnostic that zeroes out the screen.
    NoAchievementSection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct NextSteps {
    pub sections: Vec<StepsSection>,
}

/// Both answers the graph's screens need, from one reading of the profile (N8).
///
/// They were two commands, and the frontend asked for them together — with a comment saying
/// why: *"one read, because both answers belong to the same profile and asking twice could
/// straddle a change"*. Two commands could not keep that promise. Each rebuilt the whole
/// pipeline behind it: the settings file, a walk of the Steam libraries, the `.dat` read
/// whole, its parse, 642 nodes and the evaluation — **twice for one screen load**, and
/// across two moments, so a save written in between made the steps disagree with the list
/// they are a filter over.
///
/// One command is not a cache: there is nothing to invalidate, and nothing is remembered
/// between calls. The steps stay a pure function of the view here, as they always were.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct GraphViews {
    pub unlock: UnlockView,
    pub steps: NextSteps,
}

/// The pair, built from one view: the steps are the filter over exactly the list that
/// travels beside them. `queued` is what the plan queue already holds, which the steps leave
/// out — see `next_steps`.
pub fn graph_views(unlock: UnlockView, queued: &BTreeSet<u32>) -> GraphViews {
    let steps = next_steps(&unlock, queued);
    GraphViews { unlock, steps }
}

/// One reason, and the steps it produced. The screen draws the basis as a heading, because a
/// row is worth showing only together with why it is being suggested.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct StepsSection {
    pub basis: StepsBasis,
    pub steps: Vec<UnlockNode>,
}

/// What a section is ordered by. A fieldless enum: on the wire it's `"fanOut"`, not a
/// tagged object — the same rule as `ItemKindView` and `OriginView`.
///
/// `Closeness` is the basis this type was left open for. A counter is the **only**
/// requirement that carries a distance — a mark is binary and a character is a wall — so it
/// is the only one that can order a list by how near the profile is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum StepsBasis {
    FanOut,
    Closeness,
}

/// A goal as the UI sees it: the saved key plus whatever the current catalog knows
/// about it. `target: None` means "not resolvable right now" — game not installed, or
/// an id a patch has removed: the goal stays visible and is removed by its `key`,
/// which is the only thing the database holds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct GoalView {
    pub id: GoalId,
    pub key: TargetKey,
    pub target: Option<UnlockTarget>,
    pub created_unix: i64,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct PlanView {
    pub goals: Vec<GoalView>,
    pub expansion: PlanExpansion,
    /// What couldn't be read from the plan: one row per unreadable goal.
    pub diagnostics: Vec<PlanDiagnostic>,
    /// `false` when `store` failed to open: goals can't be seen or added, and the UI
    /// must say so instead of showing an empty list.
    pub store_available: bool,
}

/// The plan degrades and says why. `store_available` is derived from the absence of
/// `StoreUnavailable`: the UI uses it as a gate and reads the diagnostics for the text.
/// `UnreadableGoal` and `UnresolvedGoal` carry the id, so the UI can offer to remove them.
/// `NoCatalog` arrives once, not once per goal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum PlanDiagnostic {
    /// The database failed to open, and which case it is. `NewerSchema` is the only one
    /// the user can act on, which is why its versions travel as numbers and not inside a
    /// sentence built in Rust.
    StoreUnavailable { reason: crate::StoreReason },
    /// A `store` row whose target this version can't read: it stays in the file and
    /// is named by id, so the user can remove it.
    UnreadableGoal { id: GoalId },
    /// No catalog (game not installed): no goal resolves, and none get added. A
    /// single diagnostic, not one per goal.
    NoCatalog,
    /// The catalog exists but no longer knows this key: an id a patch has removed, or
    /// a game file that can't be read today. The goal stays.
    UnresolvedGoal { id: GoalId },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum PlanExpansion {
    Stub,
    Computed { steps: Vec<PlanStep> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct PlanStep {
    pub goal: GoalId,
    pub node: UnlockNode,
    pub done: bool,
}

use catalog::{AchievementId, BossId, Catalog, ChallengeId, CharacterId, ItemId, Origin, Unlock};

use crate::catalog_view::{item_kind, kind_view, ItemKindView};
use crate::icon::IconRef;
use crate::wiki_target;

/// How to get an achievement, in one line. The game's `unlock_condition` first — it is the
/// game's own words about its own unlock — and the wiki's requirement only where the file
/// says nothing, which is 354 of 637 achievements on the reference profile.
///
/// The wiki's requirement is an inline tree; `wiki::plain` reads it the way a reader would,
/// so a reference becomes its label and an edition wrapper keeps its words. Whitespace-only
/// is no answer: a blank line under a headline reads as a condition nobody wrote.
fn condition_of(a: &catalog::Achievement, dataset: Option<&Dataset>) -> Option<String> {
    if let Some(from_file) = a.unlock_condition.clone() {
        return Some(from_file);
    }
    let entry = dataset?.entry(&wiki_target::achievement(a.id))?;
    let wiki::Infobox::Achievement { requirements, .. } = &entry.infobox else {
        return None;
    };
    let line = wiki::plain(requirements).trim().to_string();
    (!line.is_empty()).then_some(line)
}

/// A page, only when the dataset really has one. `Some(target)` is a link the screen can
/// follow; `None` is a name it draws without one — never a link that leads nowhere.
fn page_of(dataset: Option<&Dataset>, target: Option<Target>) -> Option<Target> {
    let (ds, t) = (dataset?, target?);
    ds.entry(&t).is_some().then_some(t)
}

/// The Unlock view: one node per slot 1..=N of section 1 of the save. `flags[i]` is
/// slot i; slot 0 is unused (the `slot[id]` mapping, verified on 2026-09-05: 169 items
/// out of 171 seen with the achievement done).
///
/// `flags: None` means "section 1 wasn't read", and it is not the same thing as a save
/// with no achievements: flattening the two cases would make the view claim the catalog
/// has 638 more achievements than the file, which is false. `Some(&[])` stays the
/// degenerate save, with its own diagnostic.
/// The requirements still in the way, resolved to names. What is already satisfied is left
/// out: a node blocked by nothing shows an empty list, and that agrees with `blocked_by`.
/// `Requirement::None` never reaches here — it was judged as gating nothing.
fn missing_view(
    c: &Catalog,
    dataset: Option<&Dataset>,
    node: &graph::build::Node,
    flags: &[bool],
    progress: Option<&dyn graph::Profile>,
) -> Vec<RequirementView> {
    let en = catalog::Language::English;
    let done = |a: Option<AchievementId>| {
        a.and_then(|a| flags.get(a.0 as usize).copied())
            .unwrap_or(false)
    };
    let mut out = Vec::new();
    for r in &node.requirements {
        match r {
            graph::model::Requirement::None => {}
            graph::model::Requirement::Mark {
                character,
                column,
                level,
            } => {
                let Some(p) = progress else { continue };
                let Some(ch) = c.character(*character) else {
                    continue;
                };
                // Reached already: not missing. Cannot say: not this list's job to report
                // it either — the node is `Partial` and that is where it says so.
                match p.mark(*character, *column) {
                    None => {}
                    Some(reached) if reached.is_some_and(|r| r >= *level) => {}
                    Some(_) => out.push(RequirementView::Mark {
                        character: character.0,
                        character_name: c.text(&ch.name, en).to_string(),
                        column: column_view(*column),
                        level: level_view(*level),
                    }),
                }
            }
            graph::model::Requirement::Counter { name, at_least } => {
                let Some(p) = progress else { continue };
                let Some(current) = p.counter(*name) else {
                    continue;
                };
                if current < *at_least {
                    out.push(RequirementView::Counter {
                        label: counter_label(*name).to_string(),
                        current,
                        at_least: *at_least,
                    });
                }
            }
            graph::model::Requirement::Character { id } => {
                let Some(ch) = c.character(*id) else { continue };
                if !done(ch.unlocked_by) {
                    out.push(RequirementView::Character {
                        id: id.0,
                        name: c.text(&ch.name, en).to_string(),
                        tainted: ch.tainted,
                        page: page_of(dataset, Some(wiki_target::character(ch))),
                    });
                }
            }
            graph::model::Requirement::Boss { id } => {
                let Some(b) = c.boss(*id) else { continue };
                if !done(b.unlocked_by) {
                    out.push(RequirementView::Boss {
                        id: id.0,
                        name: b.name.clone(),
                        page: page_of(dataset, wiki_target::boss(c, b)),
                    });
                }
            }
            graph::model::Requirement::Challenge { id } => {
                let Some(ch) = c.challenge(*id) else { continue };
                let unlocked = ch.unlocked_by.iter().any(|a| done(Some(*a)));
                if !ch.unlocked_by.is_empty() && !unlocked {
                    out.push(RequirementView::Challenge {
                        id: id.0,
                        name: ch.name.clone(),
                        page: page_of(dataset, Some(wiki_target::challenge(ch))),
                    });
                }
            }
            graph::model::Requirement::Item { kind, id } => {
                let Some(i) = c.item(*kind, *id) else {
                    continue;
                };
                if !done(i.unlocked_by) {
                    out.push(RequirementView::Item {
                        item_kind: kind_view(*kind),
                        id: id.0,
                        name: c.text(&i.name, en).to_string(),
                        page: page_of(dataset, Some(wiki_target::item(i))),
                    });
                }
            }
            graph::model::Requirement::Threshold {
                transformation,
                label,
                at_least,
                of,
                unresolved,
            } => {
                let items: Vec<ThresholdItemView> = of
                    .iter()
                    .filter_map(|t| {
                        let i = c.item(t.kind, t.id)?;
                        Some(ThresholdItemView {
                            item_kind: kind_view(t.kind),
                            id: t.id.0,
                            name: c.text(&i.name, en).to_string(),
                            unlocked: done(i.unlocked_by),
                            page: page_of(dataset, Some(wiki_target::item(i))),
                        })
                    })
                    .collect();
                let current = items.iter().filter(|i| i.unlocked).count() as u32;
                // Like `Counter`: it appears only while it is not met. A threshold already
                // reached is not in the way, and the list here is what is in the way.
                if current < *at_least {
                    out.push(RequirementView::Threshold {
                        transformation: *transformation,
                        label: label.clone(),
                        current,
                        at_least: *at_least,
                        of: items,
                        unresolved: *unresolved,
                        page: page_of(
                            dataset,
                            Some(Target::Transformation {
                                id: *transformation,
                            }),
                        ),
                    });
                }
            }
            graph::model::Requirement::Gate { gate } => out.push(RequirementView::Gate {
                label: gate
                    .split_once(':')
                    .map(|(_, l)| l)
                    .unwrap_or(gate)
                    .to_string(),
            }),
            graph::model::Requirement::Unknown { label } => out.push(RequirementView::Unknown {
                label: label.clone(),
            }),
        }
    }
    out
}

/// `graph` and `eval` travel together or not at all: without a catalog there is no graph,
/// and a node then carries `Partial` with one unknown — never `Computed`, which would read
/// as "nothing is in the way".
pub fn unlock_view(
    catalog: Option<&Catalog>,
    dataset: Option<&Dataset>,
    flags: Option<&[bool]>,
    graph: Option<&graph::Graph>,
    eval: Option<&graph::evaluate::Eval>,
    progress: Option<&dyn graph::Profile>,
    mut icon: impl FnMut(&IconRef) -> Option<String>,
) -> UnlockView {
    let read = flags.unwrap_or(&[]);
    let slots = read.len() as u32;
    let mut nodes = Vec::with_capacity(read.len().saturating_sub(1));
    let (mut done, mut known, mut unknown) = (0u32, 0u32, 0u32);
    for (slot, &flag) in read.iter().enumerate().skip(1) {
        let slot = slot as u32;
        let achievement = catalog.and_then(|c| c.achievement(AchievementId(slot)));
        let (achievement_ref, unlocks, origin) = match (catalog, achievement) {
            (Some(c), Some(a)) => {
                known += 1;
                let unlocks: Vec<UnlockTarget> = c
                    .unlocks(a.id)
                    .iter()
                    .map(|u| target_of(c, u, dataset, &mut icon))
                    .collect();
                let origin = first_item_origin(c, c.unlocks(a.id));
                (
                    AchievementRef::Known {
                        id: a.id.0,
                        text: a.text.clone(),
                        condition: condition_of(a, dataset),
                        icon_url: icon(&IconRef::Achievement { id: a.id.0 }),
                    },
                    unlocks,
                    origin,
                )
            }
            _ => {
                unknown += 1;
                (AchievementRef::Unknown { slot }, Vec::new(), None)
            }
        };
        if flag {
            done += 1;
        }
        let info = match eval.and_then(|e| e.node(AchievementId(slot))) {
            Some(graph::evaluate::NodeInfo::Computed {
                available_now,
                blocked_by,
                fan_out,
                steps_missing,
            }) => GraphInfo::Computed {
                available_now: *available_now,
                blocked_by: *blocked_by,
                fan_out: *fan_out,
                steps_missing: *steps_missing,
            },
            Some(graph::evaluate::NodeInfo::Partial {
                blocked_by,
                fan_out,
                unknown,
            }) => GraphInfo::Partial {
                blocked_by: *blocked_by,
                fan_out: *fan_out,
                unknown: *unknown,
            },
            // No graph for this slot: no catalog, or a slot beyond it. The graph has
            // nothing to say, which is `Partial` with one unknown — never `Computed`.
            None => GraphInfo::Partial {
                blocked_by: 0,
                fan_out: 0,
                unknown: 1,
            },
        };
        let missing = match (catalog, graph.and_then(|g| g.node(AchievementId(slot)))) {
            (Some(c), Some(n)) => missing_view(c, dataset, n, read, progress),
            _ => Vec::new(),
        };
        nodes.push(UnlockNode {
            achievement: achievement_ref,
            done: flag,
            unlocks,
            origin,
            missing,
            graph: info,
        });
    }

    let mut diagnostics = Vec::new();
    // The two diagnostics that compare the file against the catalog assume the
    // achievement ids are contiguous 1..=N in the catalog, so that `in_catalog + 1` is
    // the number of slots the catalog expects. That holds on real files (2026-09-05:
    // 1..=637, no gaps) and isn't verified in code: a gap in the ids would cause an
    // undercount, never an overcount.
    match (catalog, flags) {
        (Some(c), Some(_)) => {
            let in_catalog = c.achievements().count() as u32;
            if unknown > 0 {
                diagnostics.push(UnlockDiagnostic::SlotsBeyondCatalog { count: unknown });
            }
            // `slots == 0` (section read but empty) is a degenerate save: the
            // diagnostic still fires, and that's intentional.
            if in_catalog + 1 > slots {
                diagnostics.push(UnlockDiagnostic::CatalogBeyondSlots {
                    count: in_catalog + 1 - slots,
                });
            }
        }
        // Without a catalog every slot is `unknown`: `SlotsBeyondCatalog` would just
        // repeat `NoCatalog` with a number attached. Without section 1 there's nothing
        // to compare.
        (None, _) => diagnostics.push(UnlockDiagnostic::NoCatalog),
        (Some(_), None) => {}
    }
    if flags.is_none() {
        diagnostics.push(UnlockDiagnostic::NoAchievementSection);
    }
    UnlockView {
        nodes,
        totals: UnlockTotals {
            slots,
            done,
            known,
            unknown,
        },
        diagnostics,
    }
}

/// A saved key as the UI sees it: name resolved against the current catalog, and an
/// icon if the sprite can be extracted. `None` when the catalog no longer knows the
/// key: this is the one place that decides whether a goal resolves.
pub fn resolve_target(
    c: &Catalog,
    key: &TargetKey,
    dataset: Option<&Dataset>,
    icon: &mut impl FnMut(&IconRef) -> Option<String>,
) -> Option<UnlockTarget> {
    let english = catalog::Language::English;
    let mut resolved = crate::goals::Resolved::default();
    match *key {
        TargetKey::Item { item_kind: k, id } => {
            let i = c.item(item_kind(k), ItemId(id))?;
            resolved.name = c.text(&i.name, english).to_string();
            resolved.icon_url = icon(&IconRef::Item { kind: k, id });
            resolved.page = page_of(dataset, Some(wiki_target::item(i)));
        }
        TargetKey::Character { id } => {
            let ch = c.character(CharacterId(id))?;
            resolved.name = c.text(&ch.name, english).to_string();
            // The base and Tainted forms carry the same name key: without the flag the two
            // go out as one character (`docs/BACKLOG.md` B28).
            resolved.tainted = ch.tainted;
            resolved.page = page_of(dataset, Some(wiki_target::character(ch)));
        }
        TargetKey::Boss { id } => {
            let b = c.boss(BossId(id))?;
            resolved.name = b.name.clone();
            // A row `boss_keys` leaves without a key names no page: `None`, never a guessed
            // variant (`wiki_target::boss`).
            resolved.page = page_of(dataset, wiki_target::boss(c, b));
        }
        TargetKey::Challenge { id } => {
            let ch = c.challenge(ChallengeId(id))?;
            resolved.rewards = ch.rewards.iter().map(|a| a.0).collect();
            resolved.name = ch.name.clone();
            resolved.page = page_of(dataset, Some(wiki_target::challenge(ch)));
        }
    }
    Some(key.view(resolved))
}

/// A catalog edge as the UI sees it. The edge is born from the catalog that resolves it
/// (`build_unlocks` constructs it straight from the entities themselves), so the key is
/// always there; if one day it weren't, the row stays nameless instead of vanishing.
pub fn target_of(
    c: &Catalog,
    u: &Unlock,
    dataset: Option<&Dataset>,
    icon: &mut impl FnMut(&IconRef) -> Option<String>,
) -> UnlockTarget {
    let key = key_of(u);
    resolve_target(c, &key, dataset, icon)
        .unwrap_or_else(|| key.view(crate::goals::Resolved::default()))
}

/// The key of a catalog edge. The catalog's ids are newtypes; at the boundary they aren't.
fn key_of(u: &Unlock) -> TargetKey {
    match *u {
        Unlock::Item { kind, id } => TargetKey::Item {
            item_kind: kind_view(kind),
            id: id.0,
        },
        Unlock::Character { id } => TargetKey::Character { id: id.0 },
        Unlock::Boss { id } => TargetKey::Boss { id: id.0 },
        Unlock::Challenge { id } => TargetKey::Challenge { id: id.0 },
    }
}

fn first_item_origin(c: &Catalog, unlocks: &[Unlock]) -> Option<OriginView> {
    match unlocks.first()? {
        Unlock::Item { kind, id } => c.item(*kind, *id)?.origin.map(origin_view),
        Unlock::Character { .. } | Unlock::Boss { .. } | Unlock::Challenge { .. } => None,
    }
}

pub(crate) fn origin_view(o: Origin) -> OriginView {
    match o {
        Origin::Rebirth => OriginView::Rebirth,
        Origin::Afterbirth => OriginView::Afterbirth,
        Origin::AfterbirthPlus => OriginView::AfterbirthPlus,
        Origin::Repentance => OriginView::Repentance,
    }
}

/// The slot a node occupies: its achievement's id, or the bare slot when the catalog names
/// none. What ties break on, so the order never depends on the nodes' arrival.
fn slot_of(n: &UnlockNode) -> u32 {
    match n.achievement {
        AchievementRef::Known { id, .. } => id,
        AchievementRef::Unknown { slot } => slot,
    }
}

fn fan_out_of(n: &UnlockNode) -> u32 {
    match n.graph {
        GraphInfo::Computed { fan_out, .. } | GraphInfo::Partial { fan_out, .. } => fan_out,
    }
}

/// How far the profile is from a requirement, when the requirement is a tally. Every other
/// kind answers `None`, and says why: a mark is binary, a character or an item is a wall, a
/// gate and an uninterpreted label are conditions nobody measured. None of them is a
/// distance, and a list ordered by nearness can only hold things that have one.
fn counter_remaining(r: &RequirementView) -> Option<u32> {
    match *r {
        RequirementView::Counter {
            current, at_least, ..
        } => Some(at_least.saturating_sub(current)),
        RequirementView::Character { .. }
        | RequirementView::Boss { .. }
        | RequirementView::Challenge { .. }
        | RequirementView::Item { .. }
        | RequirementView::Gate { .. }
        | RequirementView::Mark { .. }
        // A threshold looks like a distance and is not one. A tally's remainder counts
        // events the player performs; a threshold's counts items that have to *drop*, and
        // one item away can be a hundred runs or the next room. Putting the two on one
        // scale would order the list by a number that means two different things.
        | RequirementView::Threshold { .. }
        | RequirementView::Unknown { .. } => None,
    }
}

/// `Some(distance)` when the node still has requirements and **every one of them** is a
/// counter: the sum is how far the profile is from the whole set. One requirement of any
/// other kind and the answer is `None` — `Option`'s `Sum` short-circuits, which is exactly
/// the rule, and a node with nothing missing has no distance to speak of either.
fn closeness(n: &UnlockNode) -> Option<u32> {
    if n.missing.is_empty() {
        return None;
    }
    n.missing.iter().map(counter_remaining).sum()
}

/// The steps worth playing tonight, in sections. **Unlockable now** is the gate for all of
/// them: a node that is merely not-done isn't a step — if it's blocked, tonight can't touch
/// it; if the graph can't say (`Partial`), suggesting it would be a guess.
///
/// `Closeness` claims first, because "two runs from it" says more about a node than its
/// fan-out does. It claims only the ones that actually fit under `STEPS`, so a candidate
/// beyond the cap falls back to the other section instead of vanishing from both.
///
/// A section with no steps is not emitted: "absent" is decided once, here, rather than by
/// each screen deciding what an empty array means.
///
/// Ties break by slot ascending, so two calls on the same profile give the same list: an
/// order that shuffles reads as the app changing its mind.
///
/// What is already in the plan queue is not a suggestion: it is a decision taken, and a
/// place under the cap spent on it is a place the next candidate did not get. It leaves
/// before either section claims, so it falls back to neither.
pub fn next_steps(view: &UnlockView, queued: &BTreeSet<u32>) -> NextSteps {
    let available: Vec<&UnlockNode> = view
        .nodes
        .iter()
        .filter(|n| {
            matches!(
                n.graph,
                GraphInfo::Computed {
                    available_now: true,
                    ..
                }
            )
        })
        .filter(|n| match n.achievement {
            AchievementRef::Known { id, .. } => !queued.contains(&id),
            // A slot the catalog does not name cannot be queued.
            AchievementRef::Unknown { .. } => true,
        })
        .collect();

    let mut close: Vec<(u32, &UnlockNode)> = available
        .iter()
        .filter_map(|n| closeness(n).map(|d| (d, *n)))
        .collect();
    close.sort_by_key(|(d, n)| (*d, std::cmp::Reverse(fan_out_of(n)), slot_of(n)));
    close.truncate(STEPS);

    let claimed: std::collections::HashSet<u32> = close.iter().map(|(_, n)| slot_of(n)).collect();
    let mut open: Vec<&UnlockNode> = available
        .into_iter()
        .filter(|n| !claimed.contains(&slot_of(n)))
        .collect();
    open.sort_by_key(|n| (std::cmp::Reverse(fan_out_of(n)), slot_of(n)));
    open.truncate(STEPS);

    let sections = [
        (
            StepsBasis::FanOut,
            open.into_iter().cloned().collect::<Vec<_>>(),
        ),
        (
            StepsBasis::Closeness,
            close
                .into_iter()
                .map(|(_, n)| n.clone())
                .collect::<Vec<_>>(),
        ),
    ]
    .into_iter()
    .filter(|(_, steps)| !steps.is_empty())
    .map(|(basis, steps)| StepsSection { basis, steps })
    .collect();

    NextSteps { sections }
}

/// The plan: the saved goals resolved against the current catalog, and an expansion
/// that M3 can't compute yet. The database only keeps the keys, so name and icon are
/// born here: a goal saved when the game wasn't there shows its name as soon as the
/// game is. `store_unavailable` is the single source: `store_available` and the
/// `StoreUnavailable` diagnostic both derive from it and can never contradict each
/// other.
///
/// The diagnostics come out in this order: the store, the catalog, the unreadable
/// rows, the unresolved keys — from the problem that explains the most to the one that
/// explains the least.
pub fn plan_view(
    catalog: Option<&Catalog>,
    dataset: Option<&Dataset>,
    goals: Vec<Goal>,
    unreadable: Vec<GoalId>,
    store_unavailable: Option<crate::StoreReason>,
    mut icon: impl FnMut(&IconRef) -> Option<String>,
) -> PlanView {
    let store_available = store_unavailable.is_none();
    let mut unresolved = Vec::new();
    let goals: Vec<GoalView> = goals
        .into_iter()
        .map(|g| {
            let target = catalog.and_then(|c| resolve_target(c, &g.target, dataset, &mut icon));
            // Without a catalog nothing resolves, and `NoCatalog` already says so:
            // flagging every goal would just repeat the same news one row at a time.
            if target.is_none() && catalog.is_some() {
                unresolved.push(g.id.clone());
            }
            GoalView {
                id: g.id,
                key: g.target,
                target,
                created_unix: g.created_unix,
                note: g.note,
            }
        })
        .collect();
    let diagnostics = store_unavailable
        .into_iter()
        .map(|reason| PlanDiagnostic::StoreUnavailable { reason })
        .chain(catalog.is_none().then_some(PlanDiagnostic::NoCatalog))
        .chain(
            unreadable
                .into_iter()
                .map(|id| PlanDiagnostic::UnreadableGoal { id }),
        )
        .chain(
            unresolved
                .into_iter()
                .map(|id| PlanDiagnostic::UnresolvedGoal { id }),
        )
        .collect();
    PlanView {
        goals,
        expansion: PlanExpansion::Stub,
        diagnostics,
        store_available,
    }
}

/// The graph's column as the wire's. No `_` arm: the two are the same twelve, and a
/// thirteenth has to break the build rather than fall into a default.
fn column_view(c: graph::rules::MarkColumn) -> MarkColumnView {
    use graph::rules::MarkColumn as M;
    match c {
        M::MomsHeart => MarkColumnView::MomsHeart,
        M::Isaac => MarkColumnView::Isaac,
        M::Satan => MarkColumnView::Satan,
        M::BossRush => MarkColumnView::BossRush,
        M::BlueBaby => MarkColumnView::BlueBaby,
        M::TheLamb => MarkColumnView::TheLamb,
        M::MegaSatan => MarkColumnView::MegaSatan,
        M::Greed => MarkColumnView::Greed,
        M::Hush => MarkColumnView::Hush,
        M::Delirium => MarkColumnView::Delirium,
        M::Mother => MarkColumnView::Mother,
        M::TheBeast => MarkColumnView::TheBeast,
    }
}

fn level_view(l: graph::rules::MarkLevel) -> MarkLevelView {
    match l {
        graph::rules::MarkLevel::Base => MarkLevelView::Base,
        graph::rules::MarkLevel::Second => MarkLevelView::Second,
    }
}

/// What the screen calls the tally: the boss's English name, because that is what the
/// player is being asked to go and beat. Not the counter's identifier, which is ours.
fn counter_label(n: graph::rules::CounterName) -> &'static str {
    use graph::rules::CounterName as C;
    match n {
        C::HushKills => "Hush",
        C::DeliriumKills => "Delirium",
        C::MotherKills => "Mother",
        C::BeastKills => "The Beast",
    }
}
