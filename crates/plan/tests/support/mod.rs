#![allow(dead_code)] // one test binary today; the module is shaped for the next one too

//! Shared setup for the real-data tests. Every function says on stderr which slice of the
//! real domain it ran on, or why it skipped: a green suite that skipped everything is the
//! failure mode this discipline exists to prevent.

use graph::build::Graph;

// The real catalog, graph and profile come from `graph`'s own test support, compiled into this
// test binary from its one source rather than copied, so a change to how the real data is found
// or skipped reaches both suites. Everything it reaches is in this crate's dev-dependencies too.
#[path = "../../../graph/tests/support/mod.rs"]
mod graph_support;

pub use graph_support::real_graph_and_flags;

/// "a requires b" over the graph's transitive prerequisites, with the chains computed once
/// for the rows involved. The same shape as `ipc::queue::GraphDeps`, restated here because
/// `ipc` depends on this crate and not the other way round.
pub struct GraphDeps {
    chains: std::collections::BTreeMap<
        graph::AchievementId,
        std::collections::BTreeSet<graph::AchievementId>,
    >,
}

impl GraphDeps {
    pub fn new(g: &Graph, flags: Option<&[bool]>, rows: &[graph::AchievementId]) -> GraphDeps {
        GraphDeps {
            chains: rows
                .iter()
                .map(|a| {
                    (
                        *a,
                        g.missing_chain(*a, &graph::evaluate::FlagsOnly(flags))
                            .into_iter()
                            .collect(),
                    )
                })
                .collect(),
        }
    }
}

impl plan::Dependencies for GraphDeps {
    fn requires(&self, a: graph::AchievementId, b: graph::AchievementId) -> bool {
        self.chains.get(&a).is_some_and(|c| c.contains(&b))
    }
}
