//! Moving a row, and repairing the order around it.
//!
//! The queue is yours to order, and the graph is the one thing it may not contradict. A
//! move is never refused: the rows that must yield are moved, and the moved row lands
//! where you asked whenever that is a position at all.

use crate::model::{Queue, Row};

/// "a requires b", transitively. The queue asks this and nothing else, which keeps this
/// crate independent of how the graph computes it — and lets the tests state the relation
/// they mean instead of deriving it.
pub trait Dependencies {
    fn requires(&self, a: u32, b: u32) -> bool;
}

impl Queue {
    /// Moves a row and returns the index it actually landed at.
    ///
    /// Rows that depend on it gather immediately below; rows it depends on gather
    /// immediately above; everything else keeps its relative order. The landing index is
    /// clamped so the prerequisites have somewhere to be: dropping a row at the top with
    /// three prerequisites queued asks for three rows above position zero, which is not a
    /// position. Downward moves are never clamped — dependents can always be pushed
    /// further down.
    pub fn move_row(&mut self, achievement: u32, to: usize, deps: &impl Dependencies) -> usize {
        let Some(from) = self.position(achievement) else {
            // Not in the queue: nothing to move, and not an error.
            return to.min(self.rows().len().saturating_sub(1));
        };
        let mut rows: Vec<Row> = self.rows().to_vec();
        let moved = rows.remove(from);

        // A row the graph can't compute answers `false` both ways, so it lands in `free`:
        // it is never dragged and never drags.
        let (mut above, mut below, mut free) = (Vec::new(), Vec::new(), Vec::new());
        for r in rows {
            if deps.requires(achievement, r.achievement) {
                above.push(r);
            } else if deps.requires(r.achievement, achievement) {
                below.push(r);
            } else {
                free.push(r);
            }
        }

        let total = above.len() + below.len() + free.len();
        let landed = to.clamp(above.len(), total - below.len());
        let free_above = landed - above.len();

        let mut out = Vec::with_capacity(total + 1);
        out.append(&mut above);
        out.extend(free.drain(..free_above));
        out.push(moved);
        out.append(&mut free);
        out.append(&mut below);
        *self = Queue::from_rows(out);
        landed
    }
}
