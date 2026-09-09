//! The section names as the frontend reads them.
//!
//! `core_save::Kind` crosses the IPC inside [`SectionCount`], so a variant's *name* is a
//! wire value and renaming one changes the JSON. Nothing pinned that until B9 renamed two
//! variants and the whole workspace stayed green — a rename that reaches the frontend has
//! to be a decision, not a side effect of tidying an enum.

use core_save::Kind;
use ipc::SectionCount;

/// Every variant against the string it serializes to, spelled out rather than derived
/// from `Kind`: a table generated from the enum would rename along with it and assert
/// nothing. The strings are serde's `snake_case` of each variant — `Unknown5` has no
/// boundary before the digit, so it stays `unknown5`.
const ON_THE_WIRE: [(Kind, &str); 10] = [
    (Kind::Achievements, "achievements"),
    (Kind::Counters, "counters"),
    (Kind::LevelCounters, "level_counters"),
    (Kind::Items, "items"),
    (Kind::Unknown5, "unknown5"),
    (Kind::Bosses, "bosses"),
    (Kind::Challenges, "challenges"),
    (Kind::Unknown8, "unknown8"),
    (Kind::Unknown9, "unknown9"),
    (Kind::Bestiary, "bestiary"),
];

#[test]
fn every_section_kind_has_a_pinned_wire_name() {
    ON_THE_WIRE.iter().for_each(|&(kind, expected)| {
        let json = serde_json::to_value(SectionCount { kind, count: 1 })
            .expect("a SectionCount serializes");
        assert_eq!(json["kind"], expected, "{kind:?}");
    });
}

/// The enum is closed and the table above is exhaustive by construction — but only as
/// long as someone keeps it that way. Adding an eleventh variant has to break this.
#[test]
fn the_table_covers_every_variant() {
    let covered: Vec<u32> = ON_THE_WIRE.iter().map(|&(k, _)| k.number()).collect();
    let all: Vec<u32> = (1..=10).collect();
    assert_eq!(covered, all, "one section is missing from ON_THE_WIRE");
}
