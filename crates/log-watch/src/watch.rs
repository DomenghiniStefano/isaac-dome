//! `notify`, a debounce, and a callback. Everything worth checking is somewhere else.
//!
//! **The folder is watched, not the file.** `log.txt` is replaced on every launch of the game,
//! and a watch registered on the file follows the old handle — going deaf exactly when a new
//! launch starts, which is the one moment that matters.

use std::ffi::OsStr;
use std::path::Path;
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::time::{Duration, Instant};

use notify::{RecursiveMode, Watcher};
use run::Throttle;

use crate::WatchError;

/// How often a change is acted on at most. The game writes constantly — B8 counted 132 save
/// lines in one run — and every pass folds a source again.
const DEBOUNCE: Duration = Duration::from_secs(2);

/// Alive as long as the watch is. Dropping it ends the watch and, with it, the thread.
pub struct LogWatcher {
    _inner: notify::RecommendedWatcher,
}

/// Calls `on_change` when `log` may have changed: at once, then at most once every
/// [`DEBOUNCE`], and a change inside that wait is acted on when it ends rather than dropped.
pub fn watch(log: &Path, on_change: impl Fn() + Send + 'static) -> Result<LogWatcher, WatchError> {
    let folder = log
        .parent()
        .ok_or(WatchError::Io {
            kind: std::io::ErrorKind::InvalidInput,
        })?
        .to_path_buf();
    let name = log.file_name().map(|n| n.to_os_string());

    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        // A failed event is not a reason to stop watching: the next one may be fine.
        if let Ok(event) = res {
            let _ = tx.send(event);
        }
    })
    .map_err(watch_error)?;
    watcher
        .watch(&folder, RecursiveMode::NonRecursive)
        .map_err(watch_error)?;

    std::thread::spawn(move || {
        // When to act is `run::Throttle`'s to say: at once on a change,
        // and a change inside the quiet period kept for when it ends — never dropped.
        let mut throttle = Throttle::new(DEBOUNCE);
        loop {
            // Waits for the next change, or only as long as a kept change has left.
            let received = match throttle.wait(Instant::now()) {
                Some(wait) => rx.recv_timeout(wait),
                None => rx.recv().map_err(|_| RecvTimeoutError::Disconnected),
            };
            let act = match received {
                Ok(event) => touches(&event, name.as_deref()) && throttle.event(Instant::now()),
                Err(RecvTimeoutError::Timeout) => throttle.tick(Instant::now()),
                // The sender is dropped when the watcher is: the watch is over.
                Err(RecvTimeoutError::Disconnected) => break,
            };
            if act {
                on_change();
            }
        }
    });

    Ok(LogWatcher { _inner: watcher })
}

/// `notify`'s own error carries paths and an OS message; neither may travel, and neither is
/// worth keeping — the only thing the caller can do is go without a watch.
fn watch_error(_e: notify::Error) -> WatchError {
    WatchError::Io {
        kind: std::io::ErrorKind::Other,
    }
}

/// Whether an event in the folder is about the log itself; with no file name, every one is.
fn touches(event: &notify::Event, name: Option<&OsStr>) -> bool {
    match name {
        Some(n) => event
            .paths
            .iter()
            .any(|p| p.file_name().is_some_and(|f| f == n)),
        None => true,
    }
}
