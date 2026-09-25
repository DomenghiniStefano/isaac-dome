//! Parsing of the ARCH000 archive: header, index, reading by path.

use std::path::Path;

use crate::hash::PathKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry {
    pub key: PathKey,
    pub offset: u64,
    pub decompressed_len: u32,
    pub checksum: u32,
}

/// Data algorithm, from byte `0x07` of the header.
///
/// **This is not a version number.** It matches `ArchiveCompressionMode` from
/// Gibbed.Rebirth, and it picks how the entries are encoded: the game's archives use
/// three different ones, so it must be read from the file and never assumed.
///
/// No `Serialize`: nobody serializes it, and the `Unknown(u8)` variant is a newtype, a
/// shape the project rules forbid across the IPC boundary. If it ever needs to cross it,
/// that decision should be made then, with a tagged shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionMode {
    /// 0 — `graphics.a`, `music.a`. Block XOR with a key from the name hash.
    Bogocrypt1,
    /// 1 — `config.a`, `fonts.a`, `animations.a`. Chunked LZW.
    Lzw,
    /// 2 — `afterbirth.a`, `afterbirthp.a`, `repentance.a`. Deflate + ISAAC cipher.
    MiniZ,
    /// 5 — not observed in this installation's archives.
    Bogocrypt2,
    /// Unexpected value: the archive still opens, but its entries can't be read.
    Unknown(u8),
}

impl CompressionMode {
    fn from_byte(b: u8) -> CompressionMode {
        match b {
            0 => CompressionMode::Bogocrypt1,
            1 => CompressionMode::Lzw,
            2 => CompressionMode::MiniZ,
            5 => CompressionMode::Bogocrypt2,
            other => CompressionMode::Unknown(other),
        }
    }
}

/// An open archive: **the index in memory, the data on disk**.
///
/// The eight `.a` files of a full installation weigh about 1.3 GB. Holding them in RAM
/// to read a single icon is the most likely way to make the app crash silently on a
/// machine with little memory — that is, to violate "degrade, never fail". What stays
/// here are the open file and the index; a given entry's bytes are read on demand.
pub struct Archive {
    file: std::fs::File,
    entries: Vec<Entry>,
    /// Keyed on both hashes: two paths whose djb2 collide are two files, and
    /// an index on djb2 alone kept only the last one read.
    index: std::collections::HashMap<PathKey, usize>,
    /// Where each entry's data ends, in the same order as `entries`.
    ///
    /// The index says where an entry **starts**, never how long it is compressed: the
    /// decompressors stop on their own once they've produced `decompressed_len` bytes.
    /// But reading one without reading the whole file still needs a bound, and that
    /// bound is the offset that comes next — entry data is contiguous, verified on
    /// `fonts.a` (the last chunk of entry 0 ends exactly where entry 1 begins). The last
    /// entry ends where the index table begins.
    ends: Vec<u64>,
    mode: CompressionMode,
}

/// Reads exactly `buf.len()` bytes starting at `offset`, without touching the file's
/// shared cursor: two Tauri commands can read the same archive at once, and
/// `seek` + `read` would race on the cursor.
fn read_exact_at(file: &std::fs::File, buf: &mut [u8], offset: u64) -> std::io::Result<()> {
    #[cfg(windows)]
    use std::os::windows::fs::FileExt;
    let mut done = 0;
    while done < buf.len() {
        #[cfg(windows)]
        let n = file.seek_read(&mut buf[done..], offset + done as u64)?;
        #[cfg(not(windows))]
        let n = {
            use std::os::unix::fs::FileExt;
            file.read_at(&mut buf[done..], offset + done as u64)?
        };
        if n == 0 {
            return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof));
        }
        done += n;
    }
    Ok(())
}

#[derive(Debug)]
pub enum OpenError {
    TooShort,
    BadMagic { found: [u8; 7] },
    Io(std::io::Error),
}

impl From<std::io::Error> for OpenError {
    fn from(e: std::io::Error) -> Self {
        OpenError::Io(e)
    }
}

/// Bytes in one index record: five little-endian `u32`.
const RECORD_LEN: usize = 20;

/// Bytes in the header: the 7-byte signature, the mode byte, the index offset, the count.
const HEADER_LEN: usize = 14;

/// What the header declares.
struct Header {
    mode: CompressionMode,
    index_off: u64,
    count: usize,
}

/// The header, checked for its signature. The caller has already made sure the file holds one.
fn read_header(file: &std::fs::File) -> Result<Header, OpenError> {
    let mut header = [0u8; HEADER_LEN];
    read_exact_at(file, &mut header, 0)?;
    if &header[0..7] != b"ARCH000" {
        let mut found = [0u8; 7];
        found.copy_from_slice(&header[0..7]);
        return Err(OpenError::BadMagic { found });
    }
    Ok(Header {
        mode: CompressionMode::from_byte(header[7]),
        index_off: u32::from_le_bytes([header[8], header[9], header[10], header[11]]) as u64,
        count: u16::from_le_bytes([header[12], header[13]]) as usize,
    })
}

/// The index table, read in one shot and only for the whole records that fit in the file: a
/// truncated index degrades to "take what's there".
fn read_table(file: &std::fs::File, header: &Header, file_len: u64) -> std::io::Result<Vec<u8>> {
    let available = file_len.saturating_sub(header.index_off) / RECORD_LEN as u64;
    let readable = (header.count as u64).min(available) as usize;
    let mut table = vec![0u8; readable * RECORD_LEN];
    if readable > 0 {
        read_exact_at(file, &mut table, header.index_off)?;
    }
    Ok(table)
}

/// One index record, or `None` when its offset points past the end of the file: such a
/// record is discarded.
fn parse_record(record: &[u8], file_len: u64) -> Option<Entry> {
    let word = |k: usize| {
        let p = k * 4;
        u32::from_le_bytes([record[p], record[p + 1], record[p + 2], record[p + 3]])
    };
    let offset = word(2) as u64;
    (offset < file_len).then(|| Entry {
        key: PathKey {
            djb2: word(0),
            fnv: word(1),
        },
        offset,
        decompressed_len: word(3),
        checksum: word(4),
    })
}

/// Where each key's entry is. Two records with the same key: the later one wins.
fn index_of(entries: &[Entry]) -> std::collections::HashMap<PathKey, usize> {
    entries
        .iter()
        .enumerate()
        .map(|(i, entry)| (entry.key, i))
        .collect()
}

impl Archive {
    /// Opens the archive reading **only** the header and the index table. The data stays
    /// on disk: a 700 MB `.a` file costs a few hundred kilobytes here.
    pub fn open(path: &Path) -> Result<Archive, OpenError> {
        let file = std::fs::File::open(path)?;
        let file_len = file.metadata()?.len();
        if file_len < HEADER_LEN as u64 {
            return Err(OpenError::TooShort);
        }
        let header = read_header(&file)?;
        let table = read_table(&file, &header, file_len)?;
        let entries: Vec<Entry> = table
            .chunks_exact(RECORD_LEN)
            .filter_map(|record| parse_record(record, file_len))
            .collect();
        let index = index_of(&entries);
        // The data ends where the index begins; if the header declares that beyond the
        // file, the end of the file wins instead.
        let ends = entry_ends(&entries, header.index_off.min(file_len));
        Ok(Archive {
            file,
            entries,
            index,
            ends,
            mode: header.mode,
        })
    }

    pub fn contains(&self, resource_path: &str) -> bool {
        self.index
            .contains_key(&crate::hash::path_key(resource_path))
    }

    /// Compression mode declared by the header, read from the file.
    pub fn mode(&self) -> CompressionMode {
        self.mode
    }

    pub fn read(&self, resource_path: &str) -> Option<Vec<u8>> {
        let &i = self.index.get(&crate::hash::path_key(resource_path))?;
        self.read_entry(i)
    }

    /// Index entry `i`, decompressed. Reads only its bytes from disk.
    ///
    /// `None` covers every way an entry can fail to be read — out-of-bounds index,
    /// failed read, unimplemented compression mode, corrupt data: it's a resource that's
    /// missing, not an error that stops the app.
    pub fn read_entry(&self, i: usize) -> Option<Vec<u8>> {
        let entry = self.entries.get(i)?;
        let start = entry.offset;
        let end = *self.ends.get(i)?;
        if end <= start {
            return None;
        }
        let mut compressed = vec![0u8; (end - start) as usize];
        read_exact_at(&self.file, &mut compressed, start).ok()?;
        let len = entry.decompressed_len as usize;
        match self.mode {
            CompressionMode::Lzw => crate::lzw::decompress(&compressed, 0, len),
            CompressionMode::MiniZ => crate::miniz::decompress(&compressed, 0, len, entry.key.fnv),
            CompressionMode::Bogocrypt1 => {
                crate::bogocrypt::decompress(&compressed, 0, len, entry.key.fnv)
            }
            // Not implemented: no archive in this installation uses them, so there's
            // nothing to verify them against. The archive stays openable and indexable,
            // and the single entry just fails to read: degrade, never fail.
            CompressionMode::Bogocrypt2 | CompressionMode::Unknown(_) => None,
        }
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }
}

/// Where each entry's data ends: the first offset **greater** than its own, or
/// `data_end` if it's the last one. The index isn't assumed to be sorted by offset —
/// it's sorted here.
///
/// A bound wider than the truth does no harm: the decompressors stop once they've
/// produced the declared bytes. A narrower one does, which is why two entries sharing
/// the same offset (which the index allows) both get the same bound, not zero.
fn entry_ends(entries: &[Entry], data_end: u64) -> Vec<u64> {
    let mut offsets: Vec<u64> = entries.iter().map(|e| e.offset).collect();
    offsets.sort_unstable();
    offsets.dedup();
    entries
        .iter()
        .map(|e| {
            match offsets.binary_search(&e.offset) {
                // There's a following offset: the data reaches up to it.
                Ok(p) if p + 1 < offsets.len() => offsets[p + 1].min(data_end.max(e.offset)),
                _ => data_end.max(e.offset),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(offset: u64) -> Entry {
        Entry {
            key: PathKey { djb2: 0, fnv: 0 },
            offset,
            decompressed_len: 0,
            checksum: 0,
        }
    }

    #[test]
    fn an_entry_ends_where_the_next_one_starts() {
        let ends = entry_ends(&[entry(14), entry(100), entry(250)], 400);
        assert_eq!(ends, vec![100, 250, 400]);
    }

    /// The index makes no promise of being sorted by offset: we impose the order ourselves.
    #[test]
    fn the_order_of_the_index_does_not_matter() {
        let ends = entry_ends(&[entry(250), entry(14), entry(100)], 400);
        assert_eq!(ends, vec![400, 100, 250]);
    }

    /// Two entries at the same offset are the same slice of data (a duplicated file):
    /// they get the same bound, not an empty range.
    #[test]
    fn two_entries_at_the_same_offset_share_their_slice() {
        let ends = entry_ends(&[entry(14), entry(14), entry(100)], 400);
        assert_eq!(ends, vec![100, 100, 400]);
    }

    /// An entry that starts past the declared end of the data does not produce an
    /// inverted range: `read_entry` rejects it, and never attempts an absurd allocation.
    #[test]
    fn an_entry_past_the_end_of_the_data_does_not_invert_its_range() {
        let ends = entry_ends(&[entry(900)], 400);
        assert_eq!(ends, vec![900]);
    }
}
