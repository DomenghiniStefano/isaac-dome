//! Adding a wish and removing one. The chain arrives from the caller, already read from
//! the graph: this crate never asks the graph anything, which is what lets it be tested
//! without a catalog.

use crate::model::{Queue, Row};
use crate::order::Dependencies;

impl Queue {
    /// Adds `achievement` as a wish, with its missing prerequisites before it.
    ///
    /// A step already queued keeps its place and gains an origin; a wish already queued is
    /// left exactly where it is — asking twice is not a reason to undo an arrangement you
    /// made by hand.
    pub fn enqueue(&mut self, achievement: u32, chain: &[u32], deps: &impl Dependencies) {
        let already_queued = self.position(achievement).is_some();
        let mut rows = self.rows().to_vec();
        for step in chain {
            match rows.iter_mut().find(|r| r.achievement == *step) {
                Some(r) => {
                    if !r.origins.contains(&achievement) {
                        r.origins.push(achievement);
                    }
                }
                None => rows.push(Row {
                    achievement: *step,
                    wanted: false,
                    origins: vec![achievement],
                }),
            }
        }
        match rows.iter_mut().find(|r| r.achievement == achievement) {
            Some(r) => r.wanted = true,
            None => rows.push(Row {
                achievement,
                wanted: true,
                origins: Vec::new(),
            }),
        }
        *self = Queue::from_rows(rows);
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
    pub fn remove(&mut self, achievement: u32) {
        let mut rows = self.rows().to_vec();
        for r in rows.iter_mut() {
            r.origins.retain(|o| *o != achievement);
        }
        rows.retain(|r| r.achievement != achievement);
        rows.retain(|r| !r.is_orphan());
        *self = Queue::from_rows(rows);
    }
}
