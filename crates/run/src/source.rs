//! Is this the launch we were reading, or a new one?
//!
//! The question belongs here and not in `log-watch` for the repo's own reason: a return value
//! worth checking lives in a pure crate. The watcher supplies bytes; this decides.
//!
//! **The prefix alone cannot answer it.** Measured on 2026-09-13: the first 4 KiB of a log is
//! the machine describing itself — OpenGL, driver, OpenAL, the game's own DLL path — and two
//! launches on one machine differ there by a single load timing, if at all. So the prefix is
//! only a cheap filter and the proof is an **anchor**: the bytes that end at the offset already
//! consumed, which are run content and never the banner.
//!
//! The two mistakes do not cost the same. Reading a new launch as the old one loses the runs
//! before the offset; reading the old launch as new **imports every run in it twice**, and a
//! duplicated run is wrong in the win rate for good. The anchor makes the second impossible and
//! leaves the first to a collision of 64 bytes of run content.

/// How much of the head identifies a file at all. 4 KiB is the spec's number and it holds the
/// whole banner.
pub const PREFIX_BYTES: usize = 4096;

/// How much of what we already read proves it is the same file. Sixty-four bytes of a log are
/// most of a line: two different launches do not write the same one at the same offset.
pub const ANCHOR_BYTES: usize = 64;

/// What a source is remembered by, between one run of the app and the next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceKey {
    /// FNV-1a of the first `prefix_len` bytes.
    pub prefix: u64,
    /// How many bytes that hash covers — at most [`PREFIX_BYTES`], and fewer when the file was
    /// shorter than that the first time we read it.
    ///
    /// **It is stored because the window has to stay the same window.** A log is a few hundred
    /// bytes for its first instants, so "the first 4 KiB" of it is the whole file: hash that and
    /// the number changes with every line the game writes, every read looks like a new file, and
    /// every run already in the archive is imported again. Found by a test on 2026-09-13.
    pub prefix_len: u64,
    /// FNV-1a of the [`ANCHOR_BYTES`] that end at `offset`.
    pub anchor: u64,
    /// How far we had read.
    pub offset: u64,
}

impl SourceKey {
    /// The key of a file we have just read up to `offset`. `at_offset` is the window ending
    /// there — shorter than [`ANCHOR_BYTES`] near the start of a file, which is not a case to
    /// handle: a shorter window simply hashes to a different number.
    pub fn new(prefix: &[u8], at_offset: &[u8], offset: u64) -> Self {
        Self {
            prefix: fingerprint(prefix),
            prefix_len: prefix.len() as u64,
            anchor: fingerprint(at_offset),
            offset,
        }
    }
}

/// What to do with a file we have a stored key for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resume {
    /// The same launch: carry on from here.
    Continue { offset: u64 },
    /// Not the file we were reading. A new source, read from zero.
    Fresh,
}

/// FNV-1a, 64 bits, written out because this number is stored.
///
/// `DefaultHasher` is explicitly not stable across Rust releases, and a key that changed with
/// the toolchain would make every source look new after an upgrade — which re-imports the whole
/// archive. This is a few lines and the same number forever.
pub fn fingerprint(bytes: &[u8]) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    bytes.iter().fold(OFFSET_BASIS, |hash, b| {
        (hash ^ *b as u64).wrapping_mul(PRIME)
    })
}

/// The decision. Three ways to be a new file and one way to be the old one.
///
/// `prefix` must be read as **`stored.prefix_len` bytes**, not as [`PREFIX_BYTES`]: the window
/// has to be the one the stored hash covers, or a file that has merely grown stops matching
/// itself.
pub fn resume(stored: &SourceKey, prefix: &[u8], len: u64, at_offset: &[u8]) -> Resume {
    if len < stored.offset {
        return Resume::Fresh;
    }
    if prefix.len() as u64 != stored.prefix_len || fingerprint(prefix) != stored.prefix {
        return Resume::Fresh;
    }
    if fingerprint(at_offset) != stored.anchor {
        return Resume::Fresh;
    }
    Resume::Continue {
        offset: stored.offset,
    }
}
