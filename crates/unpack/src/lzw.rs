//! Chunked LZW decompression for the ARCH000 format.
//!
//! Ported from Gibbed.Rebirth (Rick Gibbed, zlib license),
//! `projects/Gibbed.Rebirth.FileFormats/LZW.cs` and `BitReader.cs`.
//! Variant: variable width MSB-first, starts at 8 bits, dictionary capacity 4096.
//! The dictionary is SHARED across all chunks of an entry (not reset per chunk).
//! `previousIndex` and `lastValue` persist across chunks. Automatic reset when
//! `count + 1 >= 4096`, no explicit clear code.

/// Decompresses the entry starting at `start`: a sequence of chunks
/// `[u32 chunkLen][chunkLen byte LZW]` until `decompressed_len` bytes are reached.
/// Returns `None` (never panics) on any inconsistency or overrun.
pub(crate) fn decompress(archive: &[u8], start: usize, decompressed_len: usize) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(decompressed_len);
    let mut cur = start;

    // State shared across chunks (Gibbed: dictionary created once, outside the loop).
    let mut dict = Dict::new();
    let mut prev_idx: i32 = -1; // −1 = no previous (start)
    let mut last_value: u8 = 0;

    while out.len() < decompressed_len {
        let hdr = archive.get(cur..cur + 4)?;
        let chunk_len = u32::from_le_bytes([hdr[0], hdr[1], hdr[2], hdr[3]]) as usize;
        cur += 4;
        let chunk = archive.get(cur..cur + chunk_len)?;
        cur += chunk_len;
        decode_chunk(chunk, &mut out, &mut dict, &mut prev_idx, &mut last_value)?;
    }
    if out.len() != decompressed_len {
        return None;
    }
    Some(out)
}

// ──────────────────────────────────────────────────────────────────────────────
// LZW dictionary
// ──────────────────────────────────────────────────────────────────────────────

const DICT_CAPACITY: usize = 4096;

#[derive(Clone, Copy)]
struct Entry {
    prev: i32, // −1 for root literals
    value: u8,
}

struct Dict {
    entries: Vec<Entry>,
    code_length: u32,
    window: Vec<u8>,
}

impl Dict {
    fn new() -> Self {
        let mut d = Dict {
            entries: Vec::with_capacity(DICT_CAPACITY),
            code_length: 8,
            window: vec![0u8; DICT_CAPACITY],
        };
        d.reset();
        d
    }

    fn reset(&mut self) {
        self.entries.clear();
        for i in 0u32..256 {
            self.entries.push(Entry {
                prev: -1,
                value: i as u8,
            });
        }
        self.code_length = 8;
    }

    #[inline]
    fn count(&self) -> usize {
        self.entries.len()
    }

    #[inline]
    fn update_code_length(&mut self) {
        while self.count() >= (1usize << self.code_length) {
            self.code_length += 1;
        }
    }

    /// Adds `{prev: prev_index, value: last_value}`.
    /// Matches Gibbed's `Add(byte lastValue, int previousIndex)`.
    fn add(&mut self, last_value: u8, prev_index: usize) -> Option<()> {
        if self.entries.len() >= DICT_CAPACITY {
            return None;
        }
        self.entries.push(Entry {
            prev: prev_index as i32,
            value: last_value,
        });
        Some(())
    }

    /// Decodes the sequence for `idx`, returns (first_byte, &slice).
    /// `first_byte` matches `lastValue` in Gibbed (the root of the chain).
    fn decode(&mut self, mut idx: usize) -> Option<(u8, &[u8])> {
        let write_end = self.window.len();
        let mut write_pos = write_end;
        loop {
            let e = self.entries.get(idx)?;
            write_pos = write_pos.checked_sub(1)?;
            self.window[write_pos] = e.value;
            if e.prev == -1 {
                break;
            }
            idx = e.prev as usize;
        }
        let first = self.window[write_pos];
        Some((first, &self.window[write_pos..write_end]))
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Bit reader (MSB-first, as in Gibbed.Rebirth BitReader.cs)
// ──────────────────────────────────────────────────────────────────────────────

struct BitReader<'a> {
    data: &'a [u8],
    pos: usize,
    buf: u64,
    remaining: u32,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        BitReader {
            data,
            pos: 0,
            buf: 0,
            remaining: 0,
        }
    }

    fn read(&mut self, n: u32) -> Option<u32> {
        while self.remaining < n {
            let byte = *self.data.get(self.pos)?;
            self.buf = (self.buf << 8) | byte as u64;
            self.remaining += 8;
            self.pos += 1;
        }
        self.remaining -= n;
        let mask = (1u32 << n) - 1;
        let val = ((self.buf >> self.remaining) as u32) & mask;
        Some(val)
    }

    /// True when all the chunk's bytes have been consumed.
    /// Gibbed: `reader.Position >= reader.Length`.
    /// The leftover bits in the buffer are padding and should be ignored.
    #[inline]
    fn exhausted(&self) -> bool {
        self.pos >= self.data.len()
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Decoder for a single LZW chunk
// ──────────────────────────────────────────────────────────────────────────────

/// Decodes ONE LZW chunk and appends to `out`.
/// Ported faithfully from Gibbed.Rebirth LZW.cs (Rick Gibbed, zlib license).
///
/// `dict`, `prev_idx`, `last_value` are shared across chunks: the dictionary is
/// **not** reset between one chunk and the next.
/// `prev_idx == −1` indicates the absence of a previous code (absolute start, or
/// right after a dictionary reset).
fn decode_chunk(
    chunk: &[u8],
    out: &mut Vec<u8>,
    dict: &mut Dict,
    prev_idx: &mut i32,
    last_value: &mut u8,
) -> Option<()> {
    let mut reader = BitReader::new(chunk);

    // First read of the chunk: matches the first read of every "block" in Gibbed.
    // Doesn't add an entry to the dictionary (no valid previousIndex at the start of a block).
    let first_code = reader.read(dict.code_length)? as usize;
    let (lv, seq) = dict.decode(first_code)?;
    out.extend_from_slice(seq);
    *last_value = lv;
    *prev_idx = first_code as i32;

    // Main loop — matches Gibbed's while(reader.Position < reader.Length).
    while !reader.exhausted() {
        // Reset if the dictionary is nearly full.
        if dict.count() + 1 >= DICT_CAPACITY {
            dict.reset();
            *prev_idx = -1;

            let code = reader.read(dict.code_length)? as usize;
            let (lv, seq) = dict.decode(code)?;
            out.extend_from_slice(seq);
            *last_value = lv;
            *prev_idx = code as i32;
            continue;
        }

        dict.update_code_length();
        let code = reader.read(dict.code_length)? as usize;

        if code == dict.count() {
            // KwKwK case (Gibbed): adds first, then decodes.
            if *prev_idx >= 0 {
                dict.add(*last_value, *prev_idx as usize)?;
            }
            let (lv, seq) = dict.decode(code)?;
            out.extend_from_slice(seq);
            *last_value = lv;
        } else {
            if code > dict.count() {
                return None;
            }
            let (lv, seq) = dict.decode(code)?;
            out.extend_from_slice(seq);
            *last_value = lv;
            if *prev_idx >= 0 {
                dict.add(*last_value, *prev_idx as usize)?;
            }
        }

        *prev_idx = code as i32;
    }

    Some(())
}
