//! Entry points that exist only so tests can reach a shape the public API doesn't build.
//!
//! One module per crate, and nothing test-only anywhere else in the public surface: a name
//! in the crate's `pub use` list says "call me", which is the opposite of what these mean.
//! Nothing outside a `tests/` target may call them.

use std::collections::BTreeMap;

use catalog::Catalog;
use wiki::Target;

use crate::search::{self, ProgressMark, SaveFlags, SearchIndex};

pub use crate::search::Doc;

/// The two halves a query is built on, reachable from the integration test: a test that
/// could only see the ranked answer would say nothing about them.
pub fn documents(index: &SearchIndex, catalog: Option<&Catalog>) -> BTreeMap<Target, Doc> {
    search::documents(index, catalog)
}

pub fn progress(target: &Target, flags: Option<SaveFlags<'_>>) -> ProgressMark {
    search::progress(target, flags)
}

/// A view built straight from its nodes. What `next_steps` sorts by — how much a node opens,
/// and how far a tally still is — is a property of the nodes themselves; building a catalog
/// and a graph to express "fan-out 9" would test those instead, and hide the rule under them.
pub fn unlock_view_of(nodes: Vec<crate::UnlockNode>) -> crate::UnlockView {
    crate::UnlockView {
        nodes,
        totals: crate::UnlockTotals {
            slots: 0,
            done: 0,
            known: 0,
            unknown: 0,
        },
        diagnostics: vec![],
    }
}

/// A state resting at a phase. The app only ever starts at `Idle` and moves through the
/// calls on `UpdateState`, so this exists for the tests that have to enter the machine
/// somewhere else — "a check is allowed from here", "a late chunk changes nothing there".
pub fn update_at(phase: crate::UpdatePhase) -> crate::UpdateState {
    crate::UpdateState::at(phase)
}
