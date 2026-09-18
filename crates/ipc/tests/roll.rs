//! The Roll view built with no catalog at all — which is the path this machine runs.
//!
//! Without the game's archives there are no names, no art, and no way to tell which characters
//! are unlocked; none of that may stop the screen answering, because a target is an index
//! pair, not a sprite. Every assertion but the last is about that path; the last one builds a
//! synthetic catalog (no game needed — `Catalog::build` reads inline XML) to cover the other
//! half of the two-preset rule, which `catalog: None` can never exercise.

use catalog::Catalog;
use ipc::{roll_view, RollDiagnostic, RollInputs, RollView, StatusView};
use roll::{Document, Drawn, Preset, Selection, Target};

/// Counters long enough to cover the located cells, all zero: every readable cell missing.
fn zeroed_counters() -> Vec<u32> {
    vec![0u32; 700]
}

fn view(counters: Option<&[u32]>, doc: &Document) -> RollView {
    roll_view(
        RollInputs {
            counters,
            flags: None,
            catalog: None,
            document: Ok(doc),
            store_reason: None,
        },
        |_| None,
    )
}

fn view_of_error(err: &roll::DocumentError) -> RollView {
    let c = zeroed_counters();
    roll_view(
        RollInputs {
            counters: Some(&c),
            flags: None,
            catalog: None,
            document: Err(err),
            store_reason: None,
        },
        |_| None,
    )
}

/// Every target the matrix holds, read from the tables that measured it and never written
/// here as 442.
fn every_target() -> usize {
    ipc::CHARACTERS.len() * ipc::BOSSES.len() + ipc::CHARACTERS.len()
}

#[test]
fn without_a_catalog_the_view_still_answers_and_says_the_catalog_is_missing() {
    let c = zeroed_counters();
    let v = view(Some(&c), &Document::default());
    assert!(v.diagnostics.contains(&RollDiagnostic::NoCatalog));
    assert!(v.deck.size > 0, "a target is an index pair, not a sprite");
}

#[test]
fn without_a_catalog_every_row_still_has_an_id_and_a_count() {
    let c = zeroed_counters();
    let v = view(Some(&c), &Document::default());
    assert_eq!(v.characters.len(), ipc::CHARACTERS.len());
    assert_eq!(v.columns.len(), ipc::BOSSES.len());
    assert!(v
        .characters
        .iter()
        .enumerate()
        .all(|(i, r)| r.id as usize == i));
    assert!(v
        .columns
        .iter()
        .enumerate()
        .all(|(i, r)| r.id as usize == i));
}

#[test]
fn the_deck_and_the_four_exclusions_account_for_every_target_of_the_matrix() {
    let c = zeroed_counters();
    let d = view(Some(&c), &Document::default()).deck;
    assert_eq!(
        d.size + d.taken + d.unreadable + d.locked + d.filtered,
        every_target()
    );
}

#[test]
fn without_section_two_every_cell_is_unreadable_and_the_deck_is_empty() {
    let v = view(None, &Document::default());
    assert!(v.diagnostics.contains(&RollDiagnostic::NoCounterSection));
    assert!(v.diagnostics.contains(&RollDiagnostic::EmptyDeck));
    assert_eq!(v.deck.size, 0);
    assert_eq!(v.deck.taken, 0);
    assert_eq!(v.deck.unreadable, every_target());
}

#[test]
fn without_section_two_but_with_anything_allowed_all_of_them_are_drawable() {
    let doc = Document {
        preset: Preset {
            include_taken: true,
            ..Preset::default()
        },
        ..Document::default()
    };
    let v = view(None, &doc);
    assert_eq!(v.deck.size, every_target());
    assert!(!v.diagnostics.contains(&RollDiagnostic::EmptyDeck));
}

#[test]
fn only_playable_is_not_applied_without_the_flags_and_the_view_admits_it() {
    // A filter that silently keeps everything is worse than one that says it is off.
    let c = zeroed_counters();
    let v = view(Some(&c), &Document::default());
    assert!(v.diagnostics.contains(&RollDiagnostic::PlayabilityUnknown));
    assert_eq!(v.deck.locked, 0);
    assert!(
        v.preset.only_playable,
        "the document keeps what the user chose"
    );
}

#[test]
fn a_drawn_target_has_its_status_recomputed_from_the_save_and_not_from_the_document() {
    // The card closes itself: nothing is ticked off, and nothing can be ticked off wrongly.
    let mut c = zeroed_counters();
    c[ipc::counter_index(0, 0).expect("cell (0, 0) is located")] = 1;
    let doc = Document {
        current: Some(Drawn {
            target: Target::Mark {
                character: 0,
                column: 0,
            },
            deck_size: 442,
            drawn_unix: 1_789_000_000,
        }),
        ..Document::default()
    };
    let drawn = view(Some(&c), &doc)
        .drawn
        .expect("a drawn document produces a card");
    assert_eq!(drawn.status, StatusView::Taken);
    assert_eq!(
        drawn.deck_size, 442,
        "the deck at the time is stored, not recomputed"
    );
    assert_eq!(drawn.drawn_unix, 1_789_000_000);
}

#[test]
fn a_drawn_mark_names_its_column_and_a_drawn_greedier_says_it_is_one() {
    let c = zeroed_counters();
    let mark = Document {
        current: Some(Drawn {
            target: Target::Mark {
                character: 0,
                column: 1,
            },
            deck_size: 1,
            drawn_unix: 0,
        }),
        ..Document::default()
    };
    assert_eq!(
        view(Some(&c), &mark).drawn.expect("a card").target,
        ipc::DrawnTargetView::Mark {
            column: ipc::BOSSES[1].to_string()
        }
    );

    let greedier = Document {
        current: Some(Drawn {
            target: Target::Greedier { character: 0 },
            deck_size: 1,
            drawn_unix: 0,
        }),
        ..Document::default()
    };
    assert_eq!(
        view(Some(&c), &greedier).drawn.expect("a card").target,
        ipc::DrawnTargetView::Greedier {}
    );
}

#[test]
fn without_a_catalog_a_card_carries_a_row_name_and_no_art() {
    // The names in `CHARACTERS` are the project's own, measured with the layout: they are not
    // the game's translated strings, and they are all this path has.
    let c = zeroed_counters();
    let doc = Document {
        current: Some(Drawn {
            target: Target::Greedier { character: 3 },
            deck_size: 1,
            drawn_unix: 0,
        }),
        ..Document::default()
    };
    let drawn = view(Some(&c), &doc).drawn.expect("a card");
    assert_eq!(drawn.character, ipc::CHARACTERS[3].0);
    assert_eq!(drawn.head_url, None);
    assert_eq!(drawn.art_url, None);
}

#[test]
fn a_drawn_target_the_matrix_no_longer_holds_simply_stops_being_drawn() {
    // A patch that shrank the matrix leaves a stored draw pointing nowhere. It is not an error
    // and not a card with an invented state: there is no card.
    let c = zeroed_counters();
    let doc = Document {
        current: Some(Drawn {
            target: Target::Mark {
                character: 200,
                column: 0,
            },
            deck_size: 1,
            drawn_unix: 0,
        }),
        ..Document::default()
    };
    assert!(view(Some(&c), &doc).drawn.is_none());
}

#[test]
fn an_unreadable_document_opens_on_the_default_preset_and_says_so() {
    let v = view_of_error(&roll::DocumentError::Unreadable {
        reason: "expected value".to_string(),
    });
    assert!(v.diagnostics.contains(&RollDiagnostic::DocumentUnreadable));
    assert!(!v.preset.include_taken);
    assert!(v.preset.only_playable);
    assert_eq!(v.preset.characters, ipc::SelectionView::All);
}

#[test]
fn a_document_from_the_future_says_which_version_it_was() {
    let v = view_of_error(&roll::DocumentError::FromTheFuture {
        version: 9,
        supported: 1,
    });
    assert!(v
        .diagnostics
        .contains(&RollDiagnostic::DocumentFromTheFuture {
            version: 9,
            supported: 1
        }));
    assert!(!v.diagnostics.contains(&RollDiagnostic::DocumentUnreadable));
}

#[test]
fn a_selection_travels_as_a_tagged_value_and_the_rows_agree_with_it() {
    let c = zeroed_counters();
    let doc = Document {
        preset: Preset {
            characters: Selection::Only { ids: vec![0, 5] },
            ..Preset::default()
        },
        ..Document::default()
    };
    let v = view(Some(&c), &doc);
    assert_eq!(
        v.preset.characters,
        ipc::SelectionView::Only { ids: vec![0, 5] }
    );
    assert!(v.characters[0].selected);
    assert!(v.characters[5].selected);
    assert!(!v.characters[1].selected);
}

#[test]
fn an_unticked_row_still_reports_what_ticking_it_would_be_worth() {
    let c = zeroed_counters();
    let doc = Document {
        preset: Preset {
            characters: Selection::Only { ids: vec![0] },
            ..Preset::default()
        },
        ..Document::default()
    };
    let v = view(Some(&c), &doc);
    assert!(
        v.characters[1].targets > 0,
        "a zero here is the one number that cannot help anyone decide"
    );
    assert_eq!(v.characters[0].targets, v.characters[1].targets);
}

#[test]
fn the_json_shape_is_camel_case_all_the_way_into_the_struct_variants() {
    // `rename_all` on an enum renames the variants, not the fields inside them:
    // `rename_all_fields` is what keeps `deckSize` from going out as `deck_size`, and it fails
    // silently — TypeScript reads `undefined` with no error at all.
    let c = zeroed_counters();
    let doc = Document {
        current: Some(Drawn {
            target: Target::Mark {
                character: 0,
                column: 0,
            },
            deck_size: 7,
            drawn_unix: 3,
        }),
        ..Document::default()
    };
    let json = serde_json::to_string(&view(Some(&c), &doc)).expect("the view serializes");
    for key in [
        "deckSize",
        "drawnUnix",
        "includeTaken",
        "onlyPlayable",
        "headUrl",
        "artUrl",
    ] {
        assert!(
            json.contains(&format!("\"{key}\"")),
            "missing {key} in {json}"
        );
    }
    assert!(!json.contains('_'), "snake_case survived somewhere: {json}");
}

/// Row 0 (`ISAAC`) is unlocked by nothing and always playable; row 1 (`MAGDALENE`) is unlocked
/// by achievement 5, which `flags` marks as not yet earned. Neither portrait carries the `_b`
/// token, so both resolve to their non-Tainted form — `character_for` matches `CHARACTER_KEYS`
/// by key and Tainted flag together.
const PLAYERS: &[u8] = b"<players portraitroot=\"gfx/ui/stage/\"><player id=\"0\" name=\"#ISAAC_NAME\" portrait=\"isaac.png\" /><player id=\"1\" name=\"#MAGDALENE_NAME\" portrait=\"magdalene.png\" achievement=\"5\" /></players>";

fn catalog_with_two_characters() -> Catalog {
    Catalog::build(|p| match p {
        "players.xml" => Some(PLAYERS.to_vec()),
        _ => None,
    })
}

#[test]
fn with_a_catalog_the_characters_you_have_not_unlocked_are_locked_out() {
    // Achievement 5 is not earned: Magdalene (row 1) is not playable. Isaac (row 0) is
    // unlocked by nothing, so he stays playable regardless of `flags`. The other 32 rows are
    // not named by this catalog at all, so "unknown must never hide a row" says they stay
    // playable too — nothing but Magdalene's own targets may end up `locked`.
    let c = zeroed_counters();
    let catalog = catalog_with_two_characters();
    let flags = [true, true, true, true, true, false];
    let with_only_playable = view_with_catalog(&c, &catalog, &flags, &Document::default());

    // The other half of the two-preset rule: playability is known here, so the document's own
    // `only_playable: true` (the default) is applied, not forced off — this is what the count
    // below proves.
    assert!(
        !with_only_playable
            .diagnostics
            .contains(&RollDiagnostic::PlayabilityUnknown),
        "playability was derivable this time"
    );
    // Her twelve marks plus her one Greedier, read from the tables rather than written as 13.
    let targets_of_one_row = ipc::BOSSES.len() + 1;
    assert_eq!(
        with_only_playable.deck.locked, targets_of_one_row,
        "only Magdalene's own targets are locked — the unnamed rows and Isaac are not"
    );

    // The flag is what drives `locked`, not merely the catalog's presence: with the same
    // catalog and the same flags but `only_playable: false`, nothing is locked at all.
    let doc = Document {
        preset: Preset {
            only_playable: false,
            ..Preset::default()
        },
        ..Document::default()
    };
    let without_only_playable = view_with_catalog(&c, &catalog, &flags, &doc);
    assert_eq!(without_only_playable.deck.locked, 0);
}

fn view_with_catalog(
    counters: &[u32],
    catalog: &Catalog,
    flags: &[bool],
    doc: &Document,
) -> RollView {
    roll_view(
        RollInputs {
            counters: Some(counters),
            flags: Some(flags),
            catalog: Some(catalog),
            document: Ok(doc),
            store_reason: None,
        },
        |_| None,
    )
}
