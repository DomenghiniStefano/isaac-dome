//! unpack — Isaac's ARCH000 reader, targeted extraction by path (read-only).

mod arch;
mod bogocrypt;
mod extract;
#[cfg(feature = "test-api")]
pub mod for_tests;
mod hash;
mod isaac;
mod lzw;
mod miniz;
mod resource_set;

pub use arch::{Archive, CompressionMode, Entry, OpenError};
pub use extract::{extract_subset, Diagnostic, ExtractReport};
pub use hash::{path_key, PathKey};
pub use resource_set::{ArchiveInfo, ResourceSet};

/// How much to reserve before decompressing an entry (card #80, item 09). The declared length
/// is a raw u32 from the archive's index, and a corrupted one can claim four gigabytes: reserved
/// up front, that is an allocation that aborts the process where the allocator refuses it. The
/// reservation is capped at the bytes the archive still holds from `start` — an entry that
/// really is larger grows past it the ordinary way, and one that is not ends as `None` when the
/// input runs out, as it always did.
pub(crate) fn prealloc(archive: &[u8], start: usize, decompressed_len: usize) -> usize {
    decompressed_len.min(archive.len().saturating_sub(start))
}
