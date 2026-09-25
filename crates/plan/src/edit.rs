//! Adding a wish and removing one. The chain arrives from the caller, already read from
//! the graph: this crate never asks the graph anything, which is what lets it be tested
//! without a catalog.

use crate::model::{Queue, Row};
use crate::order::Dependencies;
use graph::AchievementId;

impl Queue {
    /// Adds `achievement` as a wish, with its missing prerequisites before it.
    ///
    /// A step already queued keeps its place and gains an origin; a wish already queued is
    /// left exactly where it is — asking twice is not a reason to undo an arrangement you
    /// made by hand.
    pub fn enqueue(
        &mut self,
        achievement: AchievementId,
        chain: &[AchievementId],
        deps: &impl Dependencies,
    ) {
        let already_queued = self.position(achievement).is_some();
        let rows = playable_order(chain, deps)
            .into_iter()
            .fold(self.rows().to_vec(), |rows, step| {
                with_step(rows, step, achievement)
            });
        *self = Queue::from_rows(with_wish(rows, achievement));
        // Moving the wish onto its own position runs the repair, which pulls the steps
        // just appended above it and leaves everything else alone. A wish that was already
        // in the queue keeps the position you gave it: the repair is about the constraint,
        // not about re-deciding your order.
        let at = self.position(achievement).unwrap_or(0);
        if !already_queued || !chain.is_empty() {
            self.move_row(achievement, at, deps);
        }
    }

    /// Removes a wish. Its steps go only if nothing else keeps them: another wish that
    /// needs them, or your having asked for them yourself.
    pub fn remove(&mut self, achievement: AchievementId) {
        let rows = self
            .rows()
            .iter()
            .filter(|r| r.achievement != achievement)
            .map(|r| Row {
                origins: r
                    .origins
                    .iter()
                    .copied()
                    .filter(|o| *o != achievement)
                    .collect(),
                ..r.clone()
            })
            .filter(|r| !r.is_orphan())
            .collect();
        *self = Queue::from_rows(rows);
    }
}

/// `step` in the queue on behalf of `origin`: a row already there gains the origin once, a
/// missing one is appended as a step that only `origin` wants.
fn with_step(mut rows: Vec<Row>, step: AchievementId, origin: AchievementId) -> Vec<Row> {
    match rows.iter_mut().find(|r| r.achievement == step) {
        Some(r) if r.origins.contains(&origin) => {}
        Some(r) => r.origins.push(origin),
        None => rows.push(Row {
            achievement: step,
            wanted: false,
            origins: vec![origin],
        }),
    }
    rows
}

/// `achievement` wanted for itself: a row already there is marked, a missing one appended.
fn with_wish(mut rows: Vec<Row>, achievement: AchievementId) -> Vec<Row> {
    match rows.iter_mut().find(|r| r.achievement == achievement) {
        Some(r) => r.wanted = true,
        None => rows.push(Row {
            achievement,
            wanted: true,
            origins: Vec::new(),
        }),
    }
    rows
}

/// The chain in an order it can be played in (card #80, P4). `missing_chain` promises only
/// increasing ids, and a step with a lower id can need one with a higher id: appended as it
/// arrived, that step sat above its own prerequisite. Each step goes right before the first
/// step already placed that needs it — enough, because the relation is transitive — and steps
/// that need nothing of each other keep the order they came in.
fn playable_order(chain: &[AchievementId], deps: &impl Dependencies) -> Vec<AchievementId> {
    chain
        .iter()
        .fold(Vec::with_capacity(chain.len()), |mut ordered, &step| {
            match ordered
                .iter()
                .position(|&placed| deps.requires(placed, step))
            {
                Some(before) => ordered.insert(before, step),
                None => ordered.push(step),
            }
            ordered
        })
}
