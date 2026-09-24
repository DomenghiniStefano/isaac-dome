//! The loader, and the committed file held to the report it was transcribed from.
//!
//! The parsing tests use a fixture written here, so they specify the loader and not the day's
//! content of the wiki; the last two hold the real file to
//! `docs/superpowers/reports/2026-09-15-secret-room-rules.md`.

// A test that extracts one variant panics on every other, the ones added later included: here
// the wildcard *is* the assertion, and it fails loudly on a new variant instead of hiding it.
#![allow(clippy::wildcard_enum_match_arm)]

use floor::{Constraint, Rules, Target};

const FIXTURE: &str = r#"{
  "version": 1,
  "license": "CC BY-SA 4.0",
  "read": "2026-09-15",
  "rules": [
    {
      "id": "secret-neighbours",
      "target": "secret",
      "quote": "Regular Secret Rooms are usually located next to 3 or 4 rooms",
      "url": "https://example.invalid/Secret_Room",
      "constraint": { "kind": "neighbourCount", "allowed": [3, 4], "rank": 0 }
    },
    {
      "id": "secret-neighbours-one",
      "target": "secret",
      "quote": "1 neighbor locations can only happen if there are no valid 3+ neighbor locations",
      "url": "https://example.invalid/Secret_Room",
      "constraint": { "kind": "neighbourCountFallback", "allowed": [1], "rank": 2, "supersededByAtLeast": 3 }
    },
    {
      "id": "secret-forbidden-neighbours",
      "target": "secret",
      "quote": "except Boss Rooms, Super Secret Rooms, and other Secret Rooms",
      "url": "https://example.invalid/Secret_Room",
      "constraint": { "kind": "forbiddenNeighbour", "kinds": ["boss", "superSecret", "secret"] }
    },
    {
      "id": "ultra-secret-connections",
      "target": "ultraSecret",
      "quote": "connect to 3+ non-red rooms through its adjacent red rooms",
      "url": "https://example.invalid/Ultra_Secret_Room",
      "constraint": { "kind": "redRoomConnections", "atLeast": 3, "atMost": null, "rank": 0 }
    },
    {
      "id": "ultra-secret-shapes",
      "target": "ultraSecret",
      "quote": "next to the sides of narrow rooms",
      "url": "https://example.invalid/Ultra_Secret_Room",
      "constraint": { "kind": "unmodelled", "note": "red rooms open on a side, and a side belongs to a shape this grid does not draw" }
    }
  ]
}"#;

#[test]
fn a_rule_carries_its_quotation_and_its_url() {
    let rules = Rules::parse(FIXTURE).expect("the fixture parses");
    let r = rules
        .for_target(Target::Secret)
        .find(|r| r.id == "secret-neighbours")
        .expect("the rule is there");
    assert_eq!(
        r.quote,
        "Regular Secret Rooms are usually located next to 3 or 4 rooms"
    );
    assert_eq!(r.url, "https://example.invalid/Secret_Room");
}

#[test]
fn for_target_answers_only_that_targets_rules() {
    let rules = Rules::parse(FIXTURE).expect("the fixture parses");
    let ids: Vec<&str> = rules
        .for_target(Target::Secret)
        .map(|r| r.id.as_str())
        .collect();
    assert_eq!(
        ids,
        vec![
            "secret-neighbours",
            "secret-neighbours-one",
            "secret-forbidden-neighbours"
        ]
    );
    assert_eq!(rules.for_target(Target::SuperSecret).count(), 0);
}

#[test]
fn a_constraint_the_grid_cannot_evaluate_parses_as_unmodelled_rather_than_being_dropped() {
    let rules = Rules::parse(FIXTURE).expect("the fixture parses");
    let r = rules
        .for_target(Target::UltraSecret)
        .find(|r| r.id == "ultra-secret-shapes")
        .expect("the rule is there");
    match &r.constraint {
        Constraint::Unmodelled { note } => assert!(note.contains("red rooms")),
        other => panic!("expected Unmodelled, got {other:?}"),
    }
}

#[test]
fn an_open_band_of_red_room_connections_carries_no_ceiling() {
    // "3+" is what the sentence says. A `null` ceiling is that plus sign; writing [3,4,…,12]
    // instead would put a number in the file the wiki never states, and silently drop the
    // thirteenth if the grid ever grew one.
    let rules = Rules::parse(FIXTURE).expect("the fixture parses");
    let r = rules
        .for_target(Target::UltraSecret)
        .find(|r| r.id == "ultra-secret-connections")
        .expect("the rule is there");
    match &r.constraint {
        Constraint::RedRoomConnections {
            at_least,
            at_most,
            rank,
        } => {
            assert_eq!(*at_least, 3);
            assert_eq!(*at_most, None);
            assert_eq!(*rank, 0);
        }
        other => panic!("expected RedRoomConnections, got {other:?}"),
    }
}

#[test]
fn a_fallback_count_carries_the_number_that_switches_it_off() {
    // "1 neighbor locations can only happen if there are no valid 3+ neighbor locations".
    // The 3 is a number the sentence states, which is why it is a field and not a band name.
    let rules = Rules::parse(FIXTURE).expect("the fixture parses");
    let r = rules
        .for_target(Target::Secret)
        .find(|r| r.id == "secret-neighbours-one")
        .expect("the rule is there");
    match &r.constraint {
        Constraint::NeighbourCountFallback {
            allowed,
            rank,
            superseded_by_at_least,
        } => {
            assert_eq!(allowed, &vec![1]);
            assert_eq!(*rank, 2);
            assert_eq!(*superseded_by_at_least, 3);
        }
        other => panic!("expected NeighbourCountFallback, got {other:?}"),
    }
}

#[test]
fn a_malformed_rules_file_is_an_error_and_not_an_empty_set_of_rules() {
    assert!(Rules::parse("{").is_err());
    assert!(Rules::parse(r#"{"version":1,"rules":[{"id":"x"}]}"#).is_err());
}

#[test]
fn the_embedded_file_parses_and_every_rule_in_it_is_sourced() {
    let rules = Rules::embedded().expect("the embedded rules parse");
    assert!(
        rules.all().count() >= 4,
        "the report found at least four constraints worth encoding"
    );
    for r in rules.all() {
        assert!(!r.quote.trim().is_empty(), "{} has no quotation", r.id);
        assert!(
            r.url
                .starts_with("https://bindingofisaacrebirth.wiki.gg/wiki/"),
            "{} does not cite the wiki the repo already attributes",
            r.id
        );
    }
}

#[test]
fn the_embedded_file_is_the_fourteen_rules_the_report_sourced() {
    // The list is §2 of docs/superpowers/reports/2026-09-15-secret-room-rules.md, in its order.
    // Two of them — secret-neighbours-one and super-secret-not-next-to-secret — were found by
    // the research pass and are not in the plan's draft: a rule dropped here would read on the
    // screen as "nothing in the way".
    //
    // The last five are B68's: the Ultra Secret paragraph used to be one `unmodelled` row, so
    // that target answered nothing on every grid there is. Only the sentence about room
    // shapes is still unmodelled, and it kept the target's `Unmodelled` seat.
    //
    // **Order is not decoration here.** `solve` walks the rules in this order and a narrowing
    // rule only ever removes, so a rule that narrows before anything has been proposed narrows
    // an empty list — every `ultraSecret` proposer has to come before the two that cut.
    let rules = Rules::embedded().expect("the embedded rules parse");
    let ids: Vec<&str> = rules.all().map(|r| r.id.as_str()).collect();
    assert_eq!(
        ids,
        vec![
            "secret-neighbours",
            "secret-neighbours-two",
            "secret-neighbours-one",
            "secret-forbidden-neighbours",
            "super-secret-dead-end",
            "super-secret-neighbour-not-special",
            "super-secret-not-next-to-secret",
            "super-secret-second-longest",
            "ultra-secret-connections",
            "ultra-secret-connections-two",
            "ultra-secret-connections-one",
            "ultra-secret-not-connected",
            "ultra-secret-red-room-invalid",
            "ultra-secret-shapes",
        ]
    );
}
