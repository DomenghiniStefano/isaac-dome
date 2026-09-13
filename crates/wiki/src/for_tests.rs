//! Entry points that exist only so tests can reach a shape the public API doesn't build.
//!
//! One module per crate, and nothing test-only anywhere else in the public surface: a name
//! in the crate's `pub use` list says "call me", which is the opposite of what these mean.
//! Nothing outside a `tests/` target may call them.

use crate::Dataset;

/// An empty dataset for tests in the crates downstream: they need a `Dataset` to fill one
/// field of, and building `meta` by hand in every test would copy that block around. Not for
/// production code — `embedded()` and `from_json` are the ways in.
pub fn empty_dataset() -> Dataset {
    Dataset::empty()
}
