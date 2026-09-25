//! A view derived from the save, built again only when the save is (card #80, R10).
//!
//! The counter is a call count, the way `save_cache.rs` counts `load`: what the cache exists to
//! avoid is the build.

use std::cell::Cell;
use std::sync::Arc;

use ipc::{IpcError, PerSave};

#[test]
fn the_same_save_is_built_once() {
    let cache = PerSave::<String, u32>::default();
    let save = Arc::new("save".to_string());
    let builds = Cell::new(0);
    let build = || {
        builds.set(builds.get() + 1);
        Ok::<_, IpcError>(7)
    };

    assert_eq!(*cache.get(&save, build).unwrap(), 7);
    assert_eq!(*cache.get(&save, build).unwrap(), 7);
    assert_eq!(builds.get(), 1);
}

#[test]
fn a_save_read_again_is_built_again_even_when_it_reads_the_same() {
    // Identity, not content: `SaveCache` hands out a new `Arc` exactly when it read the file
    // again, which is the only moment the profile can have moved. Comparing contents would
    // pay for reading a whole save to save a build.
    let cache = PerSave::<String, u32>::default();
    let builds = Cell::new(0);
    let build = || {
        builds.set(builds.get() + 1);
        Ok::<_, IpcError>(builds.get())
    };

    assert_eq!(*cache.get(&Arc::new("save".to_string()), build).unwrap(), 1);
    assert_eq!(*cache.get(&Arc::new("save".to_string()), build).unwrap(), 2);
}

#[test]
fn a_failed_build_is_not_remembered() {
    // The rule this repo writes down for "the game isn't installed": a failure is asked again.
    let cache = PerSave::<String, u32>::default();
    let save = Arc::new("save".to_string());

    assert!(matches!(
        cache.get(&save, || Err(IpcError::CatalogUnavailable)),
        Err(IpcError::CatalogUnavailable)
    ));
    assert_eq!(*cache.get(&save, || Ok::<_, IpcError>(3)).unwrap(), 3);
}
