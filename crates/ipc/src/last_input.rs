//! A value built from an input, kept until the input changes (card #80, R10).
//!
//! For a derivation the pure crates ask for on every lookup — `boss_keys` is asked once per
//! boss in a graph view, and once per entity icon on every key pressed in the search — and
//! whose input is the same for the whole life of the process. The input is compared by
//! content, never by address: a catalog dropped and another built in the same place is a
//! different roster, and the tests build dozens.

use std::sync::{Arc, Mutex, PoisonError};

/// One slot: the last input and what was built from it.
pub(crate) struct LastInput<K, V>(Mutex<Option<(K, Arc<V>)>>);

impl<K, V> LastInput<K, V> {
    pub(crate) const fn new() -> Self {
        LastInput(Mutex::new(None))
    }

    /// The value built for the input `same` recognizes, or a new one from `build`, which
    /// returns the input it was built from so the next call can recognize it.
    pub(crate) fn get(&self, same: impl Fn(&K) -> bool, build: impl FnOnce() -> (K, V)) -> Arc<V> {
        // Poisoned: a panic mid-build elsewhere. The slot may hold the input or not; either
        // way what is in it was complete, since it is written in one assignment.
        let mut slot = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some((_, value)) = slot.as_ref().filter(|(input, _)| same(input)) {
            return Arc::clone(value);
        }
        let (input, value) = build();
        let value = Arc::new(value);
        *slot = Some((input, Arc::clone(&value)));
        value
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::LastInput;

    #[test]
    fn the_same_input_is_built_once() {
        let memo = LastInput::<u32, String>::new();
        let builds = Cell::new(0);
        let build = |n: u32| {
            builds.set(builds.get() + 1);
            (n, format!("built from {n}"))
        };

        assert_eq!(*memo.get(|k| *k == 1, || build(1)), "built from 1");
        assert_eq!(*memo.get(|k| *k == 1, || build(1)), "built from 1");
        assert_eq!(builds.get(), 1);
    }

    #[test]
    fn another_input_is_built_again_and_replaces_the_first() {
        let memo = LastInput::<u32, String>::new();
        let builds = Cell::new(0);
        let build = |n: u32| {
            builds.set(builds.get() + 1);
            (n, format!("built from {n}"))
        };

        memo.get(|k| *k == 1, || build(1));
        assert_eq!(*memo.get(|k| *k == 2, || build(2)), "built from 2");
        assert_eq!(*memo.get(|k| *k == 1, || build(1)), "built from 1");
        assert_eq!(builds.get(), 3);
    }
}
