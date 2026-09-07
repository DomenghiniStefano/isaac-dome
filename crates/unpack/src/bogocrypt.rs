//! `Bogocrypt1` compression mode (byte 0x07 = 0): `graphics.a`, `music.a`.
//!
//! Ported from Gibbed.Rebirth (Rick Gibbed, zlib license), `ArchiveFile.Bogocrypt1`.
//! There's no compression: the data is raw, encrypted in groups of 4 bytes with a key
//! that evolves at each group, seeded from the name hash. The on-disk size is the
//! declared one **rounded up to the next multiple of 4**; the padding bytes are discarded.
//!
//! The original processes 1024-byte blocks, carrying the key across them. Here it's a
//! single pass: 1024 is a multiple of 4, so splitting the loop or not gives the same result.

/// The entry's initial key, as `ArchiveEntry.BogocryptKey`.
fn initial_key(name_hash_b: u32) -> u32 {
    (name_hash_b ^ 0xF952_4287) | 1
}

/// Key evolution step, one group of 4 bytes at a time.
///
/// The original is a single expression that repeats the same subexpression twice:
/// `key ^= ((key ^ (key << 8)) >> 9) ^ (key << 8) ^ ((((key ^ (key << 8)) >> 9) ^ key ^ (key << 8)) << 23)`.
/// Setting `u = key ^ (key << 8)` and `t = (u >> 9) ^ (key << 8)`, the shift's second
/// operand equals `(u >> 9) ^ u`, i.e. `t ^ key`. Hence the form below, identical to it.
fn next_key(key: u32) -> u32 {
    let t = ((key ^ (key << 8)) >> 9) ^ (key << 8);
    key ^ t ^ ((t ^ key) << 23)
}

/// Decrypts the entry starting at `start`. Returns `None` (never panics) if the bytes aren't enough.
pub(crate) fn decompress(
    archive: &[u8],
    start: usize,
    decompressed_len: usize,
    name_hash_b: u32,
) -> Option<Vec<u8>> {
    let padded = decompressed_len.next_multiple_of(4);
    let end = start.checked_add(padded)?;
    let mut out = archive.get(start..end)?.to_vec();

    // `padded` is a multiple of 4 by construction, so `chunks_exact_mut` leaves no remainder.
    let mut key = initial_key(name_hash_b);
    for group in out.chunks_exact_mut(4) {
        for (i, byte) in group.iter_mut().enumerate() {
            *byte ^= (key >> (8 * i as u32)) as u8;
        }
        match key & 15 {
            2 => {
                group.swap(0, 3);
                group.swap(1, 2);
            }
            9 => {
                group.swap(0, 1);
                group.swap(2, 3);
            }
            13 => {
                group.swap(0, 2);
                group.swap(1, 3);
            }
            _ => {}
        }
        key = next_key(key);
    }

    out.truncate(decompressed_len);
    Some(out)
}
