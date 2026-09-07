//! `MiniZ` compression mode (byte 0x07 = 2): `afterbirth.a`, `afterbirthp.a`,
//! `repentance.a` — i.e. almost all of the game's content.
//!
//! Ported from Gibbed.Rebirth (Rick Gibbed, zlib license), `ArchiveEntry.cs`.
//! Each block has a `u32` header: bit 31 = last block, bits 0–30 = on-disk length.
//! A block that is **not the last one and is exactly 1024 bytes long is "stored"**; from
//! the first stored block onward the whole entry is stored (the original never goes
//! back). Stored blocks are XOR'd with the ISAAC cipher, seeded from the name hash;
//! the others are **raw deflate**.

use crate::isaac::Isaac;

/// Maximum on-disk size of a block (`blockBytes` is `new byte[0x800]`).
const MAX_BLOCK: usize = 0x800;
/// Bytes produced by a compressed block.
const BLOCK_OUTPUT: usize = 1024;

/// Decompresses the entry starting at `start`. `name_hash_b` is the second name hash,
/// used to seed ISAAC. Returns `None` (never panics) on any inconsistency.
pub(crate) fn decompress(
    archive: &[u8],
    start: usize,
    decompressed_len: usize,
    name_hash_b: u32,
) -> Option<Vec<u8>> {
    let mut out: Vec<u8> = Vec::with_capacity(decompressed_len);
    let mut cur = start;
    let mut remaining = decompressed_len;

    let mut isaac: Option<Isaac> = None;
    let mut is_compressed = true;
    let mut is_last_block = false;

    // Two exits: the block marked as last, or the declared bytes running out.
    // The original stops only on the marker; here the second condition prevents an
    // inconsistent archive from writing past the declared length.
    while !is_last_block && remaining > 0 {
        let hdr = archive.get(cur..cur + 4)?;
        let flags = u32::from_le_bytes([hdr[0], hdr[1], hdr[2], hdr[3]]);
        cur += 4;

        let block_len = (flags & !0x8000_0000) as usize;
        is_last_block = flags & 0x8000_0000 != 0;
        if block_len > MAX_BLOCK {
            return None;
        }
        let block = archive.get(cur..cur + block_len)?;
        cur += block_len;

        if !is_compressed || (!is_last_block && block_len == BLOCK_OUTPUT) {
            // "Stored" block: from here on the entry no longer goes back to compressed.
            is_compressed = false;
            if block_len > remaining {
                return None;
            }
            // One ISAAC value covers four bytes, consumed from the least significant one.
            // The shift is arithmetic as in the original, but it doesn't matter: only the
            // low byte is used from each step.
            let cipher = isaac.get_or_insert_with(|| Isaac::from_name_hash(name_hash_b));
            let mut seed: i32 = 0;
            for (o, byte) in block.iter().enumerate() {
                if o & 3 != 0 {
                    seed >>= 8;
                } else {
                    seed = cipher.value();
                }
                out.push(byte ^ seed as u8);
            }
            remaining -= block_len;
        } else {
            // `inflate_block` never produces more than `limit`, so the subtraction
            // below can never go below zero.
            let limit = remaining.min(BLOCK_OUTPUT);
            let produced = inflate_block(block, limit)?;
            remaining -= produced.len();
            out.extend_from_slice(&produced);
        }
    }

    // If the last-block marker arrived earlier than expected, the entry is shorter than
    // it declares: better no data than truncated data passed off as complete.
    if out.len() != decompressed_len {
        return None;
    }
    Some(out)
}

/// Raw inflate of a single block, with the same contract as the C#: reads up to
/// `limit` bytes and does **not** require the stream to end within the block.
/// `miniz_oxide`'s high-level helper would discard the output of a truncated stream.
fn inflate_block(block: &[u8], limit: usize) -> Option<Vec<u8>> {
    use miniz_oxide::inflate::core::{decompress, inflate_flags, DecompressorOxide};
    use miniz_oxide::inflate::TINFLStatus;

    let mut state = DecompressorOxide::new();
    let mut buf = vec![0u8; limit];
    // No zlib header (the original uses `new Inflater(true)`), non-circular buffer,
    // and `HAS_MORE_INPUT` because the stream continues beyond this block.
    let flags = inflate_flags::TINFL_FLAG_USING_NON_WRAPPING_OUTPUT_BUF
        | inflate_flags::TINFL_FLAG_HAS_MORE_INPUT;
    let (status, _consumed, written) = decompress(&mut state, block, &mut buf, 0, flags);
    match status {
        // `Done` when the stream ends within the block; the other two when it continues.
        TINFLStatus::Done | TINFLStatus::NeedsMoreInput | TINFLStatus::HasMoreOutput => {
            buf.truncate(written);
            Some(buf)
        }
        _ => None,
    }
}
