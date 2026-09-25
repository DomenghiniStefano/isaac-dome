use std::path::Path;

use serde::Serialize;

use crate::section::Section;

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

impl Save {
    /// Reads and interprets a file. Read-only.
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

        let end = bytes.len() - 4;
        let mut off = 0x14usize;
        let mut sections = Vec::new();
        let mut diagnostics = Vec::new();
        let mut expected = 1u32;

        while off + 12 <= end {
            let found_kind = read_u32(bytes, off);
            let f2 = read_u32(bytes, off + 4);
            let count = read_u32(bytes, off + 8);

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

            // The bestiary (the only variable-length section) doesn't rely on the
            // sequence number to be sized, and by format it's always the last
            // section; we exempt it from the sequence check so it doesn't raise a
            // spurious UnexpectedKind when it shows up on its own (e.g. in fixtures).
            if found_kind != expected && kind.bytes_per_entry().is_some() {
                diagnostics.push(Diagnostic::UnexpectedKind {
                    at: off,
                    expected,
                    found: found_kind,
                });
            }

            let data = off + 12;
            let available = end - data;
            let wanted = match kind.bytes_per_entry() {
                None => available, // bestiary: up to end-4
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
            // The next is expected after the one that was found, not after the one that was
            // expected: one missing section is one diagnostic, not one per section after it
            // (card #80, P11a).
            expected = found_kind.saturating_add(1);
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
    /// back with what could be read and the rest counted in `unread_words`.
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
