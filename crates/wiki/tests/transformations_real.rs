//! Properties of the sixteen transformations over the real snapshot.
//!
//! No sample and no game: both the embedded dataset and `dataset/raw/` are committed, so
//! these never skip. What they assert is the *shape* of the answer — how many, of what kind,
//! and that both outcomes of every either/or actually occur. The numbers themselves belong
//! to the wiki and are read from it, never pinned.

use std::path::PathBuf;

use wiki::{Dataset, Infobox, Raw, Target};

fn dataset() -> &'static Dataset {
    Dataset::embedded().expect("the embedded dataset loads")
}

fn transformation(ds: &Dataset, id: u32) -> (&str, Option<u32>, &Vec<Target>) {
    let e = ds
        .entry(&Target::Transformation { id })
        .unwrap_or_else(|| panic!("no entry for transformation {id}"));
    let Infobox::Transformation {
        requires,
        contributors,
        ..
    } = &e.infobox
    else {
        panic!("{} does not carry a transformation infobox", e.title)
    };
    (&e.title, *requires, contributors)
}

/// As many entries as the Cargo table has rows — never the literal sixteen, which is what
/// the wiki has today and not a constant. Read from `dataset/raw/` the way the `derived`
/// test does, because the embedded dataset does not carry the table it was built from.
#[test]
fn every_row_of_the_table_became_an_entry() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../dataset/raw");
    let raw = Raw::load(&root).expect("dataset/raw/");
    let rows = raw.tables.transformation.len();
    assert!(
        rows > 0,
        "the transformation table is empty: nothing proved"
    );
    assert_eq!(dataset().transformations.len(), rows);
}

/// Guppy's set contains a trinket, and that is the non-vacuity guard for the whole union:
/// Kid's Drawing is listed in the page's body table and **not** in the infobox's `items`, so
/// a union that quietly stopped working would fail here and nowhere else.
#[test]
fn guppys_set_contains_the_trinket_its_infobox_omits() {
    let ds = dataset();
    let (title, _, contributors) = transformation(ds, 0);
    assert_eq!(title, "Guppy");
    assert!(
        contributors
            .iter()
            .any(|t| matches!(t, Target::Trinket { .. })),
        "{contributors:?}"
    );
}

/// The cross-check has to be able to speak: on the live snapshot at least one page states
/// its set twice and states it differently. A counter that has never moved has not been
/// shown able to move.
#[test]
fn the_source_disagreement_is_not_a_dead_counter() {
    assert!(dataset().meta.diagnostics.transformation_sources_disagree > 0);
}

/// A count read for most pages and honestly unread for some. Both halves are asserted: a run
/// where every count came back `None` — or where the pattern matched something it should
/// not have and every count came back `Some` — would otherwise look like a pass.
#[test]
fn some_counts_are_read_and_some_are_honestly_unread() {
    let ds = dataset();
    let (mut read, mut unread) = (0, 0);
    for (id, e) in &ds.transformations {
        let (title, requires, _) = transformation(ds, *id);
        match requires {
            Some(n) => {
                assert!(n > 0, "{title}: a threshold of zero is met by nothing");
                read += 1;
            }
            None => unread += 1,
        }
        assert!(!e.title.is_empty());
    }
    assert!(read > 0 && unread > 0, "read {read}, unread {unread}");
}

/// A page that states a count states a set too — except where it states neither. What must
/// never happen is a count with an empty set: that is a requirement nothing can satisfy, and
/// it would read on screen as "you are three items away" forever.
///
/// Stompy is why this is a property and not an assumption: its third contributor is a pill,
/// listed as a bullet rather than in a table, so its set is smaller than its count. `graph`
/// refuses to build a threshold in that case (spec §3.1); here the shape is merely recorded.
#[test]
fn a_count_never_comes_with_an_empty_set() {
    let ds = dataset();
    for id in ds.transformations.keys() {
        let (title, requires, contributors) = transformation(ds, *id);
        if requires.is_some() {
            assert!(
                !contributors.is_empty(),
                "{title} states a count and no set"
            );
        }
    }
}
