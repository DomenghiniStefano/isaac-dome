//! The wiki index: every page the dataset has, once per window. What the tab labels, the
//! category lists and the icon of every reference inside a page are read from.

use catalog::Catalog;
use ipc::{wiki_index, IconRef, Target, WikiIndex};
use serde_json::{json, to_value};
use wiki::{Dataset, Entry, Infobox};

const ITEMS: &[u8] =
    b"<items gfxroot=\"gfx/items/\"><passive id=\"2\" gfx=\"a.png\" name=\"A\" /></items>";

fn catalog() -> Catalog {
    Catalog::build(|p| match p {
        "items.xml" => Some(ITEMS.to_vec()),
        _ => None,
    })
}

fn entry(title: &str, infobox: Infobox) -> Entry {
    Entry {
        title: title.into(),
        revid: 1,
        infobox,
        sections: vec![],
    }
}

fn dataset() -> Dataset {
    let mut ds = Dataset::empty_for_tests();
    ds.items.insert(2, entry("A", Infobox::Item));
    ds.items.insert(9, entry("Nine", Infobox::Item));
    ds.trinkets.insert(1, entry("T", Infobox::Trinket));
    ds.bosses.insert(
        Dataset::boss_key(20, 0, 0),
        entry(
            "Monstro",
            Infobox::Boss {
                base_hp: None,
                environment: vec![],
                pool: vec![],
                unlocked_by: None,
            },
        ),
    );
    ds.meta.counts.items = 2;
    ds.meta.counts.trinkets = 1;
    ds.meta.counts.bosses = 1;
    ds
}

fn link(r: &IconRef) -> Option<String> {
    Some(format!("isaac://{}", r.to_path()))
}

#[test]
fn the_shape_is_pinned_and_icons_are_null_without_a_catalog() {
    let ds = dataset();
    let index = wiki_index(Ok(&ds), None, None, link);
    let v = to_value(&index).unwrap();
    assert_eq!(v["info"]["kind"], "loaded");
    assert_eq!(
        v["pages"][0],
        json!({ "target": { "kind": "item", "id": 2 }, "title": "A", "iconUrl": null })
    );
    assert_eq!(v["pages"].as_array().unwrap().len(), 4);
}

#[test]
fn pages_come_out_by_kind_then_by_id_and_carry_the_catalog_s_icon() {
    let ds = dataset();
    let c = catalog();
    let index: WikiIndex = wiki_index(Ok(&ds), Some(&c), None, link);
    let targets: Vec<&Target> = index.pages.iter().map(|p| &p.target).collect();
    assert_eq!(
        targets,
        vec![
            &Target::Item { id: 2 },
            &Target::Item { id: 9 },
            &Target::Trinket { id: 1 },
            &Target::Entity {
                id: 20,
                variant: 0,
                subtype: 0
            },
        ]
    );
    // The catalog knows item 2 and nothing else: one link, three placeholders.
    assert_eq!(
        index.pages[0].icon_url.as_deref(),
        Some("isaac://page/item/2")
    );
    assert!(index.pages[1..].iter().all(|p| p.icon_url.is_none()));
}

#[test]
fn a_missing_dataset_is_an_empty_index_that_says_why() {
    let err = wiki::DatasetError::Malformed { reason: "x".into() };
    let index = wiki_index(Err(&err), None, None, link);
    assert!(index.pages.is_empty());
    assert_eq!(to_value(&index.info).unwrap()["kind"], "missing");
}

#[test]
fn a_boss_key_that_is_not_three_numbers_is_left_out_not_guessed() {
    let mut ds = dataset();
    ds.bosses
        .insert("Cadavra".to_string(), entry("Cadavra", Infobox::Item));
    let index = wiki_index(Ok(&ds), None, None, link);
    assert_eq!(index.pages.len(), 4);
}

#[test]
fn the_embedded_index_counts_match_its_meta_and_stay_small() {
    let ds = Dataset::embedded().expect("embedded dataset");
    let index = wiki_index(Ok(ds), None, None, link);
    let counts = &ds.meta.counts;
    let expected = counts.items
        + counts.trinkets
        + counts.achievements
        + counts.bosses
        + counts.challenges
        + counts.characters;
    assert_eq!(index.pages.len() as u32, expected);
    let json = serde_json::to_string(&index).unwrap();
    // A generous ceiling: the index is one load per window and must stay one order of
    // magnitude under `unlock`'s (crates/ipc/tests/unlock_size.rs).
    assert!(json.len() < 256_000, "wiki index is {} bytes", json.len());
}
