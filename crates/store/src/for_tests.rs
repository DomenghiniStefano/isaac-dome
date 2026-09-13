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

/// Writes a row that will not parse as an event, for the test that says an unreadable row is
/// counted rather than hiding the run around it. There is no other way to reach that state.
pub fn corrupt_event(store: &Store, source_id: i64, seq: i64, raw: &str) -> Result<(), StoreError> {
    store.replace_event_json(source_id, seq, raw)
}
