# unpack — `.a` archive reader and targeted extraction (design)

**Date:** 2026-09-01
**Sub-project:** M1 · third piece
**Status:** algorithm verified against real data; pending spec review

## Context

Third sub-project of M1. `unpack` reads the **ARCH000** archive format of
*The Binding of Isaac: Rebirth* and extracts into a local cache the **subset** of
resources the app needs (catalog XMLs, item and achievement sprites). It receives the
install folder from `discovery`; it feeds `catalog` (which normalizes the XMLs) and the
UI (which shows the extracted sprites).

It replicates the method of the official `ResourceExtractor` — same format, same
hashing, same decompression — but **in reverse and targeted**: instead of extracting the
whole archive (~1.5 GB) and rebuilding names from paths, it hashes the paths it wants and
looks them up in the index. No external process, no full dump, no game asset in the
package.

## Applicable non-negotiable constraints

1. **Read-only on the game archives.** The `.a` files are opened read-only. `unpack`
   writes **only** to the app's cache, never to the game folder.
2. **No asset in the package.** Resources are extracted from the user's copy at runtime.
   `filelist.txt` is **not** embedded: it's only used by the tests, as a source of real
   paths to verify the hashing against.
3. **Degrade, never fail.** `Archive::open` only fails on wrong magic, a file that's too
   short, or unreadable. A wanted path that's absent ends up in a report, not an error.
   Never `panic!` on a malformed/truncated archive or corrupted compressed data.
4. **No hardcoded count.** The number of records is read from the header.

## The ARCH000 format (verified against real data)

Verified against real `fonts.a` and `config.a`, and consistent with two reference
implementations (Gibbed.Rebirth, bladecoding).

### Header

```
0x00  char[7]  "ARCH000"          signature
0x07  u8       compression mode   is NOT a version number
0x08  u32 LE   index offset
0x0C  u16 LE   record count
0x0E           start of the data
...            file data (compressed)
<index offset> record table
```

The data starts at `0x0E` (right after the 14-byte header). The record table sits at the
end; each record is **20 bytes = 5 × u32 LE**.

> **Correction from 2026-09-03** (commit `33ded43`, "read all three compression modes,
> not just LZW"). This spec used to declare byte `0x07` as `version = 0x01`: that was
> wrong, and only held because the only archives tried up to that point —
> `config.a` and `fonts.a` — happen to have `1` there. The byte is
> `ArchiveCompressionMode` from Gibbed.Rebirth and says **how the entries are encoded**:
>
> | value | mode | archives |
> |---:|---|---|
> | 0 | Bogocrypt1, block XOR with a key from the name's hash | `graphics.a`, `music.a` |
> | 1 | Chunked LZW | `config.a`, `fonts.a`, `animations.a` |
> | 2 | MiniZ: deflate plus the ISAAC cipher | `afterbirth.a`, `afterbirthp.a`, `repentance.a` |
> | 5 | Bogocrypt2 | not observed on this installation |
>
> It must be read from the file and never assumed: assuming it is precisely what kept
> reading of seven archives out of eight broken while every test stayed green, because
> they all ran against the one archive the decoder happened to know how to open.

### Record (5 × u32), verified empirically

| word | content | verification |
|-----:|-----------|----------|
| 0 | **djb2 hash of the path** (seed 5381, multiplier ×33) | 21/24 paths of `config.a` match |
| 1 | **FNV-1a variant hash** of the path (init `0x5BB2220E`, prime `0x01000193`) | 21/24 match |
| 2 | **absolute offset** of the data in the archive (starts at 14) | increasing, aligned with the data |
| 3 | **decompressed size** in bytes | > on-disk size → compressed |
| 4 | checksum (of the content, presumably) — **not needed** for extraction | doesn't match either hash |

**Identification by double path hash.** A file is identified by `djb2(path)` +
`fnv(path)`. For targeted extraction we compute the two hashes of the wanted path and
look for the record with `word0 == djb2` (and, as an anti-collision guard,
`word1 == fnv`). The names aren't in the archive: `filelist.txt` is only needed if you
want to go back from records to names (a full listing), which we don't need.

**Path form for hashing:** as it appears in `filelist.txt` — lowercase, `/` separator,
no prefix stripped, no null terminator. (The app's paths must be normalized to this form
before hashing: `to_lowercase`, `\` → `/`.)

### Compression: chunked LZW (mode 1, one of three)

Each entry's data is compressed with **LZW**, framed in chunks:
```
[u32 chunkLen][LZW bytes] [u32 chunkLen][LZW bytes] ...   until reaching word3 (decompressed size)
```
Verified on `fonts.a` entry 0: 5 chunks at offsets 14, 597, 1053, 1532, 1953; the last
ends exactly at 1977 = the offset of entry 1. The first data byte (`0x43`) is the first
byte of the LZW payload, with no zlib/MSZIP header.

> **To be refined during implementation (against real data):** the exact LZW variant
> (initial code width, growth, any clear code) and the possible "mode" (stored vs LZW).
> References: Gibbed.Rebirth and bladecoding. Verification contract: decompressing an
> entry must produce exactly `word3` bytes and a file with a valid signature (a `.xml`
> starts with `<` or BOM+`<`, a `.png` with `\x89PNG`).

## Modules

```
crates/unpack/
  Cargo.toml
  src/
    lib.rs        # public types (Archive, Entry, ExtractReport, Diagnostic, OpenError), re-export
    hash.rs       # djb2 + fnv on the normalized path; path_key(&str) -> PathKey. Pure.
    lzw.rs        # chunked LZW decompression: decompress(&[u8], expected_len) -> Result<Vec<u8>>. Pure.
    arch.rs       # ARCH000 header + index parsing -> Archive/Entry; Archive::{open,contains,read,entries}
    extract.rs    # extract_subset(archive, wanted, cache_dir) -> ExtractReport
  tests/
    real_archive.rs  # against real fonts.a/config.a (samples/, skipped if absent): hash, index, read, extract
    synthetic.rs     # minimal in-memory header/index: open/degradation, always runs
```

## Model and public API

```rust
use std::path::{Path, PathBuf};

/// A file's key: the path's two hashes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PathKey { pub djb2: u32, pub fnv: u32 }

/// An index record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Entry {
    pub key: PathKey,
    pub offset: u64,
    pub decompressed_len: u32,
    pub checksum: u32,     // word4, exposed raw, not interpreted
}

/// An ARCH000 archive opened read-only.
pub struct Archive { /* path: PathBuf, entries: Vec<Entry>, index: HashMap<u32, usize> for djb2 */ }

#[derive(Debug)]
pub enum OpenError { TooShort, BadMagic { found: [u8; 7] }, Io(std::io::Error) }

impl Archive {
    /// Opens and indexes a `.a`. Read-only. Errors only on magic/length/IO.
    /// Malformed index records or ones with an out-of-bounds offset are silently
    /// discarded (defensive degradation); no diagnostic is emitted at open time.
    pub fn open(path: &Path) -> Result<Archive, OpenError>;
    /// Is there a file for this path? (matches on djb2 + fnv.)
    pub fn contains(&self, resource_path: &str) -> bool;
    /// Reads and DECOMPRESSES the bytes of the file at `resource_path`, or `None` if
    /// absent/corrupted.
    pub fn read(&self, resource_path: &str) -> Option<Vec<u8>>;
    /// The raw index (hashes and positions; no names).
    pub fn entries(&self) -> &[Entry];
}

/// Extracts into `cache_dir` only the requested paths, preserving structure. Never fails.
pub fn extract_subset(archive: &Archive, wanted: &[&str], cache_dir: &Path) -> ExtractReport;

#[derive(Debug, Clone, Serialize)]
pub struct ExtractReport {
    pub extracted: Vec<PathBuf>,
    pub missing: Vec<String>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Diagnostic {
    // Note: there is no EntryOutOfBounds. Out-of-bounds records are silently
    // discarded in Archive::open (defensive degradation, no diagnostic channel
    // at open time). Diagnostic only covers extraction errors.
    DecompressFailed { path: String },  // LZW doesn't produce decompressed_len bytes
    WriteFailed { path: PathBuf, reason: String },
}
```

`PathKey`, `Entry`, `ExtractReport`, `Diagnostic` derive `Serialize`. `OpenError` doesn't.
No dependency on Tauri. LZW implemented by hand in `lzw.rs` (no decompression
dependency: Isaac's LZW is custom, not zlib).

## Algorithm

**`path_key(path)`** (in `hash.rs`): normalize (`to_lowercase`, `\`→`/`), then
`djb2`: `h=5381; for b in bytes { h = h.wrapping_mul(33).wrapping_add(b) }`;
`fnv`: `h=0x5BB2220E; for b in bytes { h = (h ^ b).wrapping_mul(0x01000193) }`.

**`open`:** if `< 14` bytes → `TooShort`; if the first 7 ≠ `ARCH000` → `BadMagic`. Read
`index_offset` (u32@0x08) and `count` (u16@0x0C). For each of the `count` records at
`index_offset`, read the 5 words → `Entry`. If the index record is truncated or its
`offset` overruns the file, silently discard the record and continue (defensive
degradation; no `Diagnostic` is emitted at open time). Build a `HashMap<djb2, idx>`.

**`read(path)`:** `k = path_key(path)`; look for the record with `djb2 == k.djb2 &&
fnv == k.fnv`; if absent → `None`. Otherwise decompress from `offset` with the
decompressor for the mode declared in the header, up to `decompressed_len` bytes; if
decompression doesn't check out → `None`. Return the bytes.

### Memory: the index in RAM, the data on disk (as of 2026-09-06)

Until that date `Archive::open` did `std::fs::read` of the whole file. With the eight
archives of a full installation that's **about 1.3 GB** in RAM to answer a question about
one icon: on a machine with 8 GB and the game running, this is the most likely way the
app quietly dies, i.e. a violation of constraint 5.

Now `open` reads only the header and the index table, keeps the `File` open, and an
entry's bytes are read when needed. Two details the format forces on us:

- **A entry's compressed length is written nowhere.** The index only gives the starting
  offset; the decompressors stop on their own once they've produced `decompressed_len`
  bytes. To read one entry in isolation you still need an upper bound, and that bound is
  **the next offset** (the data is contiguous: verified on `fonts.a`, where the last
  chunk of entry 0 ends exactly where entry 1 begins). The last entry ends where the
  index table begins. A bound wider than the true one is harmless; a narrower one isn't,
  so two entries at the same offset share the same bound.
- **Reading is positional** (`seek_read` on Windows, `read_at` elsewhere), not `seek` +
  `read`: a single `ResourceSet` is shared by multiple Tauri commands, and the file
  cursor would be a race between threads.

**Declared threshold: opening every archive of a full installation must stay under
64 MB of heap.** The measurement is a real test — `crates/unpack/tests/streaming.rs`
installs a global allocator that counts live bytes — and today the peak is about
**1.4 MB** across eight archives and 19,473 entries. The threshold is deliberately wide:
it doesn't defend one line of code, it defends the order of magnitude (the index, unit
of MB) against the wrong one (the data, over a thousand MB).

**`extract_subset(wanted, cache_dir)`:** for each path: `read` → write to
`cache_dir/<path>` creating the folders (inside `cache_dir`, never outside) →
`extracted`; if `None` due to absence → `missing`; if decompression failed →
`DecompressFailed`; if writing failed → `WriteFailed`. No panic. Before writing, verify
that the normalized path stays inside `cache_dir` (no `..` escaping).

## Tests

### On real archives (integration, skipped if absent)

`samples/` (gitignored) holds real `fonts.a`, `config.a`, `filelist.txt`.

- **Hash**: `path_key("resources/achievements.xml").djb2` == word0 of one of the
  records in `config.a` (expected value derived from the archive in the test). Verifies
  djb2 **and** fnv.
- **Index**: `Archive::open("samples/config.a")` ok; magic/version/`count` read from the
  file; `entries().len() == count`.
- **read + LZW**: `read("resources/achievements.xml")` on `config.a` returns bytes of
  length `== decompressed_len` of the record and starting with `<` (or BOM+`<`). This
  nails down hash, layout, and decompression together.
- **Extraction**: `extract_subset(&config, &["resources/achievements.xml"], tmp)` writes
  a file whose content matches `read(...)`; `missing` empty.
- **Missing path**: `read("resources/nonexistent.xml")` → `None`; in `extract_subset` it
  ends up in `missing`, no error.

### Synthetic fixtures (always run)

A helper builds a minimal in-memory ARCH000 (header + records with known hashes/
positions; for the data, either a trivial stored entry or a minimal LZW chunk per the
confirmed variant):

- open ok, correct `entries`;
- wrong magic → `BadMagic`; 8-byte buffer → `TooShort`;
- a record that overruns the buffer → silently discarded, no panic, the others remain.

`hash.rs`'s unit tests check djb2/fnv against fixed expected values (once derived from
real records) and normalization (`\`→`/`, lowercase).

## Out of scope

- The **list of paths** to extract → passed by the caller (`catalog` or a data
  manifest).
- **Rebuilding names** from `filelist.txt` / full listing → not needed for targeted
  extraction.
- **Precedence between DLCs** (same path in multiple archives) → out of this round.
- **XML normalization** → `catalog`.

## Completion criteria

- `cargo test -p unpack` green with and without the samples in `samples/`.
- No writes outside the cache; `.a` files opened read-only.
- On real `config.a`: `read("resources/achievements.xml")` returns the correct
  decompressed XML — hash (djb2+fnv), record layout, and LZW confirmed together against
  real data.
- `open`/`read` never panic on a malformed/truncated archive or corrupted data.
- No hardcoded record count; no asset or `filelist.txt` in the crate.
