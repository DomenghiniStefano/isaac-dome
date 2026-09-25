//! The profile, kept between commands — the half of N8 that needs a rule rather than a
//! refactor.
//!
//! Every command that shows progress opens the save for itself: the settings file, a walk
//! of the Steam libraries, the `.dat` read whole and parsed. One screen is several commands,
//! and they all read the same file in the same second. What makes this a cache and not a
//! `OnceLock` is that **the file changes while the app is open**: the game writes it at the
//! end of a run, and a reader that remembers the first read shows a profile that no longer
//! exists.
//!
//! So the rule is three lines and each one has a reason:
//!
//! - a reuse happens only when the **same profile** is asked for, by the id the settings
//!   name — no name means the active profile is whatever is on disk, and what is on disk is
//!   exactly the thing that changes;
//! - and only when the file's **modified time is the one the cached read saw**, which also
//!   covers the file disappearing: no time is not the same time;
//! - and a **failure is never remembered**, the rule this repo already writes down for "the
//!   game isn't installed".
//!
//! The I/O arrives as closures, so the policy is testable without a disk and the counter
//! N8 asks for is a call count — `crates/ipc/tests/save_cache.rs`.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use crate::{IpcError, ProfileId};

struct Entry<T> {
    profile: ProfileId,
    path: PathBuf,
    modified: Option<SystemTime>,
    value: Arc<T>,
}

/// One slot: the app has one active profile, so a second entry would be a second profile
/// nobody is looking at.
pub struct SaveCache<T>(Mutex<Option<Entry<T>>>);

impl<T> Default for SaveCache<T> {
    fn default() -> Self {
        SaveCache(Mutex::new(None))
    }
}

impl<T> SaveCache<T> {
    /// The active profile and its save, read again only when the rules above say the last
    /// read cannot stand.
    ///
    /// `chosen` is what the settings name; `stat` the file's modified time now; `resolve`
    /// finds which file the active profile is, and `load` opens and parses it. The last two
    /// run only on a miss, which is what keeps the walk of the Steam libraries out of every
    /// command.
    pub fn get(
        &self,
        chosen: Option<&ProfileId>,
        stat: impl Fn(&Path) -> Option<SystemTime>,
        resolve: impl FnOnce() -> Result<(ProfileId, PathBuf), IpcError>,
        load: impl FnOnce(&Path) -> Result<T, IpcError>,
    ) -> Result<(ProfileId, Arc<T>), IpcError> {
        // A poisoned lock means another command panicked mid-read. The answer is to read
        // again, not to fail: the value behind it was never the problem.
        let mut slot = self.0.lock().unwrap_or_else(|e| {
            let mut slot = e.into_inner();
            *slot = None;
            slot
        });
        if let (Some(chosen), Some(entry)) = (chosen, slot.as_ref()) {
            if entry.profile == *chosen && stat(&entry.path) == entry.modified {
                return Ok((entry.profile.clone(), Arc::clone(&entry.value)));
            }
        }
        let (profile, path) = resolve()?;
        // The time is read **before** the load (card #80, item 05). A file written during the
        // read then carries a newer time than the one kept here, so the next command sees it
        // moved and reads it again rather than trusting a value that saw half of each version.
        // Read after, it was the new time on the old content, and passed as fresh.
        let modified = stat(&path);
        let value = Arc::new(load(&path)?);
        *slot = Some(Entry {
            profile: profile.clone(),
            path,
            modified,
            value: Arc::clone(&value),
        });
        Ok((profile, value))
    }
}
