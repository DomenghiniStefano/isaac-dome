//! `wiki_progress`: the save's state per wiki page, on synthetic fixtures (Task 2 of
//! `2026-09-27-wiki-restyle.md`). Real-data coverage is `wiki_progress_real.rs`.

use catalog::Catalog;
use core_save::{Bestiary, Column, EntityId, Record, Tally};
use ipc::{wiki_progress, PageProgress, PageProgressEntry, WikiProgress, WikiProgressInputs};
use serde_json::{json, to_value};
use wiki::for_tests::{empty_article, empty_dataset, empty_item, empty_trinket, entry};
use wiki::{ArticleCategory, Dataset, DatasetError, Infobox, Target};

const ITEMS: &[u8] = b"<items gfxroot=\"gfx/items/\"><passive id=\"1\" gfx=\"a.png\" name=\"A\" achievement=\"5\" /><trinket id=\"1\" gfx=\"t.png\" name=\"T\" achievement=\"7\" /></items>";
const ACH: &[u8] = b"<achievements gfxroot=\"gfx/ui/achievement/\"><achievement id=\"5\" text=\"t5\" gfx=\"5.png\" /><achievement id=\"7\" text=\"t7\" gfx=\"7.png\" /></achievements>";
const PLAYERS: &[u8] = b"<players root=\"gfx/characters/costumes/\" portraitroot=\"gfx/ui/stage/\">
\t<player id=\"0\" name=\"#ISAAC_NAME\" portrait=\"PlayerPortrait_Isaac.png\" />
\t<player id=\"21\" name=\"#ISAAC_NAME\" achievement=\"474\" portrait=\"PlayerPortrait_Isaac_b.png\" />
</players>";

fn catalog() -> Catalog {
    Catalog::build(|p| match p {
        "items.xml" => Some(ITEMS.to_vec()),
        "achievements.xml" => Some(ACH.to_vec()),
        "players.xml" => Some(PLAYERS.to_vec()),
        _ => None,
    })
}

fn empty_character() -> Infobox {
    Infobox::Character {
        health: vec![],
        damage: String::new(),
        tears: String::new(),
        range: String::new(),
        speed: String::new(),
        luck: String::new(),
        shot_speed: String::new(),
        pickups: vec![],
        collectibles: vec![],
        parent: None,
    }
}

fn empty_boss() -> Infobox {
    Infobox::Boss {
        base_hp: None,
        stage_hp: vec![],
        variant: None,
        environment: vec![],
        pool: vec![],
    }
}

fn empty_entity() -> Infobox {
    Infobox::Entity {
        base_hp: None,
        stage_hp: vec![],
        environment: vec![],
        behavior: vec![],
        pool: vec![],
        replace: vec![],
        replace_chance: vec![],
        replace_notes: vec![],
    }
}

/// Every input `None`, against `ds`: each test sets only the fields its case is about.
fn inputs(ds: &Dataset) -> WikiProgressInputs<'_> {
    WikiProgressInputs {
        dataset: Ok(ds),
        catalog: None,
        achievements: None,
        items: None,
        challenges: None,
        counters: None,
        bestiary: None,
    }
}

#[test]
fn a_dataset_that_did_not_load_gives_no_pages() {
    let err = DatasetError::Malformed { reason: "x".into() };
    let i = WikiProgressInputs {
        dataset: Err(&err),
        catalog: None,
        achievements: None,
        items: None,
        challenges: None,
        counters: None,
        bestiary: None,
    };
    assert_eq!(wiki_progress(i).pages, vec![]);
}

#[test]
fn an_achievement_is_absent_without_section_one_and_present_with_it() {
    let mut ds = empty_dataset();
    ds.achievements.insert(
        2,
        entry(
            "A2",
            Infobox::Achievement {
                quote: vec![],
                requirements: vec![],
                notes: vec![],
                unlocks: None,
            },
        ),
    );

    assert_eq!(
        wiki_progress(inputs(&ds)).pages,
        vec![],
        "section 1 unread: the row must not claim done or not done"
    );

    let mut i = inputs(&ds);
    i.achievements = Some(&[false, false, true]);
    assert_eq!(
        wiki_progress(i).pages,
        vec![PageProgressEntry {
            target: Target::Achievement { id: 2 },
            progress: PageProgress::Achievement { done: true },
        }]
    );
}

#[test]
fn an_item_is_collected_and_locked_by_its_own_achievement() {
    let mut ds = empty_dataset();
    ds.items.insert(1, entry("A", empty_item()));
    let c = catalog();
    let mut i = inputs(&ds);
    i.catalog = Some(&c);
    i.items = Some(&[false, true]); // slot 1 set: collected
    i.achievements = Some(&[false, false, false, false, false, false]); // achievement 5 not done

    assert_eq!(
        wiki_progress(i).pages,
        vec![PageProgressEntry {
            target: Target::Item { id: 1 },
            progress: PageProgress::Item {
                collected: Some(true),
                unlocked: Some(false),
                unlocked_by: Some(5),
            },
        }]
    );
}

#[test]
fn a_trinket_reads_its_own_gate_from_the_catalog_not_the_collection_section() {
    let mut ds = empty_dataset();
    ds.trinkets.insert(1, entry("T", empty_trinket()));
    let c = catalog();
    let flags: Vec<bool> = (0..8).map(|n| n == 7).collect();
    let mut i = inputs(&ds);
    i.catalog = Some(&c);
    i.achievements = Some(&flags);

    assert_eq!(
        wiki_progress(i).pages,
        vec![PageProgressEntry {
            target: Target::Trinket { id: 1 },
            progress: PageProgress::Unlockable {
                unlocked: true,
                unlocked_by: 7,
            },
        }]
    );
}

/// Review Focus 3: Isaac and Tainted Isaac share a name (`#ISAAC_NAME`); their marks are
/// looked up by id and Tainted flag, never by name, and must not be mixed up.
#[test]
fn isaac_and_tainted_isaac_get_their_own_marks() {
    let mut ds = empty_dataset();
    ds.characters.insert(0, entry("Isaac", empty_character()));
    ds.characters
        .insert(21, entry("Tainted Isaac", empty_character()));
    let c = catalog();

    let isaac = c
        .character(catalog::CharacterId(0))
        .expect("fixture has Isaac");
    let tainted = c
        .character(catalog::CharacterId(21))
        .expect("fixture has Tainted Isaac");
    let isaac_row = ipc::for_tests::row_for_character(isaac).expect("Isaac has a roster row");
    let tainted_row =
        ipc::for_tests::row_for_character(tainted).expect("Tainted Isaac has a roster row");
    assert_ne!(isaac_row, tainted_row, "two distinct rows for one name");

    // One column marked (bits 3 = normal + hard) for Isaac's row alone.
    let moms_heart = Column::MomsHeart;
    let index = core_save::cell_index(isaac_row, moms_heart).expect("Isaac's row is located");
    let mut counters = vec![0u32; index + 1];
    counters[index] = 3;

    let mut i = inputs(&ds);
    i.catalog = Some(&c);
    i.counters = Some(&counters);

    let out = wiki_progress(i);
    let marks_done_of = |id: u32| {
        let PageProgress::Character { marks_done, .. } = out
            .pages
            .iter()
            .find(|p| p.target == Target::Character { id })
            .map(|p| p.progress.clone())
            .unwrap_or_else(|| panic!("no row for character {id}"))
        else {
            panic!("expected a character");
        };
        marks_done
    };
    assert_eq!(marks_done_of(0), 1, "Isaac's own row has the mark");
    assert_eq!(
        marks_done_of(21),
        0,
        "Tainted Isaac's row must not see Isaac's mark"
    );
}

/// Review Focus 4: a boss and an entity page reaching the same bestiary key each read that
/// key's own numbers, never a shared or halved total.
#[test]
fn two_pages_on_one_bestiary_key_each_get_that_keys_numbers() {
    let mut ds = empty_dataset();
    ds.bosses
        .insert("20.1.0".into(), entry("Peep Eye", empty_boss()));
    ds.entities
        .insert("20.1.0".into(), entry("The Bloat", empty_entity()));

    let key = EntityId {
        kind: 20,
        variant: 1,
        subtype: 0,
    };
    let bestiary = Bestiary {
        tallies: vec![
            Tally {
                id: 1,
                records: vec![Record {
                    entity: key,
                    count: 7,
                }],
            },
            Tally {
                id: 2,
                records: vec![Record {
                    entity: key,
                    count: 4,
                }],
            },
            Tally {
                id: 3,
                records: vec![Record {
                    entity: key,
                    count: 999,
                }],
            },
            Tally {
                id: 4,
                records: vec![Record {
                    entity: key,
                    count: 2,
                }],
            },
        ],
        declared_tallies: 4,
        declared_units: 0,
        trailing: vec![],
    };

    let mut i = inputs(&ds);
    i.bestiary = Some(&bestiary);

    let out = wiki_progress(i);
    assert_eq!(out.pages.len(), 2, "one row per page, not one shared row");
    for p in &out.pages {
        assert_eq!(
            p.progress,
            PageProgress::Bestiary {
                met: 7,
                killed: 4,
                killed_you: 2
            },
            "tally 3 must never surface, on {:?}",
            p.target
        );
    }
}

#[test]
fn a_bestiary_page_carries_no_row_when_the_section_did_not_read() {
    let mut ds = empty_dataset();
    ds.bosses.insert("1.0.0".into(), entry("X", empty_boss()));
    assert_eq!(
        wiki_progress(inputs(&ds)).pages,
        vec![],
        "bestiary section unread: no row"
    );
}

#[test]
fn a_card_reads_its_gate_from_the_wikis_own_unlocked_by_the_catalog_has_none() {
    let mut ds = empty_dataset();
    let mut e = entry("A Card", empty_article());
    e.infobox = Infobox::Article {
        category: Some(ArticleCategory::Card),
    };
    e.unlocked_by = Some(Target::Achievement { id: 9 });
    ds.articles.insert("A Card".into(), e);

    let flags: Vec<bool> = (0..10).map(|n| n == 9).collect();
    let mut i = inputs(&ds);
    i.achievements = Some(&flags);

    assert_eq!(
        wiki_progress(i).pages,
        vec![PageProgressEntry {
            target: Target::Article {
                title: "A Card".into()
            },
            progress: PageProgress::Unlockable {
                unlocked: true,
                unlocked_by: 9,
            },
        }]
    );
}

#[test]
fn a_version_article_carries_no_progress() {
    let mut ds = empty_dataset();
    let mut e = entry("V1.0", empty_article());
    e.infobox = Infobox::Article {
        category: Some(ArticleCategory::Version),
    };
    ds.articles.insert("V1.0".into(), e);
    assert_eq!(wiki_progress(inputs(&ds)).pages, vec![]);
}

#[test]
fn the_shapes_are_pinned() {
    assert_eq!(
        to_value(PageProgress::Achievement { done: true }).unwrap(),
        json!({ "kind": "achievement", "done": true })
    );
    assert_eq!(
        to_value(PageProgress::Item {
            collected: Some(true),
            unlocked: None,
            unlocked_by: Some(5)
        })
        .unwrap(),
        json!({ "kind": "item", "collected": true, "unlocked": null, "unlockedBy": 5 })
    );
    assert_eq!(
        to_value(PageProgress::Unlockable {
            unlocked: false,
            unlocked_by: 7
        })
        .unwrap(),
        json!({ "kind": "unlockable", "unlocked": false, "unlockedBy": 7 })
    );
    assert_eq!(
        to_value(PageProgress::Bestiary {
            met: 1,
            killed: 2,
            killed_you: 3
        })
        .unwrap(),
        json!({ "kind": "bestiary", "met": 1, "killed": 2, "killedYou": 3 })
    );
    assert_eq!(
        to_value(WikiProgress {
            pages: vec![PageProgressEntry {
                target: Target::Achievement { id: 1 },
                progress: PageProgress::Achievement { done: false },
            }]
        })
        .unwrap(),
        json!({
            "pages": [{
                "target": {"kind": "achievement", "id": 1},
                "progress": {"kind": "achievement", "done": false}
            }]
        })
    );
}
