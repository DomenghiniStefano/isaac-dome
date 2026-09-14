//! N8's expensive half: the policy that decides when the profile may be reused, with the
//! counter the item asks for. The cache is here and not in `crates/app` because this is the
//! part with a return value worth checking — the wiring around it is not.

use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use ipc::{profile_id, IpcError, SaveCache};

fn path() -> PathBuf {
    PathBuf::from("rep+persistentgamedata1.dat")
}

/// A stat that always answers the same instant, and one that can be moved.
fn at(secs: u64) -> Option<SystemTime> {
    Some(SystemTime::UNIX_EPOCH + Duration::from_secs(secs))
}

#[test]
fn two_screens_on_one_profile_read_the_save_once() {
    let cache: SaveCache<u32> = SaveCache::default();
    let id = profile_id(&path());
    let loads = Cell::new(0);
    let load = |_: &Path| {
        loads.set(loads.get() + 1);
        Ok(7)
    };
    let stat = |_: &Path| at(100);
    let resolve = || Ok((id.clone(), path()));

    let (_, first) = cache.get(Some(&id), stat, resolve, load).unwrap();
    let (_, second) = cache
        .get(Some(&id), stat, || Ok((id.clone(), path())), load)
        .unwrap();

    assert_eq!(*first, 7);
    assert_eq!(*second, 7);
    assert_eq!(loads.get(), 1, "the second screen reused the first's read");
}

#[test]
fn a_save_written_while_the_app_is_open_is_read_again() {
    let cache: SaveCache<u32> = SaveCache::default();
    let id = profile_id(&path());
    let loads = Cell::new(0);
    let load = |_: &Path| {
        loads.set(loads.get() + 1);
        Ok(loads.get())
    };
    let clock = Cell::new(100u64);
    let stat = |_: &Path| at(clock.get());

    let _ = cache
        .get(Some(&id), stat, || Ok((id.clone(), path())), load)
        .unwrap();
    clock.set(200);
    let (_, after) = cache
        .get(Some(&id), stat, || Ok((id.clone(), path())), load)
        .unwrap();

    assert_eq!(loads.get(), 2, "the file changed under it");
    assert_eq!(*after, 2, "and the value is the new read, not the old one");
}

#[test]
fn a_save_that_disappears_is_not_answered_from_memory() {
    let cache: SaveCache<u32> = SaveCache::default();
    let id = profile_id(&path());
    let loads = Cell::new(0);
    let load = |_: &Path| {
        loads.set(loads.get() + 1);
        Ok(1)
    };
    let gone = Cell::new(false);
    let stat = |_: &Path| if gone.get() { None } else { at(100) };

    let _ = cache
        .get(Some(&id), stat, || Ok((id.clone(), path())), load)
        .unwrap();
    gone.set(true);
    let _ = cache.get(Some(&id), stat, || Ok((id.clone(), path())), load);

    assert_eq!(loads.get(), 2, "a file we can no longer stat is not fresh");
}

#[test]
fn choosing_another_profile_reads_that_one() {
    let cache: SaveCache<u32> = SaveCache::default();
    let first = profile_id(&path());
    let second = profile_id(Path::new("rep+persistentgamedata2.dat"));
    let loads = Cell::new(0);
    let load = |_: &Path| {
        loads.set(loads.get() + 1);
        Ok(loads.get())
    };
    let stat = |_: &Path| at(100);

    let _ = cache
        .get(Some(&first), stat, || Ok((first.clone(), path())), load)
        .unwrap();
    let (id, value) = cache
        .get(
            Some(&second),
            stat,
            || Ok((second.clone(), PathBuf::from("rep+persistentgamedata2.dat"))),
            load,
        )
        .unwrap();

    assert_eq!(id, second);
    assert_eq!(*value, 2);
    assert_eq!(loads.get(), 2);
}

/// With no profile named in the settings the active one is whatever is on disk, and what is
/// on disk is the thing that changes: a second save appearing has to be seen. The rule the
/// repo already writes down for "the game isn't installed" — an expected absence is never
/// cached — with the choice in place of the absence.
#[test]
fn without_a_named_profile_nothing_is_reused() {
    let cache: SaveCache<u32> = SaveCache::default();
    let id = profile_id(&path());
    let loads = Cell::new(0);
    let load = |_: &Path| {
        loads.set(loads.get() + 1);
        Ok(1)
    };
    let stat = |_: &Path| at(100);

    let _ = cache
        .get(None, stat, || Ok((id.clone(), path())), load)
        .unwrap();
    let _ = cache
        .get(None, stat, || Ok((id.clone(), path())), load)
        .unwrap();

    assert_eq!(loads.get(), 2, "the resolution is redone every time");
}

#[test]
fn a_failure_is_not_remembered() {
    let cache: SaveCache<u32> = SaveCache::default();
    let id = profile_id(&path());
    let stat = |_: &Path| at(100);
    let first = cache.get(
        Some(&id),
        stat,
        || Ok((id.clone(), path())),
        |_: &Path| Err(IpcError::NoActiveProfile),
    );
    assert!(first.is_err());

    let loads = Cell::new(0);
    let (_, value) = cache
        .get(
            Some(&id),
            stat,
            || Ok((id.clone(), path())),
            |_: &Path| {
                loads.set(loads.get() + 1);
                Ok(3)
            },
        )
        .unwrap();
    assert_eq!(*value, 3, "the next call tried again instead of failing");
    assert_eq!(loads.get(), 1);
}
