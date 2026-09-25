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

/// The two other decompressors, exposed for the same synthetic cases LZW already had (card
/// #80, item 09): a declared length far past the input, a truncated block, bytes that mean
/// nothing. Each must answer `None`, and none may ask the allocator for the declared length.
pub fn miniz_decompress(
    archive: &[u8],
    start: usize,
    decompressed_len: usize,
    name_hash_b: u32,
) -> Option<Vec<u8>> {
    crate::miniz::decompress(archive, start, decompressed_len, name_hash_b)
}

pub fn bogocrypt_decompress(
    archive: &[u8],
    start: usize,
    decompressed_len: usize,
    name_hash_b: u32,
) -> Option<Vec<u8>> {
    crate::bogocrypt::decompress(archive, start, decompressed_len, name_hash_b)
}
