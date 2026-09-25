//! The profile nobody ever played — the state the app opens in at a stranger's house.
//!
//! Every other sample in `samples/` is a profile with progress on it, because every one of them
//! came from somebody playing. This is the other end: a save slot the game created and nobody
//! touched (B61). The file is byte-identical across all fourteen 2024 backups, which is what says
//! it was never played rather than merely played little.

use core_save::{Kind, Save};

fn parse(bytes: &[u8]) -> Save {
    Save::parse(bytes).expect("a real sample must parse")
}

/// The played profile of the same day, for the comparisons that need two.
fn played_of_the_same_day() -> Option<Save> {
    test_support::sample_bytes("20240118.rep_persistentgamedata1.dat").map(|b| parse(&b))
}

/// **Nine sections of ten hold nothing**, and the tenth is the bestiary.
///
/// The first expectation written here was "no byte is set anywhere", derived from `od`: of the
/// 3956 bytes, 16 are the signature and 4 the trailing checksum, and the rest looked like section
/// headers. It was wrong, and the test said so — **the bestiary carries 7 non-zero bytes**. What
/// the whole-file count could not see is that the bestiary's payload is not all header.
///
/// So the assertion is the one that is true and says where the exception is, rather than a
/// weakened one that would stop looking: every section a player fills is empty, and the bestiary
/// is excluded by name and measured separately below.
#[test]
fn every_section_a_player_fills_is_empty_in_an_untouched_profile() {
    for path in test_support::empty_profile_samples() {
        let bytes = std::fs::read(&path).expect("a sample that is present must read");
        let save = parse(&bytes);
        for section in save.sections.iter().filter(|s| s.kind != Kind::Bestiary) {
            let set = section.bytes.iter().filter(|b| **b != 0).count();
            assert_eq!(
                set,
                0,
                "{:?} of {} has {set} bytes set, and nobody played it",
                section.kind,
                path.display()
            );
        }
    }
}

/// An untouched profile is **empty, not short**: it declares the same entry counts as a played one
/// of the same era, so the app reads a full-length section of zeros and never a missing one.
///
/// This is the half a "nothing is set" assertion cannot see, and the one that would break first:
/// a truncated file, or a parser that stopped early, satisfies "no byte set" perfectly. Both sides
/// read their counts from the file, never hardcoded.
#[test]
fn an_untouched_profile_declares_the_same_counts_as_a_played_one_of_its_era() {
    let Some(played) = played_of_the_same_day() else {
        return;
    };
    let counts =
        |s: &Save| -> Vec<(Kind, u32)> { s.sections.iter().map(|x| (x.kind, x.count)).collect() };
    for path in test_support::empty_profile_samples() {
        let bytes = std::fs::read(&path).expect("a sample that is present must read");
        assert_eq!(
            counts(&parse(&bytes)),
            counts(&played),
            "{} declares different sections from the played profile of the same day",
            path.display()
        );
    }
}

/// **The bestiary is the only section whose length moves**, and on an untouched profile it is at
/// its shortest. That is what "nothing was seen" looks like in this format: `count` stays 80 on
/// both sides — so the bestiary's `count` is not the number of entities recorded — while the
/// payload goes from 128 bytes here to 7032 in the played profile of the same day.
///
/// Stated as a relation and not as two pinned numbers: the numbers are a fixture of one era and
/// one profile, the relation holds of any pair.
#[test]
fn the_bestiary_is_the_only_section_that_grows_with_play() {
    let Some(played) = played_of_the_same_day() else {
        return;
    };
    for path in test_support::empty_profile_samples() {
        let bytes = std::fs::read(&path).expect("a sample that is present must read");
        let empty = parse(&bytes);
        for (mine, theirs) in empty.sections.iter().zip(played.sections.iter()) {
            assert_eq!(mine.kind, theirs.kind, "the sections are in file order");
            if mine.kind == Kind::Bestiary {
                assert!(
                    mine.bytes.len() < theirs.bytes.len(),
                    "{}: an untouched bestiary is not shorter than a played one ({} vs {})",
                    path.display(),
                    mine.bytes.len(),
                    theirs.bytes.len()
                );
            } else {
                assert_eq!(
                    mine.bytes.len(),
                    theirs.bytes.len(),
                    "{:?} changed length between two profiles of the same era",
                    mine.kind
                );
            }
        }
    }
}

/// What the untouched bestiary actually holds, and it is the reason this sample is worth more than
/// a fixture: **everything that is not structure is zero**, so the structure is all that is left
/// to look at.
///
/// `docs/save-format.md` records, under the unexplained gap between the game's eleven chunks and
/// our ten sections, that *"the bestiary payload's constant `words[20]` is 11"*. On a profile
/// nobody played that word is the **first** non-zero one in the whole section, with twenty zeros
/// before it and eleven more words after it — 48 bytes of structure with no entity data anywhere
/// near it to read past. Nothing here names what those words are; the test pins that they are
/// where they are, so that a future decoding starts from a measurement (B9).
#[test]
fn the_untouched_bestiary_is_structure_and_nothing_else() {
    for path in test_support::empty_profile_samples() {
        let bytes = std::fs::read(&path).expect("a sample that is present must read");
        let save = parse(&bytes);
        let Some(bestiary) = save.sections.iter().find(|s| s.kind == Kind::Bestiary) else {
            panic!("{} has no bestiary section", path.display());
        };
        let words: Vec<u32> = bestiary
            .bytes
            .chunks_exact(4)
            .map(|w| u32::from_le_bytes([w[0], w[1], w[2], w[3]]))
            .collect();
        assert!(
            words.len() > 20,
            "{}: the bestiary is {} words, too short to hold the one the document names",
            path.display(),
            words.len()
        );
        assert!(
            words[..20].iter().all(|w| *w == 0),
            "{}: something is recorded before the constant: {:?}",
            path.display(),
            &words[..20]
        );
        assert_eq!(
            words[20],
            11,
            "{}: the constant the document names is not where it is named",
            path.display()
        );
    }
}

/// **Every cell of the completion matrix is unset**, all 34 rows by 12 columns of it, and the
/// played profile of the same day is the proof the instrument can say otherwise.
///
/// This is the shape the Completion screen opens on at a stranger's house, and until now nothing
/// asserted what it produces — only that a played profile produces the right thing. A test that
/// checked the untouched profile alone would be the vacuous kind: "no cell is set" holds trivially
/// of anything this test failed to read, so the same walk has to light up on the played profile
/// before its silence here means anything.
#[test]
fn not_one_cell_of_the_matrix_is_set_in_an_untouched_profile() {
    use core_save::{cell_index, Column, ROWS};

    let cells = |save: &Save| -> Vec<u32> {
        let counters = save.u32s(Kind::Counters).unwrap_or_default();
        (0..ROWS)
            .flat_map(|row| Column::ALL.iter().map(move |c| (row, *c)))
            .filter_map(|(row, column)| cell_index(row, column))
            .filter_map(|i| counters.get(i).copied())
            .collect()
    };

    let Some(played) = played_of_the_same_day() else {
        return;
    };
    let lit = cells(&played);
    assert!(
        lit.iter().any(|c| *c != 0),
        "the played profile of the same day lights no cell either: this walk reads nothing, \
         and the silence below would mean nothing"
    );

    for path in test_support::empty_profile_samples() {
        let bytes = std::fs::read(&path).expect("a sample that is present must read");
        let empty = parse(&bytes);
        let mine = cells(&empty);
        assert_eq!(
            mine.len(),
            lit.len(),
            "{}: the two profiles do not even have the same matrix",
            path.display()
        );
        assert!(
            mine.iter().all(|c| *c == 0),
            "{}: {} cells are set on a profile nobody played",
            path.display(),
            mine.iter().filter(|c| **c != 0).count()
        );
    }
}
