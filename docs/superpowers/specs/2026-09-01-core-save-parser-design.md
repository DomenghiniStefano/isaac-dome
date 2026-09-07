# core-save — save file parser (design)

**Date:** 2026-09-01
**Sub-project:** M1 · first piece
**Status:** approved in brainstorming, pending spec review

## Context

First sub-project of M1. `core-save` is the Rust crate that reads the `.dat` files of
*The Binding of Isaac: Repentance+* in **read-only** mode and returns a structural model.
It's the most isolated foundation of M1: no dependency on Tauri, on disk (beyond reading
the file) or on the UI, so it's entirely testable first.

The format was already decoded in M0 and verified in this session against the real saves
present on the machine (2025 backup and live 2026 save). The reference implementation is
`reference/isaac_save.py`; this crate translates it into Rust.

### Module boundary (decided)

`core-save` stays **raw**: it only does structural parsing (sections → bool/u32/blob +
diff). Semantic labeling — counter names, the marks matrix, %Dead God, the logic in
`reference/isaac_counters.py` — will live in a separate module (`completion`), out of
scope for this spec. Reason: keep the crate that touches the bytes small, locked to
read-only, and free of game knowledge.

## Applicable non-negotiable constraints

1. **Read-only by construction.** No write API in the crate. The checksum is never read
   to be recalculated nor touched. No code path capable of corrupting someone's profile
   must exist.
2. **No hardcoded counts.** The entry count of each section is read from the header in
   the file. Verified: section 1 declares 641 entries in the June 2025 save and 642 in
   2026.
3. **Degrade, never fail.** If a section can't be read, the crate returns whatever it
   managed to read so far plus a diagnostic of what was skipped. Never panic on malformed
   input.

## Format (verified against real data)

```
0x00   "ISAACNGSAVE09R  "   signature, 16 bytes (14 significant + 2 spaces)
0x10   u32                  changes on every save, meaning unknown
0x14   first section header
...
end-4  checksum, 4 bytes    CRC32 with a custom polynomial, unidentified, never used
```

Section header = 3 × little-endian u32: `kind` (1..10), `f2` (= count × 4, "in memory"
size), `count` (number of entries). The data follows: `count` entries, whose **on-disk**
size depends on the section.

| kind | content                | bytes/entry |         notes                                    |
|-----:|-------------------------|:-----------:|---------------------------------------------------|
| 1    | achievements and secrets | 1          | 641 (2025) → 642 (2026)                           |
| 2    | counters and marks      | 4           | 523 entries                                       |
| 3    | per-character           | 4           | 14 entries, to be identified                      |
| 4    | item collection         | 1           | 733 entries                                       |
| 5    | to be identified        | 1           | 7 entries                                         |
| 6    | cards and pills         | 1           | 104 entries                                       |
| 7    | challenges              | 1           | 46 entries                                        |
| 8    | to be identified        | 4           | 27 entries                                        |
| 9    | to be identified        | 4           | 2 entries                                         |
| 10   | bestiary                | 8           | **length = end-4 − offset, NOT count × 8**        |

**Verified quirk of section 10:** the header declares `count = 80`, but the real data
extends all the way to `end-4` (1320 8-byte records in the live save). The header's
`count` is not the number of records; the length is derived from `end − offset`. A parser
that trusts `count` here is off by ~10 KB. Both real saves line up exactly with `end-4`
under this rule.

## Approach chosen

**Pure hand-written std.** `u32::from_le_bytes` + slices; no parser combinator (`nom`,
`binrw`) and no Kaitai-generated code. The format is three integers and a slice of bytes:
adding machinery here is complexity for free, and generated code doesn't sit well with
the "degrade, never fail" constraint. The only non-std dependency: `serde` (derive), for
the future Tauri bridge — but no dependency on Tauri in the crate.

Alternatives discarded: **Kaitai** (external tool + rigid generated code for 80 lines of
logic); **`nom`/`binrw`** (explicitly discouraged in the project document).

## Repo structure

**Cargo workspace** at the root, so future M1-and-beyond crates (`discovery`, `unpack`,
`graph`, `store`) and later `src-tauri` join as members without restructuring anything.

```
Cargo.toml            # [workspace], members = ["crates/*"]
crates/
  core-save/
    Cargo.toml
    src/
      lib.rs          # public API, re-exports the types
      section.rs      # Kind, byte-per-entry table, Section
      parse.rs        # section walk + degradation
      diff.rs         # SaveDiff
    tests/
      real_saves.rs   # integration test against real samples (skipped if absent)
      degradation.rs  # synthetic fixtures, always run
```

## Public API

```rust
pub struct Save {
    pub unknown_0x10: u32,          // the u32 at 0x10, meaning unknown, exposed raw
    pub sections: Vec<Section>,     // in reading order
    pub diagnostics: Vec<Diagnostic>,
}

pub struct Section {
    pub kind: Kind,
    pub count: u32,                 // from the header, never hardcoded
    pub f2: u32,                    // from the header ("in memory" = count × 4)
    pub offset: usize,              // offset of the data in the buffer
    pub bytes: Vec<u8>,             // the section's raw data
}

#[repr(u32)]
pub enum Kind {                     // STRUCTURAL labels, not semantic
    Achievements = 1, Counters = 2, PerChar = 3, Items = 4, Unknown5 = 5,
    CardsPills = 6, Challenges = 7, Unknown8 = 8, Unknown9 = 9, Bestiary = 10,
}

pub enum Diagnostic {
    UnexpectedKind { at: usize, expected: u32, found: u32 },
    SectionOverrun { kind: u32, at: usize, needed: usize, available: usize },
    TrailingBytes  { at: usize, len: usize },   // leftover before the checksum
}

pub enum OpenError { TooShort, BadMagic { found: [u8; 16] }, Io(std::io::Error) }

impl Save {
    pub fn open(path: impl AsRef<Path>) -> Result<Save, OpenError>;
    pub fn parse(bytes: &[u8]) -> Result<Save, OpenError>;   // error only on magic/length

    pub fn section(&self, kind: Kind) -> Option<&Section>;
    pub fn flags(&self, kind: Kind) -> Option<Vec<bool>>;    // 1-byte sections: b != 0
    pub fn u32s(&self, kind: Kind) -> Option<Vec<u32>>;      // 4-byte sections
    pub fn bestiary(&self) -> Option<&[u8]>;                 // raw bytes for now (see below)
}

pub fn diff(a: &Save, b: &Save) -> SaveDiff;

pub struct SaveDiff {
    pub achievements: Vec<usize>,   // indices that went from off to on in b
    pub items: Vec<usize>,
    pub challenges: Vec<usize>,
    pub cards_pills: Vec<usize>,
    pub counters: Vec<(usize, u32, u32)>,   // (index, before, after) where it changes
}
```

All model types derive `Serialize` (serde) for the future Tauri bridge. The crate does
not depend on Tauri.

### Bestiary: raw bytes for now (decided)

`bestiary()` exposes the raw bytes (a multiple of 8). The internal structure of the
key/value record is still open (M0), and interpreting it belongs to the `completion`
module anyway, not to `core-save`. Consistent with "core-save stays raw".

## Parsing algorithm

1. If `bytes.len() < 20` → `OpenError::TooShort`. If the first 14 bytes ≠
   `ISAACNGSAVE09R` → `OpenError::BadMagic`.
2. Read `unknown_0x10` = u32 at `0x10`. Set `off = 0x14`, `end = bytes.len() - 4`.
3. While there's room left for a header (`off + 12 <= end`), sequential reading with an
   `expected` value that advances 1, 2, 3, …:
   - Read three u32 at `off`.
   - If the `kind` read ≠ expected → `Diagnostic::UnexpectedKind`; continue using the
     kind read (degradation: trust the file, note the surprise). The bestiary, of
     variable length and always the last section, is exempt from this check.
   - `data = off + 12`. On-disk length:
     - bestiary → `end - data` (verified rule);
     - otherwise → `count × bytes_per_entry(kind)` (with `saturating_mul`, so a hostile
       count doesn't overflow).
   - If the wanted length > bytes available (`end - data`) →
     `Diagnostic::SectionOverrun`, clamp to the bytes available, save the truncated
     section and stop.
   - Save the `Section`; `off = data + len`.
4. If at the end of the loop `off < end` → `Diagnostic::TrailingBytes` (covers both
   leftover data and a header truncated to less than 12 bytes before the checksum).

No step panics on short or inconsistent input: every byte access is checked beforehand.

## Degradation (constraint #5)

- **Hard error only** for wrong magic or a file that's too short — cases where there is
  nothing useful to return.
- Everything else is `Diagnostic` + partial parsing. The app receives the readable
  sections and the list of what was skipped, and decides what to show and what to flag
  as missing.

## Tests (TDD)

Implementation order follows the tests: red first, then green.

### Synthetic fixtures (always run, even without samples)

Built in-code by a helper that assembles a minimal valid `.dat` buffer, then corrupts it:

- valid magic, tiny sections, a dummy checksum → `parse` ok, correct sections;
- wrong magic → `OpenError::BadMagic`;
- an 8-byte buffer → `OpenError::TooShort`;
- a section whose `count` overruns the buffer → `Diagnostic::SectionOverrun`, no panic,
  previous sections present;
- kind out of sequence → `Diagnostic::UnexpectedKind`, parsing continues;
- extra bytes before the checksum → `Diagnostic::TrailingBytes`.

### Integration test against real samples (skipped if absent)

The samples live in `samples/` (gitignored). A helper skips the test with a note if the
file isn't there, so CI stays green for anyone who clones without the data.

Against `samples/20250626.rep+persistentgamedata1.dat` and
`samples/live.rep+persistentgamedata1.dat`:

- magic = `ISAACNGSAVE09R`, `parse` ok, ten sections in order 1..10;
- **count from the file**: section 1 = 641 in the 2025 save, 642 in the live one
  (explicit assertion against hardcoding);
- sections 2/3/4/6/7/8/9 with the expected counts (523/14/733/104/46/27/2);
- **section 10**: length = `end-4 − offset`, a multiple of 8, and the header's `count`
  (80) ≠ the actual number of records;
- the last section ends exactly at `end-4` (no `TrailingBytes`);
- `flags(Achievements).len()` == the section's count; `u32s(Counters).len()` == 523.

Diff `2025 → live`:

- `achievements` contains at least the new index that appeared with the patch;
- `counters` non-empty (game counters advanced between the two dates).

### Multi-corpus validation

Every additional real save (other users, other DLCs, the `rep_` format without `+`)
comes in as one more gitignored sample with its own assertions, always behind the same
skip helper. A second user with different DLCs is the strongest proof of the "count from
the file" invariant, because it varies the counts across profiles and not just across
dates of the same profile. Full coverage only comes with variety (DLCs owned, Steam
Cloud on/off, multiple slots): each extra corpus is net gain, none alone is "every
installation".

## Out of scope

- Labeling of counters and marks, completion matrix, %Dead God → `completion` module.
- Interpretation of the bestiary record → `completion`.
- Identification of sections 3/5/8/9 → M0 data work, not code.
- `discovery`, `unpack`, Tauri bridge, UI → later M1 sub-projects.

## Completion criteria

- `cargo test -p core-save` green with and without the samples in `samples/`.
- No write function in the API or in the crate's body.
- No hardcoded entry count in the code (only the byte-per-kind table).
- Both real samples parse fully and line up with `end-4`; the 2025→live diff is
  consistent.
