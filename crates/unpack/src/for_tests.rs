//! Entry points that exist only so tests can reach a shape the public API doesn't build.
//!
//! One module per crate, and nothing test-only anywhere else in the public surface: a name
//! in the crate's `pub use` list says "call me", which is the opposite of what these mean.
//! Nothing outside a `tests/` target may call them.

/// One compression mode on its own, for the tests that state what a malformed stream does.
/// Production code goes through `Archive`, which reads the mode from byte `0x07`.
pub fn lzw_decompress(archive: &[u8], start: usize, decompressed_len: usize) -> Option<Vec<u8>> {
    crate::lzw::decompress(archive, start, decompressed_len)
}
