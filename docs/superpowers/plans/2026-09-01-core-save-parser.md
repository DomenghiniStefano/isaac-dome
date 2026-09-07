# core-save Parser Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A Rust crate `core-save` that reads the `.dat` files of *Isaac Repentance+* read-only and returns a structural model (raw sections + diff), degrading gracefully on malformed input instead of failing.

**Architecture:** Cargo workspace at the root; `core-save` as the first member, in `crates/core-save`. Hand-written parser using `u32::from_le_bytes` + slices, no parser combinator. The model derives `serde::Serialize` for the future Tauri bridge, but the crate has no dependency on Tauri.

**Tech Stack:** Rust (edition 2021), `serde` (the only non-std dependency), `cargo test`.

## Global Constraints

- **Read-only by construction:** no write function or API anywhere in the crate. The checksum is never read for recomputation, nor touched.
- **No hardcoded counts:** the number of entries in each section is read from the header. Only the *bytes-per-entry* table for `kind` is static.
- **Degrade, never fail:** a hard error only on bad magic or a file that's too short; every other inconsistency becomes a `Diagnostic` plus partial parsing. Never `panic!`/`unwrap` on untrusted input.
- **Magic:** `ISAACNGSAVE09R` (14 significant bytes, padded to 16 with spaces).
- **Section 10 (bestiary):** length = `fine-4 − offset`, **not** `count × 8`.
- **serde:** every model type (`Save`, `Section`, `Kind`, `Diagnostic`, `SaveDiff`) derives `Serialize`. `OpenError` does not (it contains `io::Error`).
- Windows environment, PowerShell shell; `cargo` commands are cross-shell.

---

## File Structure

```
Cargo.toml                      # [workspace], members = ["crates/*"]
crates/core-save/
  Cargo.toml                    # package + serde dependency
  src/
    lib.rs                      # declares the modules, re-exports the public API
    section.rs                  # Kind, bytes-per-entry table, Section
    parse.rs                    # Save, Diagnostic, OpenError, MAGIC, section walk
    diff.rs                     # SaveDiff, diff()
  tests/
    degradation.rs              # synthetic fixtures, always run
    real_saves.rs              # integration tests against real samples, skip if absent
```

---

### Task 1: Workspace and crate skeleton

**Files:**
- Create: `Cargo.toml`
- Create: `crates/core-save/Cargo.toml`
- Create: `crates/core-save/src/lib.rs`

**Interfaces:**
- Consumes: nothing.
- Produces: `core-save` crate that compiles with `cargo test -p core-save`.

- [ ] **Step 1: Create the workspace `Cargo.toml`**

```toml
[workspace]
resolver = "2"
members = ["crates/*"]
```

- [ ] **Step 2: Create `crates/core-save/Cargo.toml`**

```toml
[package]
name = "core-save"
version = "0.1.0"
edition = "2021"
description = "Read-only reader for The Binding of Isaac: Repentance+ save files"

[dependencies]
serde = { version = "1", features = ["derive"] }
```

- [ ] **Step 3: Create `crates/core-save/src/lib.rs` with a trivial test**

```rust
//! core-save — read-only, structural reading of Isaac Repentance+ `.dat` files.

#[cfg(test)]
mod smoke {
    #[test]
    fn crate_compiles() {
        assert_eq!(2 + 2, 4);
    }
}
```

- [ ] **Step 4: Build and test**

Run: `cargo test -p core-save`
Expected: PASS (1 test, `smoke::crate_compiles`), workspace resolves with no blocking warnings.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml crates/core-save/Cargo.toml crates/core-save/src/lib.rs
git commit -m "core-save: scheletro del crate e workspace"
```

---

### Task 2: `Kind` and the bytes-per-entry table

**Files:**
- Create: `crates/core-save/src/section.rs`
- Modify: `crates/core-save/src/lib.rs`

**Interfaces:**
- Consumes: nothing.
- Produces:
  - `enum Kind { Achievements, Counters, PerChar, Items, Unknown5, CardsPills, Challenges, Unknown8, Unknown9, Bestiary }` (derives `Debug, Clone, Copy, PartialEq, Eq, Serialize`)
  - `Kind::number(self) -> u32` (1..=10)
  - `Kind::from_number(n: u32) -> Option<Kind>`
  - `Kind::bytes_per_entry(self) -> Option<usize>` (`None` = variable, only for `Bestiary`)
  - `struct Section { kind: Kind, count: u32, f2: u32, offset: usize, bytes: Vec<u8> }` (derives `Debug, Clone, Serialize`)

- [ ] **Step 1: Write the tests at the bottom of `section.rs`**

Create `crates/core-save/src/section.rs` with ONLY the test block at the bottom (we'll fill in the top at step 3):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_and_from_number_are_inverse() {
        for n in 1u32..=10 {
            let k = Kind::from_number(n).expect("1..=10 is valid");
            assert_eq!(k.number(), n);
        }
        assert_eq!(Kind::from_number(0), None);
        assert_eq!(Kind::from_number(11), None);
    }

    #[test]
    fn bytes_per_entry_matches_format() {
        assert_eq!(Kind::Achievements.bytes_per_entry(), Some(1));
        assert_eq!(Kind::Items.bytes_per_entry(), Some(1));
        assert_eq!(Kind::CardsPills.bytes_per_entry(), Some(1));
        assert_eq!(Kind::Challenges.bytes_per_entry(), Some(1));
        assert_eq!(Kind::Unknown5.bytes_per_entry(), Some(1));
        assert_eq!(Kind::Counters.bytes_per_entry(), Some(4));
        assert_eq!(Kind::PerChar.bytes_per_entry(), Some(4));
        assert_eq!(Kind::Unknown8.bytes_per_entry(), Some(4));
        assert_eq!(Kind::Unknown9.bytes_per_entry(), Some(4));
        assert_eq!(Kind::Bestiary.bytes_per_entry(), None);
    }
}
```

- [ ] **Step 2: Declare the module and check that the tests fail to compile**

In `lib.rs`, below the `//! ...` line, add:

```rust
mod section;

pub use section::{Kind, Section};
```

Run: `cargo test -p core-save`
Expected: FAIL to compile (`Kind`/`Section` not defined).

- [ ] **Step 3: Implement `Kind` and `Section` above the test block**

At the top of `section.rs`, before the `#[cfg(test)]`:

```rust
use serde::Serialize;

/// The ten sections of the save. STRUCTURAL labels, not semantic ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Achievements,
    Counters,
    PerChar,
    Items,
    Unknown5,
    CardsPills,
    Challenges,
    Unknown8,
    Unknown9,
    Bestiary,
}

impl Kind {
    /// The section's sequence number as it appears in the file (1..=10).
    pub fn number(self) -> u32 {
        match self {
            Kind::Achievements => 1,
            Kind::Counters => 2,
            Kind::PerChar => 3,
            Kind::Items => 4,
            Kind::Unknown5 => 5,
            Kind::CardsPills => 6,
            Kind::Challenges => 7,
            Kind::Unknown8 => 8,
            Kind::Unknown9 => 9,
            Kind::Bestiary => 10,
        }
    }

    /// Maps the file's sequence number to a `Kind`. `None` if outside 1..=10.
    pub fn from_number(n: u32) -> Option<Kind> {
        Some(match n {
            1 => Kind::Achievements,
            2 => Kind::Counters,
            3 => Kind::PerChar,
            4 => Kind::Items,
            5 => Kind::Unknown5,
            6 => Kind::CardsPills,
            7 => Kind::Challenges,
            8 => Kind::Unknown8,
            9 => Kind::Unknown9,
            10 => Kind::Bestiary,
            _ => return None,
        })
    }

    /// Bytes per entry ON DISK. `None` = variable length up to `end-4`
    /// (bestiary only: the header's `count` doesn't count the records).
    pub fn bytes_per_entry(self) -> Option<usize> {
        match self {
            Kind::Achievements
            | Kind::Items
            | Kind::Unknown5
            | Kind::CardsPills
            | Kind::Challenges => Some(1),
            Kind::Counters | Kind::PerChar | Kind::Unknown8 | Kind::Unknown9 => Some(4),
            Kind::Bestiary => None,
        }
    }
}

/// A section read from the file: header + raw data bytes.
#[derive(Debug, Clone, Serialize)]
pub struct Section {
    pub kind: Kind,
    pub count: u32,
    pub f2: u32,
    pub offset: usize,
    pub bytes: Vec<u8>,
}
```

- [ ] **Step 4: Run the tests**

Run: `cargo test -p core-save`
Expected: PASS (smoke + `number_and_from_number_are_inverse` + `bytes_per_entry_matches_format`).

- [ ] **Step 5: Commit**

```bash
git add crates/core-save/src/section.rs crates/core-save/src/lib.rs
git commit -m "core-save: Kind, tabella byte-per-voce e Section"
```

---

### Task 3: Model types and magic/length validation

**Files:**
- Create: `crates/core-save/src/parse.rs`
- Modify: `crates/core-save/src/lib.rs`

**Interfaces:**
- Consumes: `Section`, `Kind` from `section.rs`.
- Produces:
  - `const MAGIC: &[u8; 14] = b"ISAACNGSAVE09R";`
  - `struct Save { unknown_0x10: u32, sections: Vec<Section>, diagnostics: Vec<Diagnostic> }` (derives `Debug, Clone, Serialize`)
  - `enum Diagnostic { UnexpectedKind { at, expected, found }, SectionOverrun { kind, at, needed, available }, TrailingBytes { at, len } }` (derives `Debug, Clone, PartialEq, Eq, Serialize`)
  - `enum OpenError { TooShort, BadMagic { found: [u8; 16] }, Io(std::io::Error) }` (derives `Debug`, `From<io::Error>`)
  - `Save::open(path) -> Result<Save, OpenError>`
  - `Save::parse(bytes: &[u8]) -> Result<Save, OpenError>` — in this task it only validates, returning empty `sections`/`diagnostics`.

- [ ] **Step 1: Write the tests at the bottom of `parse.rs`**

Create `crates/core-save/src/parse.rs` with ONLY the test block at the bottom:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// Valid header: magic (16) + unknown_0x10, then 4 dummy checksum bytes.
    fn minimal_valid(unknown: u32) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(b"ISAACNGSAVE09R  ");
        v.extend_from_slice(&unknown.to_le_bytes());
        v.extend_from_slice(&[0u8; 4]); // dummy checksum
        v
    }

    #[test]
    fn reads_unknown_0x10() {
        let save = Save::parse(&minimal_valid(0xDEADBEEF)).unwrap();
        assert_eq!(save.unknown_0x10, 0xDEADBEEF);
    }

    #[test]
    fn rejects_bad_magic() {
        let mut b = minimal_valid(0);
        b[0] = b'X';
        match Save::parse(&b) {
            Err(OpenError::BadMagic { found }) => assert_eq!(found[0], b'X'),
            other => panic!("expected BadMagic, found {other:?}"),
        }
    }

    #[test]
    fn rejects_too_short() {
        assert!(matches!(Save::parse(&[0u8; 8]), Err(OpenError::TooShort)));
    }
}
```

- [ ] **Step 2: Declare the module and check the failure**

In `lib.rs`, add below `mod section;`:

```rust
mod parse;

pub use parse::{Diagnostic, OpenError, Save};
```

Run: `cargo test -p core-save`
Expected: FAIL to compile (`Save`/`OpenError` not defined).

- [ ] **Step 3: Implement the types and validation above the test block**

At the top of `parse.rs`, before the `#[cfg(test)]`:

```rust
use std::path::Path;

use serde::Serialize;

use crate::section::Section;

/// The 14 significant bytes of the signature (followed by 2 padding spaces in the file).
pub const MAGIC: &[u8; 14] = b"ISAACNGSAVE09R";

/// The structural model of a save: raw sections + diagnostics.
#[derive(Debug, Clone, Serialize)]
pub struct Save {
    /// Il `u32` a 0x10, significato ignoto, esposto grezzo.
    pub unknown_0x10: u32,
    pub sections: Vec<Section>,
    pub diagnostics: Vec<Diagnostic>,
}

/// What went wrong without preventing a partial parse.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Diagnostic {
    UnexpectedKind { at: usize, expected: u32, found: u32 },
    SectionOverrun { kind: u32, at: usize, needed: usize, available: usize },
    TrailingBytes { at: usize, len: usize },
}

/// Hard error: there is nothing useful to return.
#[derive(Debug)]
pub enum OpenError {
    TooShort,
    BadMagic { found: [u8; 16] },
    Io(std::io::Error),
}

impl From<std::io::Error> for OpenError {
    fn from(e: std::io::Error) -> Self {
        OpenError::Io(e)
    }
}

fn read_u32(bytes: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([bytes[off], bytes[off + 1], bytes[off + 2], bytes[off + 3]])
}

impl Save {
    /// Legge e interpreta un file. Sola lettura.
    pub fn open(path: impl AsRef<Path>) -> Result<Save, OpenError> {
        let bytes = std::fs::read(path)?;
        Save::parse(&bytes)
    }

    /// Interprets a buffer already in memory. Errors only on magic/length;
    /// every other inconsistency becomes a `Diagnostic`.
    pub fn parse(bytes: &[u8]) -> Result<Save, OpenError> {
        if bytes.len() < 20 {
            return Err(OpenError::TooShort);
        }
        if &bytes[0..14] != MAGIC.as_slice() {
            let mut found = [0u8; 16];
            found.copy_from_slice(&bytes[0..16]);
            return Err(OpenError::BadMagic { found });
        }
        let unknown_0x10 = read_u32(bytes, 0x10);

        Ok(Save {
            unknown_0x10,
            sections: Vec::new(),
            diagnostics: Vec::new(),
        })
    }
}
```

- [ ] **Step 4: Run the tests**

Run: `cargo test -p core-save`
Expected: PASS (`reads_unknown_0x10`, `rejects_bad_magic`, `rejects_too_short`).

- [ ] **Step 5: Commit**

```bash
git add crates/core-save/src/parse.rs crates/core-save/src/lib.rs
git commit -m "core-save: modello Save/Diagnostic/OpenError e validazione magic"
```

---

### Task 4: Section walk with graceful degradation

**Files:**
- Modify: `crates/core-save/src/parse.rs` (body of `parse`)
- Create: `crates/core-save/tests/degradation.rs`

**Interfaces:**
- Consumes: `Save`, `Diagnostic`, `Kind`, `Section`.
- Produces: `Save::parse` populates `sections` in order and `diagnostics`; terminates the bestiary section at `fine-4`.

- [ ] **Step 1: Write the degradation fixtures as an integration test**

Create `crates/core-save/tests/degradation.rs`:

```rust
use core_save::{Diagnostic, Kind, Save};

/// Assembles a synthetic `.dat`. Each section: (kind, declared count, raw data).
/// `f2` is written as count*4, as in the real format.
fn build(unknown: u32, sections: &[(u32, u32, &[u8])]) -> Vec<u8> {
    let mut v = Vec::new();
    v.extend_from_slice(b"ISAACNGSAVE09R  ");
    v.extend_from_slice(&unknown.to_le_bytes());
    for &(kind, count, data) in sections {
        v.extend_from_slice(&kind.to_le_bytes());
        v.extend_from_slice(&count.wrapping_mul(4).to_le_bytes());
        v.extend_from_slice(&count.to_le_bytes());
        v.extend_from_slice(data);
    }
    v.extend_from_slice(&[0u8; 4]); // dummy checksum
    v
}

#[test]
fn parses_well_formed_sections() {
    // kind 1: 3 achievements (1 byte each); kind 2: 2 counters (4 bytes each).
    let counters: Vec<u8> = [7u32, 42u32].iter().flat_map(|v| v.to_le_bytes()).collect();
    let bytes = build(0, &[(1, 3, &[1, 0, 1]), (2, 2, &counters)]);

    let save = Save::parse(&bytes).unwrap();
    assert_eq!(save.sections.len(), 2);
    assert_eq!(save.sections[0].kind, Kind::Achievements);
    assert_eq!(save.sections[0].count, 3);
    assert_eq!(save.sections[0].bytes, vec![1, 0, 1]);
    assert_eq!(save.sections[1].kind, Kind::Counters);
    assert_eq!(save.sections[1].count, 2);
    assert!(save.diagnostics.is_empty(), "expected no diagnostics");
}

#[test]
fn bestiary_extends_to_end_ignoring_header_count() {
    // kind 10 with a bogus count (80) but only 16 bytes of real data (2 records of 8).
    let data = [0u8; 16];
    let bytes = build(0, &[(10, 80, &data)]);

    let save = Save::parse(&bytes).unwrap();
    assert_eq!(save.sections.len(), 1);
    let bestiary = &save.sections[0];
    assert_eq!(bestiary.kind, Kind::Bestiary);
    assert_eq!(bestiary.count, 80); // header preserved...
    assert_eq!(bestiary.bytes.len(), 16); // ...but length comes from the bytes, not from count
    assert!(save.diagnostics.is_empty());
}

#[test]
fn flags_a_section_that_overruns_the_buffer() {
    // kind 1 declares 100 entries of 1 byte, but supplies only 3.
    let bytes = build(0, &[(1, 100, &[0, 0, 0])]);

    let save = Save::parse(&bytes).unwrap();
    // The section is present, truncated to the available bytes.
    assert_eq!(save.sections.len(), 1);
    assert_eq!(save.sections[0].bytes.len(), 3);
    assert!(
        save.diagnostics.iter().any(|d| matches!(
            d,
            Diagnostic::SectionOverrun { kind: 1, needed: 100, available: 3, .. }
        )),
        "expected SectionOverrun, found {:?}",
        save.diagnostics
    );
}

#[test]
fn flags_an_unexpected_kind_but_keeps_going() {
    // First section declares kind 5 instead of 1: it's noted and parsing continues with the kind read.
    let bytes = build(0, &[(5, 2, &[0, 0]), (2, 1, &1u32.to_le_bytes())]);

    let save = Save::parse(&bytes).unwrap();
    assert!(
        save.diagnostics.iter().any(|d| matches!(
            d,
            Diagnostic::UnexpectedKind { expected: 1, found: 5, .. }
        )),
        "expected UnexpectedKind, found {:?}",
        save.diagnostics
    );
    assert_eq!(save.sections[0].kind, Kind::Unknown5);
    assert_eq!(save.sections.len(), 2);
}

#[test]
fn flags_trailing_bytes_before_checksum() {
    // A 2-byte section, then 3 leftover bytes, then the checksum.
    let bytes = build(0, &[(1, 2, &[0, 0, 9, 9, 9])]); // count=2 but 5 bytes of data
    // count*1 = 2 consumes 2 bytes; 3 bytes remain before the checksum.
    let save = Save::parse(&bytes).unwrap();
    assert_eq!(save.sections[0].bytes.len(), 2);
    assert!(
        save.diagnostics.iter().any(|d| matches!(d, Diagnostic::TrailingBytes { len: 3, .. })),
        "expected TrailingBytes, found {:?}",
        save.diagnostics
    );
}
```

- [ ] **Step 2: Run and confirm the failure**

Run: `cargo test -p core-save --test degradation`
Expected: FAIL (parse still returns empty `sections` → assertions fail).

- [ ] **Step 3: Replace the body of `parse` with the section walk**

In `parse.rs`, replace the block `Ok(Save { unknown_0x10, sections: Vec::new(), diagnostics: Vec::new() })` with:

```rust
        let end = bytes.len() - 4;
        let mut off = 0x14usize;
        let mut sections = Vec::new();
        let mut diagnostics = Vec::new();
        let mut expected = 1u32;

        while off + 12 <= end {
            let found_kind = read_u32(bytes, off);
            let f2 = read_u32(bytes, off + 4);
            let count = read_u32(bytes, off + 8);

            if found_kind != expected {
                diagnostics.push(Diagnostic::UnexpectedKind {
                    at: off,
                    expected,
                    found: found_kind,
                });
            }

            let kind = match crate::section::Kind::from_number(found_kind) {
                Some(k) => k,
                None => {
                    // Unknown kind: we don't know the entry size, so we stop here.
                    diagnostics.push(Diagnostic::UnexpectedKind {
                        at: off,
                        expected,
                        found: found_kind,
                    });
                    break;
                }
            };

            let data = off + 12;
            let available = end - data;
            let wanted = match kind.bytes_per_entry() {
                None => available,                       // bestiary: up to end-4
                Some(sz) => (count as usize).saturating_mul(sz),
            };

            let len = if wanted > available {
                diagnostics.push(Diagnostic::SectionOverrun {
                    kind: found_kind,
                    at: data,
                    needed: wanted,
                    available,
                });
                available
            } else {
                wanted
            };

            sections.push(Section {
                kind,
                count,
                f2,
                offset: data,
                bytes: bytes[data..data + len].to_vec(),
            });

            if len != wanted {
                break; // truncated: we can't trust the rest
            }

            off = data + len;
            expected += 1;
        }

        if off < end {
            diagnostics.push(Diagnostic::TrailingBytes {
                at: off,
                len: end - off,
            });
        }

        Ok(Save {
            unknown_0x10,
            sections,
            diagnostics,
        })
```

- [ ] **Step 4: Run the tests**

Run: `cargo test -p core-save`
Expected: PASS (unit tests from parse.rs + all fixtures in `degradation.rs`).

- [ ] **Step 5: Commit**

```bash
git add crates/core-save/src/parse.rs crates/core-save/tests/degradation.rs
git commit -m "core-save: walk delle sezioni con diagnostica e degradazione"
```

---

### Task 5: Typed accessors

**Files:**
- Modify: `crates/core-save/src/parse.rs` (new `impl Save`)
- Modify: `crates/core-save/tests/degradation.rs` (accessor tests)

**Interfaces:**
- Consumes: `Save`, `Kind`, `Section`.
- Produces:
  - `Save::section(&self, kind: Kind) -> Option<&Section>`
  - `Save::flags(&self, kind: Kind) -> Option<Vec<bool>>` (only for 1-byte sections)
  - `Save::u32s(&self, kind: Kind) -> Option<Vec<u32>>` (only for 4-byte sections)
  - `Save::bestiary(&self) -> Option<&[u8]>`

- [ ] **Step 1: Add the accessor tests at the bottom of `degradation.rs`**

```rust
#[test]
fn accessors_expose_typed_views() {
    let counters: Vec<u8> = [7u32, 42u32].iter().flat_map(|v| v.to_le_bytes()).collect();
    let bytes = build(0, &[(1, 3, &[1, 0, 5]), (2, 2, &counters)]);
    let save = Save::parse(&bytes).unwrap();

    assert_eq!(save.section(Kind::Achievements).unwrap().count, 3);
    assert_eq!(save.flags(Kind::Achievements), Some(vec![true, false, true]));
    assert_eq!(save.u32s(Kind::Counters), Some(vec![7, 42]));

    // Wrong type for the section → None (not a panic).
    assert_eq!(save.flags(Kind::Counters), None);
    assert_eq!(save.u32s(Kind::Achievements), None);
    // Section absent → None.
    assert_eq!(save.flags(Kind::Items), None);
}

#[test]
fn bestiary_accessor_returns_raw_bytes() {
    let data = [1u8; 24];
    let bytes = build(0, &[(10, 3, &data)]);
    let save = Save::parse(&bytes).unwrap();
    assert_eq!(save.bestiary(), Some(&[1u8; 24][..]));
}
```

- [ ] **Step 2: Run and confirm the failure**

Run: `cargo test -p core-save --test degradation`
Expected: FAIL to compile (`section`/`flags`/`u32s`/`bestiary` not defined).

- [ ] **Step 3: Implement the accessors in `parse.rs`**

Add a second `impl Save` block (after the one with `open`/`parse`, before the `#[cfg(test)]`):

```rust
impl Save {
    /// The first section of a given `kind`, if present.
    pub fn section(&self, kind: crate::section::Kind) -> Option<&Section> {
        self.sections.iter().find(|s| s.kind == kind)
    }

    /// Boolean view of a 1-byte section (`b != 0`). `None` if the section
    /// isn't present or isn't 1 byte per entry.
    pub fn flags(&self, kind: crate::section::Kind) -> Option<Vec<bool>> {
        if kind.bytes_per_entry() != Some(1) {
            return None;
        }
        let s = self.section(kind)?;
        Some(s.bytes.iter().map(|&b| b != 0).collect())
    }

    /// Little-endian `u32` view of a 4-byte section. `None` if the section
    /// isn't present or isn't 4 bytes per entry.
    pub fn u32s(&self, kind: crate::section::Kind) -> Option<Vec<u32>> {
        if kind.bytes_per_entry() != Some(4) {
            return None;
        }
        let s = self.section(kind)?;
        Some(
            s.bytes
                .chunks_exact(4)
                .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect(),
        )
    }

    /// Raw bytes of the bestiary (a multiple of 8). Interpreting the record
    /// belongs to the `completion` module, not to this crate.
    pub fn bestiary(&self) -> Option<&[u8]> {
        self.section(crate::section::Kind::Bestiary)
            .map(|s| s.bytes.as_slice())
    }
}
```

- [ ] **Step 4: Run the tests**

Run: `cargo test -p core-save`
Expected: PASS (including `accessors_expose_typed_views`, `bestiary_accessor_returns_raw_bytes`).

- [ ] **Step 5: Commit**

```bash
git add crates/core-save/src/parse.rs crates/core-save/tests/degradation.rs
git commit -m "core-save: accessori tipati flags/u32s/section/bestiary"
```

---

### Task 6: `diff` between two saves

**Files:**
- Create: `crates/core-save/src/diff.rs`
- Modify: `crates/core-save/src/lib.rs`
- Modify: `crates/core-save/tests/degradation.rs` (diff test)

**Interfaces:**
- Consumes: `Save`, `Kind` (via the `flags`/`u32s` accessors).
- Produces:
  - `struct SaveDiff { achievements: Vec<usize>, items: Vec<usize>, challenges: Vec<usize>, cards_pills: Vec<usize>, counters: Vec<(usize, u32, u32)> }` (derives `Debug, Clone, Default, PartialEq, Eq, Serialize`)
  - `fn diff(a: &Save, b: &Save) -> SaveDiff`

- [ ] **Step 1: Write the diff test at the bottom of `degradation.rs`**

```rust
#[test]
fn diff_reports_newly_set_flags_and_changed_counters() {
    use core_save::diff;

    let ca: Vec<u8> = [1u32, 1u32].iter().flat_map(|v| v.to_le_bytes()).collect();
    let cb: Vec<u8> = [1u32, 9u32].iter().flat_map(|v| v.to_le_bytes()).collect();

    // a: achievement [1,0,0]; b: [1,0,1] → indice 2 nuovo. Contatore 1: 1→9.
    let a = Save::parse(&build(0, &[(1, 3, &[1, 0, 0]), (2, 2, &ca)])).unwrap();
    let b = Save::parse(&build(0, &[(1, 3, &[1, 0, 1]), (2, 2, &cb)])).unwrap();

    let d = diff(&a, &b);
    assert_eq!(d.achievements, vec![2]);
    assert_eq!(d.items, Vec::<usize>::new());
    assert_eq!(d.counters, vec![(1, 1, 9)]);
}
```

- [ ] **Step 2: Declare the module and confirm the failure**

In `lib.rs`, add:

```rust
mod diff;

pub use diff::{diff, SaveDiff};
```

Run: `cargo test -p core-save --test degradation`
Expected: FAIL to compile (`diff`/`SaveDiff` not defined).

- [ ] **Step 3: Implement `diff.rs`**

```rust
use serde::Serialize;

use crate::parse::Save;
use crate::section::Kind;

/// What changed going from save `a` to save `b`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct SaveDiff {
    /// Indici passati da spento ad acceso in `b`.
    pub achievements: Vec<usize>,
    pub items: Vec<usize>,
    pub challenges: Vec<usize>,
    pub cards_pills: Vec<usize>,
    /// (index, value in `a`, value in `b`) where the counters differ.
    pub counters: Vec<(usize, u32, u32)>,
}

/// Structural diff. No semantic knowledge: only indices and values.
pub fn diff(a: &Save, b: &Save) -> SaveDiff {
    let newly_set = |kind: Kind| -> Vec<usize> {
        let fa = a.flags(kind).unwrap_or_default();
        let fb = b.flags(kind).unwrap_or_default();
        fb.iter()
            .enumerate()
            .filter(|&(i, &on)| on && !fa.get(i).copied().unwrap_or(false))
            .map(|(i, _)| i)
            .collect()
    };

    let ca = a.u32s(Kind::Counters).unwrap_or_default();
    let cb = b.u32s(Kind::Counters).unwrap_or_default();
    let counters = cb
        .iter()
        .enumerate()
        .filter_map(|(i, &y)| {
            let x = ca.get(i).copied().unwrap_or(0);
            (x != y).then_some((i, x, y))
        })
        .collect();

    SaveDiff {
        achievements: newly_set(Kind::Achievements),
        items: newly_set(Kind::Items),
        challenges: newly_set(Kind::Challenges),
        cards_pills: newly_set(Kind::CardsPills),
        counters,
    }
}
```

- [ ] **Step 4: Run the tests**

Run: `cargo test -p core-save`
Expected: PASS (including `diff_reports_newly_set_flags_and_changed_counters`).

- [ ] **Step 5: Commit**

```bash
git add crates/core-save/src/diff.rs crates/core-save/src/lib.rs crates/core-save/tests/degradation.rs
git commit -m "core-save: diff strutturale tra due salvataggi"
```

---

### Task 7: Integration tests against real saves

**Files:**
- Create: `crates/core-save/tests/real_saves.rs`

**Interfaces:**
- Consumes: the whole public API (`Save`, `Kind`, `diff`).
- Produces: validation against the real samples in `samples/` (gitignored); the tests skip themselves with a note if the files are missing.

- [ ] **Step 1: Write the integration tests against the real samples**

Create `crates/core-save/tests/real_saves.rs`:

```rust
use core_save::{diff, Kind, Save};

/// Loads a sample from `samples/` at the repo root. `None` if absent
/// (samples are gitignored: CI for whoever doesn't have them stays green).
fn load(name: &str) -> Option<Vec<u8>> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../samples")
        .join(name);
    std::fs::read(path).ok()
}

const S2025: &str = "20250626.rep+persistentgamedata1.dat";
const LIVE: &str = "live.rep+persistentgamedata1.dat";

#[test]
fn real_save_has_ten_sections_in_order() {
    let Some(bytes) = load(LIVE) else {
        eprintln!("skip: {LIVE} missing from samples/");
        return;
    };
    let save = Save::parse(&bytes).unwrap();
    let kinds: Vec<u32> = save.sections.iter().map(|s| s.kind.number()).collect();
    assert_eq!(kinds, (1..=10).collect::<Vec<_>>());
    assert!(
        save.diagnostics.is_empty(),
        "a real save must not produce diagnostics: {:?}",
        save.diagnostics
    );
}

#[test]
fn achievement_count_is_read_from_file_not_hardcoded() {
    let (Some(b2025), Some(blive)) = (load(S2025), load(LIVE)) else {
        eprintln!("skip: real samples missing from samples/");
        return;
    };
    let s2025 = Save::parse(&b2025).unwrap();
    let slive = Save::parse(&blive).unwrap();
    assert_eq!(s2025.section(Kind::Achievements).unwrap().count, 641);
    assert_eq!(slive.section(Kind::Achievements).unwrap().count, 642);
}

#[test]
fn stable_section_counts_match_the_format() {
    let Some(bytes) = load(LIVE) else {
        eprintln!("skip: {LIVE} missing from samples/");
        return;
    };
    let save = Save::parse(&bytes).unwrap();
    let count = |k: Kind| save.section(k).unwrap().count;
    assert_eq!(count(Kind::Counters), 523);
    assert_eq!(count(Kind::PerChar), 14);
    assert_eq!(count(Kind::Items), 733);
    assert_eq!(count(Kind::Unknown5), 7);
    assert_eq!(count(Kind::CardsPills), 104);
    assert_eq!(count(Kind::Challenges), 46);
    assert_eq!(count(Kind::Unknown8), 27);
    assert_eq!(count(Kind::Unknown9), 2);
    assert_eq!(save.u32s(Kind::Counters).unwrap().len(), 523);
}

#[test]
fn bestiary_length_follows_bytes_not_header_count() {
    let Some(bytes) = load(LIVE) else {
        eprintln!("skip: {LIVE} missing from samples/");
        return;
    };
    let save = Save::parse(&bytes).unwrap();
    let bestiary = save.section(Kind::Bestiary).unwrap();
    assert!(bestiary.bytes.len() % 8 == 0, "8-byte records");
    assert!(
        bestiary.bytes.len() as u32 / 8 != bestiary.count,
        "the header's count is not the number of records"
    );
    // The last section reaches exactly the checksum: no leftover.
    let end = bestiary.offset + bestiary.bytes.len();
    assert_eq!(end, bytes.len() - 4);
}

#[test]
fn diff_2025_to_live_is_coherent() {
    let (Some(b2025), Some(blive)) = (load(S2025), load(LIVE)) else {
        eprintln!("skip: real samples missing from samples/");
        return;
    };
    let d = diff(&Save::parse(&b2025).unwrap(), &Save::parse(&blive).unwrap());
    // The patch added an achievement (641 → 642): index 641 is new.
    assert!(
        d.achievements.contains(&641),
        "expected the new achievement 641, found {:?}",
        d.achievements
    );
    // Between June 2025 and August 2026 the game's counters advanced.
    assert!(!d.counters.is_empty(), "the counters should have changed");
}
```

- [ ] **Step 2: Run the tests**

Run: `cargo test -p core-save --test real_saves -- --nocapture`
Expected: PASS. With the samples present in `samples/`, all the assertions run; without them, the tests print `skip: ...` and pass.

> Note for the executor: the samples `20250626.rep+persistentgamedata1.dat` and `live.rep+persistentgamedata1.dat` were already copied into `samples/` during brainstorming. If they're missing, see the spec (the "Test" section) for where to get them; the tests skip themselves.

- [ ] **Step 3: Verify the whole suite**

Run: `cargo test -p core-save`
Expected: PASS — all unit tests, `degradation`, `real_saves`.

- [ ] **Step 4: Commit**

```bash
git add crates/core-save/tests/real_saves.rs
git commit -m "core-save: integration test sui salvataggi reali (skip se assenti)"
```

---

## Self-Review

**Spec coverage:**
- Repo structure / workspace → Task 1. ✓
- `Kind` + bytes-per-entry table, no hardcoded counts → Task 2. ✓
- `Save`/`Section`/`Diagnostic`/`OpenError` model, magic/length → Task 3. ✓
- Walk algorithm + graceful degradation (overrun, unexpected kind, trailing, bestiary at fine-4) → Task 4. ✓
- `flags`/`u32s`/`section`/raw `bestiary` accessors → Task 5. ✓
- `diff`/`SaveDiff` → Task 6. ✓
- Synthetic fixtures always active → Task 4/5/6 (`tests/degradation.rs`). ✓
- Real integration tests with skip, 641 vs 642, section 10, diff → Task 7. ✓
- serde on the whole model, no Tauri → derives in every task that defines types; no Tauri dependency in `Cargo.toml`. ✓
- Read-only, no write API → no task introduces write methods; `bytes` is owned and immutable. ✓
- Multi-corpus validation → set up via `load()` in Task 7 (just add samples and assertions). ✓

**Placeholder scan:** no TBD/TODO; every code step shows the complete code. ✓

**Type consistency:** `Kind`, `Section`, `Save`, `Diagnostic`, `OpenError`, `SaveDiff`, `diff`, `read_u32` used with consistent signatures across tasks; `flags`/`u32s` always return `Option<Vec<_>>`; `bytes_per_entry` is the single source of truth for a section's type (used by the walk, the accessors, and the diff). ✓
