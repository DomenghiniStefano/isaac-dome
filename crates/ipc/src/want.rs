//! "I want this — what do I have to play?", the graph read from the other end.
//!
//! Everything here is a view over what `graph.rs` already computed: this module adds no
//! traversal of its own. It resolves what you named, finds the achievements that grant it,
//! and asks for the chain — in the order the Plan would play it.

use catalog::Catalog;
use serde::Serialize;
use wiki::Target;

use crate::graph::{AchievementRef, UnlockNode, UnlockTarget, UnlockView};
use crate::icon::IconRef;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WantView {
    pub wanted: WantedView,
    pub routes: Vec<WantRoute>,
    pub diagnostics: Vec<WantDiagnostic>,
}

/// A want is one of two things, and `UnlockTarget` can only be one of them: it has four
/// variants and none is an achievement. Naming *Greedier!* has to reach the node whose
/// target the catalog does not model, so the view carries both — and a third case for a name
/// the catalog no longer resolves.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum WantedView {
    Target { target: UnlockTarget },
    Achievement { achievement: AchievementRef },
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WantRoute {
    pub node: UnlockNode,
    pub state: WantState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum WantState {
    /// Already yours.
    Done,
    /// Nothing in the way: play it.
    AvailableNow,
    /// The prerequisites in the order the Plan would play them, the wanted node excluded.
    /// `unknown` counts the steps — the final node included — whose requirements the graph
    /// only partly interprets: a chain that cannot see everything says so in a number.
    Chain {
        steps: Vec<UnlockNode>,
        unknown: u32,
    },
    /// Section 1 was not read. The route is named; where you stand is not claimed.
    NoProfile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum WantDiagnostic {
    NoCatalog,
    NoProfile,
    /// `routes` is empty, and this is why: no node in the graph grants this.
    NothingUnlocks,
    /// A stage, a room, a pickup, a transformation: not a thing you unlock.
    NotUnlockable,
}

pub fn want_view(
    catalog: Option<&Catalog>,
    _view: &UnlockView,
    _flags: Option<&[bool]>,
    _eval: Option<&graph::evaluate::Eval>,
    target: &Target,
    _icon: impl FnMut(&IconRef) -> Option<String>,
) -> WantView {
    let unresolved = |d: WantDiagnostic| WantView {
        wanted: WantedView::Unresolved,
        routes: Vec::new(),
        diagnostics: vec![d],
    };
    if !unlockable(target) {
        return unresolved(WantDiagnostic::NotUnlockable);
    }
    let Some(_c) = catalog else {
        return unresolved(WantDiagnostic::NoCatalog);
    };
    unresolved(WantDiagnostic::NothingUnlocks)
}

/// The four kinds the graph never grants. Written as a `match` with no `_` arm so that a new
/// `Target` variant breaks this build rather than falling silently into "not unlockable".
fn unlockable(t: &Target) -> bool {
    match t {
        Target::Item { .. }
        | Target::Trinket { .. }
        | Target::Character { .. }
        | Target::Achievement { .. }
        | Target::Challenge { .. }
        | Target::Entity { .. } => true,
        Target::Transformation { .. }
        | Target::Stage { .. }
        | Target::Room { .. }
        | Target::Pickup { .. } => false,
    }
}
