//! The wire types of the graph screens: the node, its requirements, the views and their
//! diagnostics.

use serde::{Deserialize, Serialize};
use wiki::Target;

use crate::catalog_view::ItemKindView;
use crate::goals::{GoalId, TargetKey, UnlockTarget};

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

/// The twelve columns, as a value on the wire: the layout's own enum, which serializes as a
/// bare camelCase string and is declared to TypeScript under this name. The graph's `MarkColumn` is the same type, so a requirement's column crosses as it is.
pub use core_save::Column as MarkColumnView;

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
        /// the 119 unlockable *now* — what the landing page draws from — for only 16.
        ///
        /// The game's words win where it has any: the wiki is the fallback, never a rewrite.
        condition: Option<String>,
        icon_url: Option<String>,
    },
    /// In the save but not in the catalog: a patch newer than the file.
    Unknown { slot: u32 },
}

/// What the graph knows about one node. A node that is `Partial` must never be drawn as
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

/// Both answers the graph's screens need, from one reading of the profile.
///
/// One answer and not two, because the steps are a filter over the list: two readings could
/// straddle a save written in between, and the steps would then disagree with the list they
/// filter. Each reading also rebuilds the whole pipeline behind it — the settings file, a walk
/// of the Steam libraries, the `.dat` read and parsed, 642 nodes and the evaluation.
///
/// It is not a cache: nothing is remembered between calls, and the steps stay a pure function
/// of the view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct GraphViews {
    pub unlock: UnlockView,
    pub steps: NextSteps,
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
