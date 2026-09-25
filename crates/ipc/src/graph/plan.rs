//! The plan: the saved goals resolved against the current catalog.

use catalog::Catalog;
use wiki::Dataset;

use super::target::resolve_target;
use super::types::{GoalView, PlanDiagnostic, PlanView};
use crate::goals::{Goal, GoalId};
use crate::icon::IconRef;
use crate::target_sprite::BossKeys;

/// The plan: the saved goals resolved against the current catalog. The database only keeps
/// the keys, so name and icon are born here: a goal saved when the game wasn't there shows its
/// name as soon as the game is. `store_unavailable` is the single source: `store_available`
/// and the `StoreUnavailable` diagnostic both derive from it and can never contradict each
/// other.
///
/// The diagnostics come out in this order: the store, the catalog, the unreadable
/// rows, the unresolved keys — from the problem that explains the most to the one that
/// explains the least.
pub fn plan_view(
    catalog: Option<&Catalog>,
    bosses: &BossKeys,
    dataset: Option<&Dataset>,
    goals: Vec<Goal>,
    unreadable: Vec<GoalId>,
    store_unavailable: Option<crate::StoreReason>,
    mut icon: impl FnMut(&IconRef) -> Option<String>,
) -> PlanView {
    let store_available = store_unavailable.is_none();
    let goals: Vec<GoalView> = goals
        .into_iter()
        .map(|g| {
            let target =
                catalog.and_then(|c| resolve_target(c, bosses, &g.target, dataset, &mut icon));
            goal_view(g, target)
        })
        .collect();
    // Without a catalog nothing resolves, and `NoCatalog` already says so: flagging every goal
    // would just repeat the same news one row at a time.
    let unresolved = goals
        .iter()
        .filter(|g| catalog.is_some() && g.target.is_none())
        .map(|g| PlanDiagnostic::UnresolvedGoal { id: g.id.clone() });
    let diagnostics = store_unavailable
        .into_iter()
        .map(|reason| PlanDiagnostic::StoreUnavailable { reason })
        .chain(catalog.is_none().then_some(PlanDiagnostic::NoCatalog))
        .chain(
            unreadable
                .into_iter()
                .map(|id| PlanDiagnostic::UnreadableGoal { id }),
        )
        .chain(unresolved)
        .collect();
    PlanView {
        goals,
        diagnostics,
        store_available,
    }
}

fn goal_view(g: Goal, target: Option<crate::goals::UnlockTarget>) -> GoalView {
    GoalView {
        id: g.id,
        key: g.target,
        target,
        created_unix: g.created_unix,
        note: g.note,
    }
}
