//! Moving a row, and repairing the order around it.
//!
//! The queue is yours to order, and the graph is the one thing it may not contradict. A
//! move is never refused: the rows that must yield are moved, and the moved row lands
//! where you asked whenever that is a position at all.

use crate::model::{Queue, Row};
use graph::AchievementId;

/// "a requires b", transitively. The queue asks this and nothing else, which keeps this
/// crate independent of how the graph computes it — and lets the tests state the relation
/// they mean instead of deriving it.
pub trait Dependencies {
    fn requires(&self, a: AchievementId, b: AchievementId) -> bool;
}

impl Queue {
    /// Moves a row right below another one — `None` for the top.
    ///
    /// The way a screen names a drop: by the row it lands under. An index would not do, since
    /// the view leaves completed and unresolved rows out and its positions are not the file's.
    /// An `after` that isn't queued, or is the moved row itself, leaves the queue as it is: the
    /// caller's picture was stale, and the view it gets back is the truth.
    pub fn move_after(
        &mut self,
        achievement: AchievementId,
        after: Option<AchievementId>,
        deps: &impl Dependencies,
    ) {
        let Some(from) = self.position(achievement) else {
            return;
        };
        let to = match after {
            None => 0,
            Some(a) if a == achievement => return,
            Some(a) => match self.position(a) {
                // A position once the moved row is out: one past the anchor.
                Some(i) if i < from => i + 1,
                Some(i) => i,
                None => return,
            },
        };
        self.move_row(achievement, to, deps);
    }

    /// Moves a row to `to` — a position in the queue once the row is taken out — and returns
    /// the index it landed at among the rows that didn't move with it.
    ///
    /// Rows that depend on it are dragged along, right below it; rows it depends on are a wall
    /// it stops under, and never move; everything else keeps its relative order. The landing
    /// is clamped between one past the last prerequisite and the end of the list.
    pub fn move_row(
        &mut self,
        achievement: AchievementId,
        to: usize,
        deps: &impl Dependencies,
    ) -> usize {
        let Some(from) = self.position(achievement) else {
            // Not in the queue: nothing to move, and not an error.
            return to.min(self.rows().len().saturating_sub(1));
        };
        let mut rows: Vec<Row> = self.rows().to_vec();
        let moved = rows.remove(from);

        // The two relations behave differently, and the asymmetry is the rule itself:
        //
        // **Dependents are dragged.** Move a prerequisite down and what needs it follows,
        // gathered right below it — the case this feature was asked for.
        //
        // **Prerequisites are a wall.** They are never moved: one already above the row is
        // fine where you put it, and hauling it into a block would reorder rows you had
        // arranged by hand. They only stop the row from rising past them.
        //
        // A row the graph can't compute answers `false` both ways, so it neither drags nor
        // walls: it stays exactly where it is.
        let mut dragged = Vec::new();
        let mut rest = Vec::new();
        // `to` counts the dragged rows too, and each one above it leaves with the moved row:
        // the target among the rows that stay is that much higher.
        let mut target = to;
        for (i, r) in rows.into_iter().enumerate() {
            if deps.requires(r.achievement, achievement) {
                if i < to {
                    target -= 1;
                }
                dragged.push(r);
            } else {
                rest.push(r);
            }
        }

        // The floor: one past the last prerequisite of the moved row left in the list. It
        // bounds the moved row only; the rows dragged with it can have prerequisites of their
        // own, and those are placed below.
        let floor = rest
            .iter()
            .rposition(|r| deps.requires(achievement, r.achievement))
            .map(|i| i + 1)
            .unwrap_or(0);
        let landed = target.clamp(floor, rest.len());

        // A dragged row lands right below the moved one — unless one of its **own** other
        // prerequisites is still further down, and then right below that one (card #80,
        // item 03: `[2, 1, 3]` with 3 needing both, moving 1 to the top used to put 3 above
        // 2). Walked in the dragged rows' own order, and each one looks at the rows already
        // placed below as well: a dragged row that needs another dragged row placed lower
        // follows it there.
        let mut tail = rest.split_off(landed);
        let mut out = rest;
        out.push(moved);
        for row in dragged {
            match tail
                .iter()
                .rposition(|r| deps.requires(row.achievement, r.achievement))
            {
                Some(last) => tail.insert(last + 1, row),
                None => out.push(row),
            }
        }
        out.extend(tail);
        *self = Queue::from_rows(out);
        landed
    }
}
