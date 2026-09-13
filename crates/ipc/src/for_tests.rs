//! Entry points that exist only so tests can reach a shape the public API doesn't build.
//!
//! One module per crate, and nothing test-only anywhere else in the public surface: a name
//! in the crate's `pub use` list says "call me", which is the opposite of what these mean.
//! Nothing outside a `tests/` target may call them.

use std::collections::BTreeMap;

use catalog::Catalog;
use wiki::Target;

use crate::search::{self, SaveFlags, SearchIndex};

pub use crate::search::{Doc, ProgressMark};

/// The two halves a query is built on, reachable from the integration test: a test that
/// could only see the ranked answer would say nothing about them.
pub fn documents(index: &SearchIndex, catalog: Option<&Catalog>) -> BTreeMap<Target, Doc> {
    search::documents(index, catalog)
}

pub fn progress(target: &Target, flags: Option<SaveFlags<'_>>) -> ProgressMark {
    search::progress(target, flags)
}
