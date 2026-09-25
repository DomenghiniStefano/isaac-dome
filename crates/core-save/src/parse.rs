use std::path::Path;

use serde::Serialize;

use crate::section::{Kind, Section};

/// The 14 significant bytes of the signature (followed by 2 padding spaces in the file).
pub const MAGIC: &[u8; 14] = b"ISAACNGSAVE09R";

/// The structural model of a save: raw sections + diagnostics.
#[derive(Debug, Clone)]
pub struct Save {
    /// The `u32` at 0x10, meaning unknown, exposed raw.
    pub unknown_0x10: u32,
    pub sections: Vec<Section>,
    pub diagnostics: Vec<Diagnostic>,
}

/// What went wrong without preventing a partial parse.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Diagnostic {
    UnexpectedKind {
        at: usize,
        expected: u32,
        found: u32,
    },
    SectionOverrun {
        kind: u32,
        at: usize,
        needed: usize,
        available: usize,
    },
    TrailingBytes {
        at: usize,
        len: usize,
    },
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

/// Where the first section header is: after the 16-byte signature and the `u32` at 0x10.
const FIRST_HEADER: usize = 0x14;

/// What reading the header at one offset came to.
enum Step {
    /// No room for another header before the checksum: the reading is over.
    Done,
    /// A section read whole, and where the next header is.
    Next {
        section: Section,
        diagnostics: Vec<Diagnostic>,
        next: usize,
    },
    /// The reading stops here: a kind with no known entry size, or a section cut short —
    /// kept, truncated, because what it holds is still worth reading, but nothing after it
    /// can be trusted.
    Stop {
        section: Option<Section>,
        diagnostics: Vec<Diagnostic>,
    },
}

const HEADER_LEN: usize = 12;

/// The three `u32` of a section header, as the file wrote them.
struct Header {
    kind: u32,
    declared_size: u32,
    count: u32,
}

/// The header at `off`, or `None` when fewer than 12 bytes are left before the checksum.
fn read_header(body: &[u8], off: usize) -> Option<Header> {
    let raw = body.get(off..)?.first_chunk::<HEADER_LEN>()?;
    let word = |i: usize| u32::from_le_bytes([raw[i], raw[i + 1], raw[i + 2], raw[i + 3]]);
    Some(Header {
        kind: word(0),
        declared_size: word(4),
        count: word(8),
    })
}

/// How many bytes a section of `kind` asks for, given the `available` ones before the checksum.
/// The bestiary has no fixed entry size and runs to the checksum.
fn wanted_len(kind: Kind, count: u32, available: usize) -> usize {
    match kind.bytes_per_entry() {
        None => available,
        Some(size) => (count as usize).saturating_mul(size),
    }
}

/// The section whose header is at `off`. `body` is the file without its checksum, so nothing
/// read here can reach into it. Every read is a `.get()`: a header or a section the file is too
/// short for is a `Step`, never a panic.
fn read_section(body: &[u8], off: usize, expected: u32) -> Step {
    let Some(header) = read_header(body, off) else {
        return Step::Done;
    };
    let unexpected = Diagnostic::UnexpectedKind {
        at: off,
        expected,
        found: header.kind,
    };
    let Some(kind) = Kind::from_number(header.kind) else {
        // Unknown kind: we don't know the entry size, so we stop here.
        return Step::Stop {
            section: None,
            diagnostics: vec![unexpected],
        };
    };
    // The bestiary (the only variable-length section) doesn't rely on the sequence number to
    // be sized, and by format it's always the last section; we exempt it from the sequence
    // check so it doesn't raise a spurious UnexpectedKind when it shows up on its own (e.g. in
    // fixtures).
    let out_of_sequence = (header.kind != expected && kind.bytes_per_entry().is_some())
        .then_some(unexpected)
        .into_iter();

    let data = off + HEADER_LEN;
    let available = body.len().saturating_sub(data);
    let wanted = wanted_len(kind, header.count, available);
    let len = wanted.min(available);
    let section = Section {
        kind,
        count: header.count,
        declared_size: header.declared_size,
        offset: data,
        bytes: body
            .get(data..data + len)
            .map(<[u8]>::to_vec)
            .unwrap_or_default(),
    };

    if wanted > available {
        let overrun = Diagnostic::SectionOverrun {
            kind: header.kind,
            at: data,
            needed: wanted,
            available,
        };
        // Truncated: we can't trust the rest.
        return Step::Stop {
            section: Some(section),
            diagnostics: out_of_sequence.chain([overrun]).collect(),
        };
    }
    Step::Next {
        section,
        diagnostics: out_of_sequence.collect(),
        next: data + len,
    }
}

/// How far the section headers have been read, and what they held.
struct Reading {
    off: usize,
    /// The kind the next header should carry: one after the kind the last one **did** carry,
    /// not after the one that was expected — so one missing section is one diagnostic, not one
    /// per section after it.
    expected: u32,
    sections: Vec<Section>,
    diagnostics: Vec<Diagnostic>,
}

impl Reading {
    fn start() -> Reading {
        Reading {
            off: FIRST_HEADER,
            expected: 1,
            sections: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    /// The reading after taking in `step`, and whether it goes on.
    fn take(mut self, step: Step) -> (Reading, bool) {
        match step {
            Step::Done => (self, false),
            Step::Next {
                section,
                diagnostics,
                next,
            } => {
                self.expected = section.kind.number().saturating_add(1);
                self.sections.push(section);
                self.diagnostics.extend(diagnostics);
                self.off = next;
                (self, true)
            }
            Step::Stop {
                section,
                diagnostics,
            } => {
                self.sections.extend(section);
                self.diagnostics.extend(diagnostics);
                (self, false)
            }
        }
    }

    /// Whatever the headers did not account for, from the header the reading stopped at.
    fn trailing(&self, body: &[u8]) -> Option<Diagnostic> {
        (self.off < body.len()).then(|| Diagnostic::TrailingBytes {
            at: self.off,
            len: body.len() - self.off,
        })
    }
}

/// Every section header from `reading` on, until one says the reading is over.
fn read_sections(body: &[u8], reading: Reading) -> Reading {
    let step = read_section(body, reading.off, reading.expected);
    match reading.take(step) {
        (reading, true) => read_sections(body, reading),
        (reading, false) => reading,
    }
}

/// The signature check: too short to hold a signature and a checksum, or the wrong signature.
fn check_magic(bytes: &[u8]) -> Result<(), OpenError> {
    if bytes.len() < 20 {
        return Err(OpenError::TooShort);
    }
    if &bytes[0..14] != MAGIC.as_slice() {
        let mut found = [0u8; 16];
        found.copy_from_slice(&bytes[0..16]);
        return Err(OpenError::BadMagic { found });
    }
    Ok(())
}

impl Save {
    /// Reads and interprets a file. Read-only.
    pub fn open(path: impl AsRef<Path>) -> Result<Save, OpenError> {
        let bytes = std::fs::read(path)?;
        Save::parse(&bytes)
    }

    /// Interprets a buffer already in memory. Errors only on magic/length;
    /// every other inconsistency becomes a `Diagnostic`.
    pub fn parse(bytes: &[u8]) -> Result<Save, OpenError> {
        check_magic(bytes)?;
        let unknown_0x10 = read_u32(bytes, 0x10);
        // The last four bytes are the checksum, never a section.
        let body = &bytes[..bytes.len() - 4];
        let reading = read_sections(body, Reading::start());
        let trailing = reading.trailing(body);
        Ok(Save {
            unknown_0x10,
            sections: reading.sections,
            diagnostics: reading.diagnostics.into_iter().chain(trailing).collect(),
        })
    }
}

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

    /// Raw bytes of the bestiary, up to `end-4`. Kept alongside the typed reading
    /// because a section whose layout is only partly understood is worth being able to
    /// look at whole.
    pub fn bestiary(&self) -> Option<&[u8]> {
        self.section(crate::section::Kind::Bestiary)
            .map(|s| s.bytes.as_slice())
    }

    /// The bestiary read as the tallies it declares. `None` when the section is absent
    /// or too short to carry a header; a section that is present but malformed comes
    /// back with what could be read and the rest kept, as values, in `Bestiary::trailing`.
    pub fn bestiary_tallies(&self) -> Option<crate::bestiary::Bestiary> {
        self.bestiary().and_then(crate::bestiary::read)
    }
}

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
