//! The document: one preset, always saved, and the current draw beside it.
//!
//! `deck_size` is stored because it is the only part of a draw that cannot be recomputed
//! later — the deck at the moment of the draw is gone once the save moves.

use roll::{Document, DocumentError, Drawn, Preset, Selection, Target, DOCUMENT_VERSION};

#[test]
fn the_default_document_has_the_default_preset_and_nothing_drawn() {
    let d = Document::default();
    assert_eq!(d.version, DOCUMENT_VERSION);
    assert_eq!(d.preset, Preset::default());
    assert_eq!(d.current, None);
}

#[test]
fn a_document_survives_a_round_trip() {
    let doc = Document {
        version: DOCUMENT_VERSION,
        preset: Preset {
            characters: Selection::Only {
                ids: vec![0, 4, 17],
            },
            columns: Selection::Only { ids: vec![7] },
            include_taken: true,
            only_playable: false,
        },
        current: Some(Drawn {
            target: Target::Greedier { character: 4 },
            deck_size: 137,
            drawn_unix: 1_789_000_000,
        }),
    };
    assert_eq!(Document::from_json(&doc.to_json()), Ok(doc));
}

#[test]
fn a_mark_survives_a_round_trip_too() {
    let doc = Document {
        current: Some(Drawn {
            target: Target::Mark {
                character: 33,
                column: 11,
            },
            deck_size: 1,
            drawn_unix: 0,
        }),
        ..Document::default()
    };
    assert_eq!(Document::from_json(&doc.to_json()), Ok(doc));
}

#[test]
fn the_wire_names_are_the_ones_the_document_promises() {
    // The document is a format: renaming a field is a migration, so the names are pinned here
    // rather than left to whatever serde happens to do.
    let doc = Document {
        current: Some(Drawn {
            target: Target::Mark {
                character: 1,
                column: 2,
            },
            deck_size: 9,
            drawn_unix: 5,
        }),
        ..Document::default()
    };
    let json = doc.to_json();
    assert!(json.contains("\"includeTaken\""), "{json}");
    assert!(json.contains("\"onlyPlayable\""), "{json}");
    assert!(json.contains("\"deckSize\""), "{json}");
    assert!(json.contains("\"drawnUnix\""), "{json}");
    assert!(json.contains("\"kind\":\"mark\""), "{json}");
}

#[test]
fn a_document_that_does_not_parse_is_reported_and_not_replaced() {
    let e = Document::from_json("{ this is not json").expect_err("garbage is not a document");
    assert!(matches!(e, DocumentError::Unreadable { .. }));
}

#[test]
fn a_document_from_the_future_says_so_rather_than_being_read_anyway() {
    // A document written by a version that knew more is not garbage to be silently replaced:
    // it is a different sentence on the screen, and the caller has to tell the two apart.
    let json = format!(
        "{{\"version\":{},\"preset\":{},\"current\":null}}",
        DOCUMENT_VERSION + 1,
        serde_json::to_string(&Preset::default()).expect("a preset serializes")
    );
    assert_eq!(
        Document::from_json(&json),
        Err(DocumentError::FromTheFuture {
            version: DOCUMENT_VERSION + 1,
            supported: DOCUMENT_VERSION,
        })
    );
}

#[test]
fn a_document_from_an_older_version_is_still_read() {
    // Version 0 never shipped, but the rule it exercises is the one that matters: older is
    // readable, newer is not.
    let json = format!(
        "{{\"version\":0,\"preset\":{},\"current\":null}}",
        serde_json::to_string(&Preset::default()).expect("a preset serializes")
    );
    let doc = Document::from_json(&json).expect("older documents are readable");
    assert_eq!(doc.preset, Preset::default());
}
