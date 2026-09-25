//! A view derived from the save, kept for as long as the save it was derived from (card #80,
//! R10).
//!
//! Live asks for the graph's answer every time the watcher reports a line — every two seconds
//! while a run is played — and the graph's answer is 642 nodes evaluated against the profile.
//! The profile moves when the game writes the `.dat`, which is at the end of a run and not on
//! every line. `SaveCache` already knows when that happens: it hands out a new `Arc` exactly
//! when it read the file again. So the key is that `Arc`'s identity, and nothing is compared
//! by content.

use std::sync::{Arc, Mutex};

struct Entry<S, T> {
    // Held, not only compared: while the entry keeps it alive, no other save can be given its
    // address, so `ptr_eq` cannot mistake a new save for the old one.
    source: Arc<S>,
    value: Arc<T>,
}

/// One slot, like `SaveCache`: one active profile, so one save worth deriving from.
pub struct PerSave<S, T>(Mutex<Option<Entry<S, T>>>);

impl<S, T> Default for PerSave<S, T> {
    fn default() -> Self {
        PerSave(Mutex::new(None))
    }
}

impl<S, T> PerSave<S, T> {
    /// The value derived from `source`, built only when `source` is not the save the last
    /// value came from. A failed build is not remembered: the next call builds again.
    pub fn get<E>(
        &self,
        source: &Arc<S>,
        build: impl FnOnce() -> Result<T, E>,
    ) -> Result<Arc<T>, E> {
        // Poisoned: another command panicked while holding it. Build again; the value was
        // never the problem.
        let mut slot = self.0.lock().unwrap_or_else(|e| {
            let mut slot = e.into_inner();
            *slot = None;
            slot
        });
        if let Some(entry) = slot.as_ref().filter(|e| Arc::ptr_eq(&e.source, source)) {
            return Ok(Arc::clone(&entry.value));
        }
        let value = Arc::new(build()?);
        *slot = Some(Entry {
            source: Arc::clone(source),
            value: Arc::clone(&value),
        });
        Ok(value)
    }
}
