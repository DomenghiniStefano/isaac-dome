//! The contracts for the graph screens: Unlock, Next Steps, Plan. A single node shared
//! by all three. Whatever the graph (M2) doesn't know yet travels as a declared `Stub`,
//! never as a value that looks computed.

use serde::{Deserialize, Serialize};

pub use crate::goals::{Goal, GoalId, TargetKey, UnlockTarget};

/// How many "next steps" the screen shows. Presentation, not domain.
pub const STEPS: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockNode {
    pub achievement: AchievementRef,
    /// From the save, section 1: real.
    pub done: bool,
    /// From the catalog, reverse index: real. Empty if it unlocks nothing known.
    pub unlocks: Vec<UnlockTarget>,
    /// Origin DLC of the first item unlocked: real.
    pub origin: Option<OriginView>,
    /// What the graph knows: `Stub` until M2 exists.
    pub graph: GraphInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum AchievementRef {
    Known {
        id: u32,
        text: String,
        hint: Option<String>,
        icon_url: Option<String>,
    },
    /// In the save but not in the catalog: a patch newer than the file.
    Unknown { slot: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum GraphInfo {
    Stub,
    Computed {
        available_now: bool,
        blocked_by: u32,
        fan_out: u32,
        steps_missing: u32,
    },
}

/// `catalog::Origin` doesn't cross the IPC boundary: this is its view, like `ItemKindView` for `ItemKind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OriginView {
    Rebirth,
    Afterbirth,
    AfterbirthPlus,
    Repentance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockView {
    pub nodes: Vec<UnlockNode>,
    pub totals: UnlockTotals,
    pub diagnostics: Vec<UnlockDiagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockTotals {
    pub slots: u32,
    pub done: u32,
    pub known: u32,
    pub unknown: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NextSteps {
    pub steps: Vec<UnlockNode>,
    pub basis: StepsBasis,
}

/// What the steps are ordered by. A fieldless enum: on the wire it's `"stub"` or
/// `"fanOut"`, not a tagged object — the same rule as `ItemKindView` and `OriginView`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StepsBasis {
    Stub,
    FanOut,
}

/// A goal as the UI sees it: the saved key plus whatever the current catalog knows
/// about it. `target: None` means "not resolvable right now" — game not installed, or
/// an id a patch has removed: the goal stays visible and is removed by its `key`,
/// which is the only thing the database holds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalView {
    pub id: GoalId,
    pub key: TargetKey,
    pub target: Option<UnlockTarget>,
    pub created_unix: i64,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum PlanDiagnostic {
    /// The database failed to open, and why (text of our own, never SQLite's):
    /// "newer version" is the only case the user can act on, and it must be said.
    StoreUnavailable { reason: String },
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum PlanExpansion {
    Stub,
    Computed { steps: Vec<PlanStep> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanStep {
    pub goal: GoalId,
    pub node: UnlockNode,
    pub done: bool,
}

use catalog::{AchievementId, BossId, Catalog, ChallengeId, CharacterId, ItemId, Origin, Unlock};

use crate::catalog_view::{item_kind, kind_view};
use crate::resources::data_url;

/// The Unlock view: one node per slot 1..=N of section 1 of the save. `flags[i]` is
/// slot i; slot 0 is unused (the `slot[id]` mapping, verified on 2026-09-05: 169 items
/// out of 171 seen with the achievement done).
///
/// `flags: None` means "section 1 wasn't read", and it is not the same thing as a save
/// with no achievements: flattening the two cases would make the view claim the catalog
/// has 638 more achievements than the file, which is false. `Some(&[])` stays the
/// degenerate save, with its own diagnostic.
pub fn unlock_view(
    catalog: Option<&Catalog>,
    flags: Option<&[bool]>,
    mut icon: impl FnMut(&str) -> Option<Vec<u8>>,
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
                    .map(|u| target_of(c, u, &mut icon))
                    .collect();
                let origin = first_item_origin(c, c.unlocks(a.id));
                (
                    AchievementRef::Known {
                        id: a.id.0,
                        text: a.text.clone(),
                        hint: a.unlock_condition.clone(),
                        icon_url: icon(&a.sprite.path).map(|png| data_url(&png)),
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
        nodes.push(UnlockNode {
            achievement: achievement_ref,
            done: flag,
            unlocks,
            origin,
            graph: GraphInfo::Stub,
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
    icon: &mut impl FnMut(&str) -> Option<Vec<u8>>,
) -> Option<UnlockTarget> {
    let english = catalog::Language::English;
    let mut rewards = Vec::new();
    let (name, icon_url) = match *key {
        TargetKey::Item { item_kind: k, id } => {
            let i = c.item(item_kind(k), ItemId(id))?;
            (
                c.text(&i.name, english).to_string(),
                icon(&i.sprite.path).map(|png| data_url(&png)),
            )
        }
        TargetKey::Character { id } => (
            c.text(&c.character(CharacterId(id))?.name, english)
                .to_string(),
            None,
        ),
        TargetKey::Boss { id } => (c.boss(BossId(id))?.name.clone(), None),
        TargetKey::Challenge { id } => {
            let ch = c.challenge(ChallengeId(id))?;
            rewards = ch.rewards.iter().map(|a| a.0).collect();
            (ch.name.clone(), None)
        }
    };
    Some(key.view(name, icon_url, rewards))
}

/// A catalog edge as the UI sees it. The edge is born from the catalog that resolves it
/// (`build_unlocks` constructs it straight from the entities themselves), so the key is
/// always there; if one day it weren't, the row stays nameless instead of vanishing.
pub fn target_of(
    c: &Catalog,
    u: &Unlock,
    icon: &mut impl FnMut(&str) -> Option<Vec<u8>>,
) -> UnlockTarget {
    let key = key_of(u);
    resolve_target(c, &key, icon).unwrap_or_else(|| key.view(String::new(), None, Vec::new()))
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

fn origin_view(o: Origin) -> OriginView {
    match o {
        Origin::Rebirth => OriginView::Rebirth,
        Origin::Afterbirth => OriginView::Afterbirth,
        Origin::AfterbirthPlus => OriginView::AfterbirthPlus,
        Origin::Repentance => OriginView::Repentance,
    }
}

/// Without a graph: the first `STEPS` not done, in slot order. `basis: Stub` declares it.
pub fn next_steps(view: &UnlockView) -> NextSteps {
    NextSteps {
        steps: view
            .nodes
            .iter()
            .filter(|n| !n.done)
            .take(STEPS)
            .cloned()
            .collect(),
        basis: StepsBasis::Stub,
    }
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
    goals: Vec<Goal>,
    unreadable: Vec<GoalId>,
    store_unavailable: Option<String>,
    mut icon: impl FnMut(&str) -> Option<Vec<u8>>,
) -> PlanView {
    let store_available = store_unavailable.is_none();
    let mut unresolved = Vec::new();
    let goals: Vec<GoalView> = goals
        .into_iter()
        .map(|g| {
            let target = catalog.and_then(|c| resolve_target(c, &g.target, &mut icon));
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
