//! Entry points that exist only so tests can reach a shape the public API doesn't build.
//!
//! One module per crate, and nothing test-only anywhere else in the public surface: a name
//! in the crate's `pub use` list says "call me", which is the opposite of what these mean.
//! Nothing outside a `tests/` target may call them.

use crate::build::Graph;
use crate::model::Requirement;

/// A graph straight from edges, for tests on the walk that don't need a catalog.
pub fn from_edges(edges: &[(u32, &[u32])], unknown: &[(u32, &[&str])]) -> Graph {
    Graph::from_edges(edges, unknown)
}

/// A graph whose nodes carry requirements directly, for tests about evaluation rather than
/// about building. Prerequisites stay empty: these nodes are held by the profile, not by
/// other achievements.
pub fn from_requirements(rows: &[(u32, Vec<Requirement>)]) -> Graph {
    Graph::from_requirements(rows)
}
