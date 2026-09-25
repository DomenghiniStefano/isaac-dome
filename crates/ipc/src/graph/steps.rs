//! The steps worth playing tonight: a filter over the Unlock view.

use std::collections::BTreeSet;

use super::types::{
    AchievementRef, GraphInfo, GraphViews, NextSteps, RequirementView, StepsBasis, StepsSection,
    UnlockNode, UnlockView,
};

/// How many "next steps" the screen shows. Presentation, not domain.
pub const STEPS: usize = 5;

/// The pair, built from one view: the steps are the filter over exactly the list that
/// travels beside them. `queued` is what the plan queue already holds, which the steps leave
/// out — see `next_steps`.
pub fn graph_views(unlock: UnlockView, queued: &BTreeSet<u32>) -> GraphViews {
    let steps = next_steps(&unlock, queued);
    GraphViews { unlock, steps }
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
