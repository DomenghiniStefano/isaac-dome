//! `notify`, a debounce, and a callback. Everything worth checking is somewhere else.
//!
//! **The folder is watched, not the file.** `log.txt` is replaced on every launch of the game,
//! and a watch registered on the file follows the old handle — going deaf exactly when a new
//! launch starts, which is the one moment that matters.

use std::path::Path;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

use notify::{RecursiveMode, Watcher};

use crate::WatchError;

/// How often a change is acted on at most. The game writes constantly — B8 counted 132 save
/// lines in one run — and every pass folds a source again.
const DEBOUNCE: Duration = Duration::from_secs(2);

/// Alive as long as the watch is. Dropping it ends the watch and, with it, the thread.
pub struct LogWatcher {
    _inner: notify::RecommendedWatcher,
}

/// Calls `on_change` when `log` may have changed, at most once every [`DEBOUNCE`].
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
        let mut last: Option<Instant> = None;
        // Ends when the sender is dropped, which is when the watcher is.
        while let Ok(event) = rx.recv() {
            let touched = match &name {
                Some(n) => event
                    .paths
                    .iter()
                    .any(|p| p.file_name().is_some_and(|f| f == n.as_os_str())),
                None => true,
            };
            if touched && last.is_none_or(|t| t.elapsed() >= DEBOUNCE) {
                last = Some(Instant::now());
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
