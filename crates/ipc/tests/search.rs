//! Search: one index over the wiki's text and the catalog's names. The tests state what a
//! page reads as, which field a query hits, and how the hits are ordered — never what the
//! current code happens to answer.

use catalog::Catalog;
use ipc::{documents_for_tests, progress_for_tests, ProgressMark, SaveFlags, SearchIndex, Target};
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

const ITEMS: &[u8] = b"<items gfxroot=\"gfx/items/\"><passive id=\"105\" gfx=\"d6.png\" name=\"The D6\" achievement=\"1\" /><trinket id=\"97\" gfx=\"t.png\" name=\"Tonsil\" /></items>";
const ACH: &[u8] = b"<achievements gfxroot=\"gfx/ui/achievement/\"><!-- Defeat Mom's Heart 10 times --><achievement id=\"1\" text=\"You unlocked The D6\" gfx=\"1.png\" /></achievements>";
const BOSSES: &[u8] = b"<bossportraits gfxroot=\"gfx/ui/boss/\"><boss id=\"1\" name=\"Monstro\" portrait=\"Portrait_20.0_Monstro.png\" /></bossportraits>";

fn catalog() -> Catalog {
    Catalog::build(|p| match p {
        "items.xml" => Some(ITEMS.to_vec()),
        "achievements.xml" => Some(ACH.to_vec()),
        "bossportraits.xml" => Some(BOSSES.to_vec()),
        _ => None,
    })
}

#[test]
fn a_target_both_sides_know_is_one_document_with_the_catalog_name_as_its_title() {
    let ds = dataset();
    let index = SearchIndex::build(Ok(&ds));
    let docs = documents_for_tests(&index, Some(&catalog()));
    let d6 = docs
        .get(&Target::Item { id: 105 })
        .expect("the item is on both sides");
    assert_eq!(d6.title, "The D6");
    // The wiki title equals the catalog name here, so there is no second name to carry.
    assert_eq!(d6.alias, None);
    assert!(d6.has_page);
    // The achievement is in the catalog only: still a document, with no page to open.
    let a = docs
        .get(&Target::Achievement { id: 1 })
        .expect("the achievement is in the catalog");
    assert_eq!(a.title, "You unlocked The D6");
    assert_eq!(a.condition.as_deref(), Some("Defeat Mom's Heart 10 times"));
    assert!(!a.has_page);
    // The boss's entity key comes from the portrait's file name, the way the icon does.
    assert!(docs.contains_key(&Target::Entity {
        id: 20,
        variant: 0,
        subtype: 0
    }));
}

#[test]
fn without_a_catalog_the_documents_are_the_wiki_pages_alone() {
    let ds = dataset();
    let index = SearchIndex::build(Ok(&ds));
    let docs = documents_for_tests(&index, None);
    assert_eq!(docs.len(), index.len());
    assert_eq!(
        docs.get(&Target::Trinket { id: 97 })
            .map(|d| d.title.as_str()),
        Some("Tonsil")
    );
}

#[test]
fn a_mark_is_read_from_the_section_that_holds_it() {
    // Achievement 1 done, achievement 2 not; no collectible slot is set.
    let done = [false, true, false];
    let owned = [false; 106];
    let flags = SaveFlags {
        achievements: Some(&done),
        items: Some(&owned),
    };
    assert_eq!(
        progress_for_tests(&Target::Achievement { id: 1 }, Some(flags)),
        ProgressMark::Done
    );
    assert_eq!(
        progress_for_tests(&Target::Achievement { id: 2 }, Some(flags)),
        ProgressMark::Pending
    );
    assert_eq!(
        progress_for_tests(&Target::Item { id: 105 }, Some(flags)),
        ProgressMark::Pending
    );
    // A trinket has no slot in section 4, and a boss none anywhere: no mark, not "unknown".
    assert_eq!(
        progress_for_tests(&Target::Trinket { id: 97 }, Some(flags)),
        ProgressMark::None
    );
    // No profile, and a profile whose section didn't read, are both "unknown".
    assert_eq!(
        progress_for_tests(&Target::Achievement { id: 1 }, None),
        ProgressMark::Unknown
    );
    let unread = SaveFlags {
        achievements: None,
        items: None,
    };
    assert_eq!(
        progress_for_tests(&Target::Item { id: 105 }, Some(unread)),
        ProgressMark::Unknown
    );
}
