//! Search: one index over the wiki's text and the catalog's names. The tests state what a
//! page reads as, which field a query hits, and how the hits are ordered — never what the
//! current code happens to answer.

use catalog::Catalog;
use ipc::for_tests;
use ipc::ProgressMark;
use ipc::{
    search, IconRef, SaveFlags, SearchDiagnostic, SearchIndex, SearchMatch, SearchView, Target,
};
use serde_json::{json, to_value};
use wiki::for_tests::{empty_boss, empty_item, empty_trinket};
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
        sections,
        ..wiki::for_tests::entry(title, infobox)
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
    let mut ds = wiki::for_tests::empty_dataset();
    ds.items
        .insert(105, entry_with("The D6", empty_item(), vec![d6_effects()]));
    ds.trinkets
        .insert(97, entry_with("Tonsil", empty_trinket(), vec![]));
    ds.bosses.insert(
        Dataset::boss_key(20, 0, 0),
        entry_with(
            "Monstro",
            empty_boss(),
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
    let docs = for_tests::documents(&index, Some(&catalog()));
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
    let docs = for_tests::documents(&index, None);
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
        for_tests::progress(&Target::Achievement { id: 1 }, Some(flags)),
        ProgressMark::Done
    );
    assert_eq!(
        for_tests::progress(&Target::Achievement { id: 2 }, Some(flags)),
        ProgressMark::Pending
    );
    assert_eq!(
        for_tests::progress(&Target::Item { id: 105 }, Some(flags)),
        ProgressMark::Pending
    );
    // A trinket has no slot in section 4, and a boss none anywhere: no mark, not "unknown".
    assert_eq!(
        for_tests::progress(&Target::Trinket { id: 97 }, Some(flags)),
        ProgressMark::None
    );
    // No profile, and a profile whose section didn't read, are both "unknown".
    assert_eq!(
        for_tests::progress(&Target::Achievement { id: 1 }, None),
        ProgressMark::Unknown
    );
    let unread = SaveFlags {
        achievements: None,
        items: None,
    };
    assert_eq!(
        for_tests::progress(&Target::Item { id: 105 }, Some(unread)),
        ProgressMark::Unknown
    );
}

fn link(r: &IconRef) -> Option<String> {
    Some(format!("isaac://{}", r.to_path()))
}

fn view(query: &str, limit: usize, with_catalog: bool, flags: Option<SaveFlags<'_>>) -> SearchView {
    let ds = dataset();
    let index = SearchIndex::build(Ok(&ds));
    let c = catalog();
    search(
        &index,
        with_catalog.then_some(&c),
        flags,
        query,
        limit,
        link,
    )
}

#[test]
fn the_shapes_are_pinned() {
    assert_eq!(
        to_value(SearchMatch::Title).unwrap(),
        json!({"kind":"title"})
    );
    assert_eq!(
        to_value(SearchMatch::Condition {
            text: "Defeat Mom".into()
        })
        .unwrap(),
        json!({"kind":"condition","text":"Defeat Mom"})
    );
    assert_eq!(
        to_value(SearchMatch::Section {
            section: SectionKind::Effects,
            before: "a ".into(),
            matched: "b".into(),
            after: " c".into()
        })
        .unwrap(),
        json!({"kind":"section","section":"effects","before":"a ","matched":"b","after":" c"})
    );
    // Fieldless enums are bare strings, not tagged objects: the repo has no other kind.
    assert_eq!(
        to_value(SearchDiagnostic::NoProfile).unwrap(),
        json!("noProfile")
    );
    assert_eq!(to_value(ProgressMark::Pending).unwrap(), json!("pending"));

    let v = to_value(view("d6", 10, true, None)).unwrap();
    assert_eq!(v["query"], json!("d6"));
    assert_eq!(v["hits"][0]["target"], json!({"kind":"item","id":105}));
    assert_eq!(v["hits"][0]["title"], json!("The D6"));
    assert_eq!(v["hits"][0]["hasPage"], json!(true));
    assert_eq!(v["hits"][0]["iconUrl"], json!("isaac://page/item/105"));
    assert_eq!(v["hits"][0]["match"], json!({"kind":"title"}));
    assert_eq!(v["hits"][0]["progress"], json!("unknown"));
}

#[test]
fn one_hit_per_target_named_by_the_first_field_that_matches() {
    // "d6" is in the item's title and in the achievement's text: two targets, two hits, and
    // neither is listed twice for matching in a section as well.
    let v = view("d6", 10, true, None);
    assert_eq!(v.hits.len(), 2);
    assert_eq!(v.total, 2);
    // The achievement matches by title too ("You unlocked The D6"), never by its condition.
    let achievement = v
        .hits
        .iter()
        .find(|h| h.target == Target::Achievement { id: 1 })
        .expect("the achievement is a hit");
    assert_eq!(achievement.matched, SearchMatch::Title);

    // A word only the condition has names the condition; one only a section has names the
    // section, with the page's own text around it.
    let v = view("heart", 10, true, None);
    assert!(matches!(
        v.hits.first().map(|h| &h.matched),
        Some(SearchMatch::Condition { .. })
    ));
    let v = view("spits", 10, true, None);
    let Some(SearchMatch::Section {
        section, matched, ..
    }) = v.hits.first().map(|h| h.matched.clone())
    else {
        panic!("a word only a section has must name that section")
    };
    assert_eq!(section, SectionKind::Behavior);
    assert_eq!(matched, "spits");
}

#[test]
fn every_word_of_the_query_must_be_in_one_field() {
    // "monstro" is a title and "spits" is in its Behavior section: no field holds both, so
    // the page is not a hit.
    assert_eq!(view("monstro spits", 10, true, None).hits.len(), 0);
    assert_eq!(view("jumps blood", 10, true, None).hits.len(), 1);
}

#[test]
fn the_six_tiers_order_the_answer() {
    // Four titles that all contain "the": the whole title, its start, the start of a word in
    // it, and — "Mother" — the query buried inside a word, which is the weakest of the four.
    let mut ds = wiki::for_tests::empty_dataset();
    for (id, title) in [(1, "Mother"), (2, "The"), (3, "The Bible"), (4, "Of the")] {
        ds.items.insert(id, entry_with(title, empty_item(), vec![]));
    }
    let index = SearchIndex::build(Ok(&ds));
    let v = search(&index, None, None, "the", 10, link);
    let titles: Vec<&str> = v.hits.iter().map(|h| h.title.as_str()).collect();
    assert_eq!(titles, vec!["The", "The Bible", "Of the", "Mother"]);
}

#[test]
fn not_done_comes_before_done_inside_a_tier() {
    let mut ds = wiki::for_tests::empty_dataset();
    ds.items
        .insert(1, entry_with("Bomb One", empty_item(), vec![]));
    ds.items
        .insert(2, entry_with("Bomb Two", empty_item(), vec![]));
    let index = SearchIndex::build(Ok(&ds));
    // Item 1 is in the collection, item 2 isn't: the one still to find is listed first.
    let owned = [false, true, false];
    let flags = SaveFlags {
        achievements: Some(&[]),
        items: Some(&owned),
    };
    let v = search(&index, None, Some(flags), "bomb", 10, link);
    let titles: Vec<&str> = v.hits.iter().map(|h| h.title.as_str()).collect();
    assert_eq!(titles, vec!["Bomb Two", "Bomb One"]);
}

#[test]
fn the_limit_cuts_the_hits_and_total_says_how_many_there_were() {
    let mut ds = wiki::for_tests::empty_dataset();
    for id in 1..=5 {
        ds.items
            .insert(id, entry_with(&format!("Bomb {id}"), empty_item(), vec![]));
    }
    let index = SearchIndex::build(Ok(&ds));
    let v = search(&index, None, None, "bomb", 2, link);
    assert_eq!(v.hits.len(), 2);
    assert_eq!(v.total, 5);
}

#[test]
fn the_five_diagnostics_say_what_is_missing() {
    // No profile: one diagnostic, not one per section.
    let v = view("d6", 10, true, None);
    assert_eq!(v.diagnostics, vec![SearchDiagnostic::NoProfile]);
    // A profile whose two sections didn't read says so, once each.
    let unread = SaveFlags {
        achievements: None,
        items: None,
    };
    let v = view("d6", 10, true, Some(unread));
    assert_eq!(
        v.diagnostics,
        vec![
            SearchDiagnostic::NoAchievementSection,
            SearchDiagnostic::NoCollectionSection
        ]
    );
    // No game: wiki titles only, no icons, and no condition to match.
    let v = view("d6", 10, false, None);
    assert_eq!(
        v.diagnostics,
        vec![SearchDiagnostic::NoCatalog, SearchDiagnostic::NoProfile]
    );
    assert!(v.hits.iter().all(|h| h.icon_url.is_none()));
    // No dataset: catalog names only, and nothing has a page.
    let e = DatasetError::Malformed { reason: "x".into() };
    let index = SearchIndex::build(Err(&e));
    let c = catalog();
    let v = search(&index, Some(&c), None, "d6", 10, link);
    assert!(v.diagnostics.contains(&SearchDiagnostic::NoWiki));
    assert!(v.hits.iter().all(|h| !h.has_page));
    assert!(!v.hits.is_empty(), "the catalog still answers by name");
}

#[test]
fn an_empty_query_answers_nothing_and_says_nothing() {
    let v = view("   ", 10, true, None);
    assert!(v.hits.is_empty());
    assert_eq!(v.total, 0);
    assert!(v.diagnostics.is_empty());
}

/// B46's other half, found by typing "Guppy" into the app on 2026-09-14: the wiki index
/// learned about the transformation pages and the **search index** did not, so the one place
/// that answers "where is this thing" could not name a transformation at all. Six kinds were
/// indexed of seven, the same arithmetic the wiki index was carrying — a page that exists
/// and cannot be found is indistinguishable from a page that does not exist.
#[test]
fn a_transformation_is_a_document_of_the_search_index() {
    let mut ds = dataset();
    ds.transformations.insert(
        1,
        entry_with(
            "Guppy",
            Infobox::Transformation {
                requires: Some(3),
                contributors: Vec::new(),
                target: Vec::new(),
            },
            vec![],
        ),
    );
    let index = SearchIndex::build(Ok(&ds));
    assert!(
        index.title(&Target::Transformation { id: 1 }) == Some("Guppy"),
        "the transformation is a document"
    );
}
