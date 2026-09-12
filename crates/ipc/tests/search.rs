//! Search: one index over the wiki's text and the catalog's names. The tests state what a
//! page reads as, which field a query hits, and how the hits are ordered — never what the
//! current code happens to answer.

use ipc::{SearchIndex, Target};
use wiki::{
    Block, Dataset, DatasetError, Dlc, Entry, Infobox, Inline, ListItem, Section, SectionKind,
    Style,
};

fn text(s: &str) -> Inline {
    Inline::Text {
        text: s.into(),
        style: Style::Plain,
    }
}

fn entry_with(title: &str, infobox: Infobox, sections: Vec<Section>) -> Entry {
    Entry {
        title: title.into(),
        revid: 1,
        infobox,
        sections,
    }
}

/// The D6's Effects section as the dataset holds it: a heading, a paragraph with a `ref` and
/// an `edition` inline, a list with a nested block, and a one-row table. The flattening is
/// what a query is matched against, so the test states the string it becomes.
fn d6_effects() -> Section {
    Section {
        kind: SectionKind::Effects,
        blocks: vec![
            Block::Heading {
                level: 2,
                inline: vec![text("Effects")],
            },
            Block::Paragraph {
                inline: vec![
                    text("Rerolls the items in the"),
                    Inline::Ref {
                        target: Target::Room {
                            name: "Treasure Room".into(),
                        },
                        label: "Treasure Room".into(),
                    },
                    Inline::Edition {
                        only: vec![Dlc::Repentance],
                        inline: vec![text("only in Repentance")],
                    },
                ],
            },
            Block::List {
                ordered: false,
                items: vec![ListItem {
                    inline: vec![text("Damage up")],
                    children: vec![Block::Paragraph {
                        inline: vec![text("stacks")],
                    }],
                }],
            },
            Block::Table {
                header: vec![vec![text("Quality")]],
                rows: vec![vec![vec![text("4")]]],
            },
        ],
    }
}

fn dataset() -> Dataset {
    let mut ds = Dataset::empty_for_tests();
    ds.items
        .insert(105, entry_with("The D6", Infobox::Item, vec![d6_effects()]));
    ds.trinkets
        .insert(97, entry_with("Tonsil", Infobox::Trinket, vec![]));
    ds.bosses.insert(
        Dataset::boss_key(20, 0, 0),
        entry_with(
            "Monstro",
            Infobox::Boss {
                base_hp: None,
                environment: vec![],
                pool: vec![],
                unlocked_by: None,
            },
            vec![Section {
                kind: SectionKind::Behavior,
                blocks: vec![Block::Paragraph {
                    inline: vec![text("Jumps at the player and spits blood")],
                }],
            }],
        ),
    );
    ds
}

#[test]
fn a_section_reads_as_its_text_labels_and_cells_in_order() {
    let ds = dataset();
    let index = SearchIndex::build(Ok(&ds));
    assert!(index.is_loaded());
    assert_eq!(index.len(), 3);
    assert_eq!(index.title(&Target::Item { id: 105 }), Some("The D6"));
    // Headings, ref labels, nested list blocks and table cells are all text to search; an
    // `edition` inline is unwrapped, because the page shows its words.
    assert_eq!(
        index.section_text(&Target::Item { id: 105 }, SectionKind::Effects),
        Some(
            "Effects Rerolls the items in the Treasure Room only in Repentance Damage up stacks Quality 4"
        )
    );
    assert_eq!(
        index.section_text(&Target::Item { id: 105 }, SectionKind::Notes),
        None
    );
    assert_eq!(
        index.section_text(
            &Target::Entity {
                id: 20,
                variant: 0,
                subtype: 0
            },
            SectionKind::Behavior
        ),
        Some("Jumps at the player and spits blood")
    );
}

#[test]
fn a_dataset_that_did_not_load_is_an_empty_index_that_says_so() {
    let e = DatasetError::Malformed { reason: "x".into() };
    let index = SearchIndex::build(Err(&e));
    assert!(!index.is_loaded());
    assert!(index.is_empty());
    assert_eq!(index.title(&Target::Item { id: 105 }), None);
}
