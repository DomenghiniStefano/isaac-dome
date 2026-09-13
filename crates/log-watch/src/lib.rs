//! log-watch — the half of the run archive that touches the disk.
//!
//! Deliberately thin, and the split is the repo's rule: in a file watcher the parts worth
//! checking are not the ones that hold the handle. Whether a file is the launch we were reading
//! is decided by `run::resume`; what a line means is decided by `run`'s fold; what is stored is
//! decided by `store`. What is left here is an offset, a read, and `notify`.

mod read;
mod sessions;

pub use read::{chunk, head, len, window_ending_at, CHUNK};
pub use sessions::sessions;

/// What can go wrong down here. An `io::ErrorKind`, never the OS's own sentence: it is not
/// translatable and on some platforms it repeats the path it was given.
#[derive(Debug)]
pub enum WatchError {
    Io { kind: std::io::ErrorKind },
    Store(store::StoreError),
}

impl WatchError {
    pub(crate) fn io(e: std::io::Error) -> Self {
        Self::Io { kind: e.kind() }
    }
}

impl From<store::StoreError> for WatchError {
    fn from(e: store::StoreError) -> Self {
        Self::Store(e)
    }
}
