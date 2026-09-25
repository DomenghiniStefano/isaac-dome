//! Positional reads over a file somebody else is writing.
//!
//! Never the whole file: the log B8 measured is 33,222 lines and its size is the user's choice,
//! not ours. Each call opens its own handle, so a seek here races with nothing — the repo's rule
//! about `seek_read` is about a handle shared between commands, which this is not.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::WatchError;

/// How much is read in one go while catching up on a file.
pub const CHUNK: usize = 256 * 1024;

/// How long the file is right now.
pub fn len(path: &Path) -> Result<u64, WatchError> {
    Ok(std::fs::metadata(path).map_err(WatchError::io)?.len())
}

/// The first `n` bytes, or the whole file when it is shorter.
pub fn head(path: &Path, n: usize) -> Result<Vec<u8>, WatchError> {
    chunk(path, 0, n)
}

/// The `n` bytes that **end** at `offset` — the anchor of `run::resume`. Near the start of a
/// file the window is shorter, which needs no special case: a shorter window hashes to a
/// different number, and it is compared against one taken the same way.
pub fn window_ending_at(path: &Path, offset: u64, n: usize) -> Result<Vec<u8>, WatchError> {
    let n = n as u64;
    let start = offset.saturating_sub(n);
    chunk(path, start, (offset - start) as usize)
}

/// At most `max` bytes from `offset`. Past the end of the file this is empty, not an error: the
/// file shrank, and `run::resume` is what says so.
pub fn chunk(path: &Path, offset: u64, max: usize) -> Result<Vec<u8>, WatchError> {
    let mut file = File::open(path).map_err(WatchError::io)?;
    file.seek(SeekFrom::Start(offset)).map_err(WatchError::io)?;
    // As many reads as it takes, an interrupted one retried (`read_to_end`), and never more than
    // `max` bytes however far the file has grown (`take`).
    let mut buf = Vec::with_capacity(max);
    file.take(max as u64)
        .read_to_end(&mut buf)
        .map_err(WatchError::io)?;
    Ok(buf)
}
