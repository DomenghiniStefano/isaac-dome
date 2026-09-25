//! Section 10, read from its own declarations rather than from a guess at its shape.
//!
//! The section is self-describing: a run of zero words, a three-word header, then as
//! many `(id, size)` blocks as the header declares, each holding a key → count list in
//! key order. The sizes sum to the header's total and the blocks consume the section to
//! the word, on every save collected so far.
//!
//! **What the blocks count is not known and is not named here.** They keep the id the
//! file gives them. Telling them apart needs a matched window against a live game, which
//! is recorded as open work; a name invented in the meantime would be the same mistake
//! that had section 6 reporting bosses as cards.
//!
//! No screen reads it: it is kept for the measuring examples (`bestiary_diff`,
//! `matched_window`), the instruments that will tell the blocks apart.

use serde::Serialize;

/// Zero words before the header in every save collected so far. The game reads a
/// "Special Seed Counters" chunk here before the bestiary, which is the likeliest thing
/// they belong to — likeliest, not measured, so nothing is claimed about them.
const PREAMBLE_WORDS: usize = 20;

/// Marker, declared total, declared number of tallies.
const HEADER_WORDS: usize = 3;

/// A tally's declared size is in units of two bytes. A record is a key and a count, so
/// eight bytes: four units, two words.
const UNITS_PER_RECORD: u32 = 4;
const WORDS_PER_RECORD: usize = 2;

/// An entity as the game identifies it — Type, Variant, SubType — packed into the key
/// the save writes: `(kind << 20) | (variant << 8) | subtype`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct EntityId {
    pub kind: u32,
    pub variant: u32,
    pub subtype: u32,
}

impl EntityId {
    pub fn from_packed(packed: u32) -> Self {
        EntityId {
            kind: packed >> 20,
            variant: (packed >> 8) & 0xFFF,
            subtype: packed & 0xFF,
        }
    }

    pub fn packed(self) -> u32 {
        (self.kind << 20) | ((self.variant & 0xFFF) << 8) | (self.subtype & 0xFF)
    }
}

/// One entity and the number this tally has against it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Record {
    pub entity: EntityId,
    pub count: u32,
}

/// One counted list. `id` is the number the file gives it, in file order 4, 2, 3, 1 on
/// every save collected so far — which is why it is carried and not inferred from the
/// position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Tally {
    pub id: u32,
    pub records: Vec<Record>,
}

/// The section, plus what it declared about itself. The two `declared_*` fields are kept
/// because they are what makes the reading checkable: a test can ask whether the blocks
/// actually add up to the total the file states, which is the invariant that identified
/// the layout.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Bestiary {
    pub tallies: Vec<Tally>,
    pub declared_tallies: u32,
    pub declared_units: u32,
    /// Words after the last tally that the layout does not account for. Exactly one on
    /// every save collected so far, and it grows with the profile — 11,343 in January
    /// 2024, 29,725 a year later — so it is carried as a **value** and not as a count.
    /// Whatever it turns out to be, the next session can look at it without re-deriving
    /// a single offset, which is how it nearly went unnoticed: the throwaway script that
    /// mapped this section had an off-by-one that consumed it, and only the reader's own
    /// arithmetic gave it back.
    pub trailing: Vec<u32>,
}

/// Reads the section. `None` only when there is not even a header to read: a section
/// that is present but malformed comes back with the tallies that could be read and the
/// remainder in `trailing`, because a truncated file has to degrade.
pub(crate) fn read(bytes: &[u8]) -> Option<Bestiary> {
    let words: Vec<u32> = bytes
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();

    let declared_units = *words.get(PREAMBLE_WORDS + 1)?;
    let declared_tallies = *words.get(PREAMBLE_WORDS + 2)?;

    let (tallies, end) = read_tallies(&words, PREAMBLE_WORDS + HEADER_WORDS, declared_tallies);

    Some(Bestiary {
        tallies,
        declared_tallies,
        declared_units,
        trailing: words.get(end..).unwrap_or_default().to_vec(),
    })
}

/// Reads at most `declared` tallies, stopping early at the first one that doesn't fit.
/// Stopping is not an error: it leaves the words behind it counted as unread.
fn read_tallies(words: &[u32], start: usize, declared: u32) -> (Vec<Tally>, usize) {
    let attempts = (declared as usize).min(words.len());
    (0..attempts).fold((Vec::new(), start), |(mut acc, at), _| {
        match read_tally(words, at) {
            None => (acc, at),
            Some((tally, next)) => {
                acc.push(tally);
                (acc, next)
            }
        }
    })
}

/// One `(id, size)` block and its records, and where the next one starts.
fn read_tally(words: &[u32], at: usize) -> Option<(Tally, usize)> {
    let id = *words.get(at)?;
    let units = *words.get(at + 1)?;
    let len = (units / UNITS_PER_RECORD) as usize;
    let body = words.get(at + 2..at + 2 + len * WORDS_PER_RECORD)?;

    let records = body
        .chunks_exact(WORDS_PER_RECORD)
        .map(|p| Record {
            entity: EntityId::from_packed(p[0]),
            count: p[1],
        })
        .collect();

    Some((Tally { id, records }, at + 2 + len * WORDS_PER_RECORD))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The packing is the only place this crate interprets a number rather than moving
    /// it, so it round-trips against itself for every field boundary.
    #[test]
    fn a_key_survives_being_unpacked_and_packed_again() {
        [
            0x00A0_0000u32,
            0x00A0_0100,
            0x02600100,
            0x3B70_2800,
            0xFFF0_0000,
            0x000F_FF00,
            0x0000_00FF,
            0,
        ]
        .iter()
        .for_each(|&packed| {
            assert_eq!(
                EntityId::from_packed(packed).packed(),
                packed,
                "{packed:#010X}"
            );
        });
    }

    #[test]
    fn the_fields_land_where_the_game_puts_them() {
        // 0x02600100: type 38, variant 1, subtype 0 — worked out by hand from the shifts,
        // and the same example the analysis in B9 used.
        let id = EntityId::from_packed(0x0260_0100);
        assert_eq!((id.kind, id.variant, id.subtype), (38, 1, 0));
    }

    fn section(preamble: usize, header: [u32; 3], body: &[u32]) -> Vec<u8> {
        std::iter::repeat_n(0u32, preamble)
            .chain(header)
            .chain(body.iter().copied())
            .flat_map(u32::to_le_bytes)
            .collect()
    }

    #[test]
    fn a_section_too_short_for_a_header_reads_as_nothing() {
        assert!(read(&[0u8; 4]).is_none());
        assert!(read(&[]).is_none());
    }

    /// A tally whose size runs past the end of the section doesn't panic and doesn't
    /// invent records: it is skipped, and its words show up as unread.
    #[test]
    fn a_tally_that_overruns_the_section_is_left_unread() {
        let bytes = section(PREAMBLE_WORDS, [11, 8, 1], &[7, 800]);
        let b = read(&bytes).expect("the header is there");
        assert_eq!(
            b.tallies,
            vec![],
            "a size of 800 units cannot fit in 2 words"
        );
        assert_eq!(b.declared_tallies, 1);
        assert_eq!(
            b.trailing,
            vec![7, 800],
            "the block that didn't fit stays visible"
        );
    }

    /// The declared count is what is read, not the number of blocks that happen to fit:
    /// a file claiming three when it holds one gives back the one, and says so.
    #[test]
    fn fewer_tallies_than_declared_degrade_to_what_is_there() {
        let bytes = section(PREAMBLE_WORDS, [11, 4, 3], &[7, 4, 0x00A0_0000, 2]);
        let b = read(&bytes).expect("the header is there");
        assert_eq!(b.declared_tallies, 3);
        assert_eq!(b.tallies.len(), 1);
        assert_eq!(b.tallies[0].id, 7);
        assert_eq!(b.tallies[0].records.len(), 1);
        assert_eq!(b.trailing, vec![]);
    }
}
