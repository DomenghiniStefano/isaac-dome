//! The three questions `graph::Profile` asks, answered from the save.
//!
//! The numbers are the tests' own, so none of this depends on a sample.

use graph::rules::{CounterName, MarkColumn, MarkLevel};
use graph::Profile;
use ipc::SaveProgress;

/// Bits 0 and 1 are the mark's two levels. Bit 2 is **not** a level — it is "won online",
/// measured 2026-09-12 — so it is masked away before a level is read, or an online clear
/// would read as a second level nobody reached offline.
#[test]
fn a_cell_reports_the_highest_level_its_bits_show() {
    let mut counters = vec![0u32; 600];
    counters[142] = 3; // Greed, Keeper: base + second
    counters[131] = 1; // Greed, Magdalene: base only
    counters[133] = 5; // Greed, Judas: base + bit 2, which is not a level
    counters[135] = 0; // Greed, Eve: read, and nothing reached
    let p = SaveProgress::new(None, Some(&counters), None);
    assert_eq!(
        p.level_at(12, MarkColumn::Greed),
        Some(Some(MarkLevel::Second))
    );
    assert_eq!(
        p.level_at(1, MarkColumn::Greed),
        Some(Some(MarkLevel::Base))
    );
    assert_eq!(
        p.level_at(3, MarkColumn::Greed),
        Some(Some(MarkLevel::Base)),
        "bit 2 is 'won online', not a level: 5 is the base level, not the second"
    );
    assert_eq!(
        p.level_at(5, MarkColumn::Greed),
        Some(None),
        "read, and nothing reached: that is not the same as 'I can't say'"
    );
}

/// One of the 40 cells nobody has located. The layout says `None`, and so does the profile.
#[test]
fn an_unlocated_cell_cannot_be_answered() {
    let counters = vec![0u32; 600];
    let p = SaveProgress::new(None, Some(&counters), None);
    assert_eq!(p.level_at(15, MarkColumn::Mother), None, "Bethany × Mother");
}

#[test]
fn a_value_that_is_not_a_mask_is_not_read_as_one() {
    let mut counters = vec![0u32; 600];
    counters[142] = 4096;
    let p = SaveProgress::new(None, Some(&counters), None);
    assert_eq!(
        p.level_at(12, MarkColumn::Greed),
        None,
        "outside 0..=7 the index points somewhere else, and a guess would invent a mark"
    );
}

#[test]
fn a_short_or_absent_section_two_answers_none() {
    let p = SaveProgress::new(None, None, None);
    assert_eq!(p.counter(CounterName::HushKills), None);
    assert_eq!(p.level_at(12, MarkColumn::Greed), None);

    let short = SaveProgress::new(None, Some(&[0u32; 10]), None);
    assert_eq!(
        short.counter(CounterName::MotherKills),
        None,
        "index 491 is past the end of what was read"
    );
}

#[test]
fn a_tally_reads_its_value() {
    let mut counters = vec![0u32; 600];
    counters[158] = 17;
    counters[491] = 0;
    let p = SaveProgress::new(None, Some(&counters), None);
    assert_eq!(p.counter(CounterName::HushKills), Some(17));
    assert_eq!(
        p.counter(CounterName::MotherKills),
        Some(0),
        "read and zero is an answer; only an unread section is not"
    );
}

/// Without a catalog there is no way from a `CharacterId` to a row, so the mark question
/// cannot be answered at all — and says so rather than answering about row 0.
#[test]
fn without_a_catalog_no_character_maps_to_a_row() {
    let counters = vec![3u32; 600];
    let p = SaveProgress::new(None, Some(&counters), None);
    assert_eq!(p.mark(catalog::CharacterId(0), MarkColumn::Greed), None);
}
