//! unpack — Isaac's ARCH000 reader, targeted extraction by path (read-only).

mod arch;
mod bogocrypt;
mod extract;
mod hash;
mod isaac;
mod lzw;
mod miniz;
mod resource_set;

pub use arch::{Archive, CompressionMode, Entry, OpenError};
pub use extract::{extract_subset, Diagnostic, ExtractReport};
pub use hash::{path_key, PathKey};
pub use resource_set::{ArchiveInfo, ResourceSet};

#[doc(hidden)]
pub fn __lzw_decompress(archive: &[u8], start: usize, decompressed_len: usize) -> Option<Vec<u8>> {
    crate::lzw::decompress(archive, start, decompressed_len)
}
