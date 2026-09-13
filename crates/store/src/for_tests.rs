//! Entry points that exist only so tests can reach a shape the public API doesn't build.
//!
//! One module per crate, and nothing test-only anywhere else in the public surface: a name
//! in the crate's `pub use` list says "call me", which is the opposite of what these mean.
//! Nothing outside a `tests/` target may call them.

use crate::{Store, StoreError};

/// Writes a queue document straight in, for the test that an unreadable queue is declared
/// rather than flattened to an empty one. There is no other way to reach that state through
/// the public API, which is the point of the API.
pub fn corrupt_queue(store: &Store, raw: &str) -> Result<(), StoreError> {
    store.write_queue_json(raw)
}
