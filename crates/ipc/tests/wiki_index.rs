//! The wiki index: every page the dataset has, once per window. What the tab labels, the
//! category lists and the icon of every reference inside a page are read from.

use catalog::Catalog;
use ipc::{wiki_index, IconRef, Target, WikiIndex};
use serde_json::{json, to_value};
use wiki::Dataset;

const ITEMS: &[u8] =
    b"<items gfxroot=\"gfx/items/\"><passive id=\"2\" gfx=\"a.png\" name=\"A\" /></items>";

fn catalog() -> Catalog {
    Catalog::build(|p| match p {
        "items.xml" => Some(ITEMS.to_vec()),
        _ => None,
    })
}

use wiki::for_tests::{empty_boss, empty_item, empty_trinket, entry};

fn dataset() -> Dataset {
    let mut ds = wiki::for_tests::empty_dataset();
    ds.items.insert(2, entry("A", empty_item()));
    ds.items.insert(9, entry("Nine", empty_item()));
    ds.trinkets.insert(1, entry("T", empty_trinket()));
    ds.bosses
        .insert(Dataset::boss_key(20, 0, 0), entry("Monstro", empty_boss()));
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
    let index = wiki_index(Ok(&ds), None, ipc::BossKeys::NONE, None, link);
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
    let index: WikiIndex = wiki_index(Ok(&ds), Some(&c), &ipc::for_tests::bosses(&c), None, link);
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
    let index = wiki_index(Err(&err), None, ipc::BossKeys::NONE, None, link);
    assert!(index.pages.is_empty());
    assert_eq!(to_value(&index.info).unwrap()["kind"], "missing");
}

#[test]
fn a_boss_key_that_is_not_three_numbers_is_left_out_not_guessed() {
    let mut ds = dataset();
    ds.bosses
        .insert("Cadavra".to_string(), entry("Cadavra", empty_item()));
    let index = wiki_index(Ok(&ds), None, ipc::BossKeys::NONE, None, link);
    assert_eq!(index.pages.len(), 4);
}

#[test]
fn the_embedded_index_counts_match_its_meta_and_stay_small() {
    let ds = Dataset::embedded().expect("embedded dataset");
    let index = wiki_index(Ok(ds), None, ipc::BossKeys::NONE, None, link);
    let counts = &ds.meta.counts;
    // Every kind the dataset counts, and the sum is the point: this test could have caught
    // B46 the day the transformations entered the dataset, and did not, because the sum
    // left the same kind out that the index did. A count that is not added here is a kind
    // the index may quietly stop carrying.
    let expected = counts.items
        + counts.trinkets
        + counts.achievements
        + counts.bosses
        + counts.challenges
        + counts.characters
        + counts.transformations;
    assert_eq!(index.pages.len() as u32, expected);
    let json = serde_json::to_string(&index).unwrap();
    // A generous ceiling: the index is one load per window and must stay one order of
    // magnitude under `unlock`'s (crates/ipc/tests/unlock_size.rs).
    assert!(json.len() < 256_000, "wiki index is {} bytes", json.len());
}

/// B46. A transformation has had a page in the dataset since the transformations
/// sub-project (2026-09-13) — `Dataset::entry` answers `Target::Transformation` — and the
/// index was never told: it lists six kinds and leaves the sixteen out. Nothing downstream
/// could show them, because the index is what the category lists and every reference's icon
/// are read from. The premise that "a transformation has no page" was true when the index
/// was written and stopped being true the next day.
#[test]
fn a_transformation_is_a_page_of_the_index() {
    let mut ds = dataset();
    ds.transformations.insert(
        1,
        entry(
            "Guppy",
            wiki::Infobox::Transformation {
                requires: Some(3),
                contributors: Vec::new(),
                target: Vec::new(),
            },
        ),
    );
    ds.meta.counts.transformations = 1;
    let index: WikiIndex = wiki_index(Ok(&ds), None, ipc::BossKeys::NONE, None, link);
    let page = index
        .pages
        .iter()
        .find(|p| p.target == Target::Transformation { id: 1 })
        .expect("the transformation is a page of the index");
    assert_eq!(page.title, "Guppy");
}
