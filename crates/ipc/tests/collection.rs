//! The Collection as the UI sees it: section 4 joined with the catalog's collectibles. The
//! tests that matter keep "unread" from ever reading as "not in the collection".

use catalog::Catalog;
use ipc::{collection_view, CollectionDiagnostic, CollectionView, LockView};
use serde_json::{json, to_value};
use wiki::{Infobox, Target};

const ITEMS: &[u8] = b"<items gfxroot=\"gfx/items/\"><passive id=\"1\" gfx=\"a.png\" name=\"A\" achievement=\"1\" /><active id=\"2\" gfx=\"b.png\" name=\"B\" /><familiar id=\"5\" gfx=\"c.png\" name=\"C\" achievement=\"2\" /><trinket id=\"1\" gfx=\"t.png\" name=\"T\" /></items>";
const META: &[u8] = b"<items><item id=\"1\" quality=\"4\" tags=\"\"/><item id=\"2\" quality=\"1\" tags=\"\"/></items>";
const POOLS: &[u8] = b"<ItemPools><Pool Name=\"treasure\"><Item Id=\"1\" Weight=\"1\" DecreaseBy=\"1\" RemoveOn=\"0.1\"/><Item Id=\"2\" Weight=\"1\" DecreaseBy=\"1\" RemoveOn=\"0.1\"/></Pool><Pool Name=\"boss\"><Item Id=\"1\" Weight=\"1\" DecreaseBy=\"1\" RemoveOn=\"0.1\"/></Pool><Pool Name=\"angel\"></Pool></ItemPools>";
const ACH: &[u8] = b"<achievements gfxroot=\"gfx/ui/achievement/\"><achievement id=\"1\" text=\"t1\" gfx=\"1.png\" /><achievement id=\"2\" text=\"t2\" gfx=\"2.png\" /></achievements>";

fn catalog() -> Catalog {
    Catalog::build(|p| match p {
        "items.xml" => Some(ITEMS.to_vec()),
        "items_metadata.xml" => Some(META.to_vec()),
        "itempools.xml" => Some(POOLS.to_vec()),
        "achievements.xml" => Some(ACH.to_vec()),
        _ => None,
    })
}

// Slots 0..=3: item 1 is in the collection, item 2 isn't, item 5 has no slot.
const SLOTS: [bool; 4] = [false, true, false, false];
// Achievement 1 done, 2 not.
const DONE: [bool; 3] = [false, true, false];

fn view(items: Option<&[bool]>, achievements: Option<&[bool]>) -> CollectionView {
    collection_view(Some(&catalog()), None, items, achievements, |_| None)
}

#[test]
fn the_shapes_are_pinned() {
    assert_eq!(
        to_value(LockView::Free).expect("serializes"),
        json!({ "kind": "free" })
    );
    assert_eq!(
        to_value(LockView::Locked {
            achievement: 2,
            text: Some("t2".into()),
            page: None
        })
        .expect("serializes"),
        json!({ "kind": "locked", "achievement": 2, "text": "t2", "page": null }),
        "`page: null` is a key the screen reads: the dataset has no page for this one"
    );
    assert_eq!(
        to_value(LockView::Locked {
            achievement: 2,
            text: Some("t2".into()),
            page: Some(Target::Achievement { id: 2 })
        })
        .expect("serializes"),
        json!({
            "kind": "locked", "achievement": 2, "text": "t2",
            "page": { "kind": "achievement", "id": 2 }
        })
    );
    assert_eq!(
        to_value(CollectionDiagnostic::ItemsBeyondSlots { count: 1 }).expect("serializes"),
        json!({ "kind": "itemsBeyondSlots", "count": 1 })
    );
    let v = to_value(view(Some(&SLOTS), Some(&DONE))).expect("serializes");
    assert_eq!(v["items"][0]["inCollection"], json!(true));
    assert_eq!(v["items"][0]["iconUrl"], json!(null));
    assert_eq!(v["totals"]["inCollection"], json!(1));
    assert_eq!(v["items"][2]["kind"], json!("familiar"));
}

#[test]
fn only_collectibles_are_listed_by_id() {
    let v = view(Some(&SLOTS), Some(&DONE));
    assert_eq!(
        v.items.iter().map(|i| i.id).collect::<Vec<_>>(),
        vec![1, 2, 5],
        "the trinket has no slot in section 4 and isn't listed"
    );
    assert_eq!(v.items[0].name, "A");
}

#[test]
fn the_collection_flag_comes_from_the_slot_and_a_missing_slot_is_null() {
    let v = view(Some(&SLOTS), Some(&DONE));
    assert_eq!(
        v.items.iter().map(|i| i.in_collection).collect::<Vec<_>>(),
        vec![Some(true), Some(false), None]
    );
    assert_eq!(
        (v.totals.slots, v.totals.items, v.totals.in_collection),
        (4, 3, 1)
    );
    assert_eq!(
        v.diagnostics,
        vec![CollectionDiagnostic::ItemsBeyondSlots { count: 1 }]
    );
}

#[test]
fn an_unread_collection_is_null_everywhere_and_says_so() {
    let v = view(None, Some(&DONE));
    assert!(
        v.items.iter().all(|i| i.in_collection.is_none()),
        "unread is never false"
    );
    assert_eq!((v.totals.slots, v.totals.in_collection), (0, 0));
    assert_eq!(
        v.diagnostics,
        vec![CollectionDiagnostic::NoCollectionSection]
    );
}

#[test]
fn locks_follow_the_unlocking_achievement() {
    let v = view(Some(&SLOTS), Some(&DONE));
    assert_eq!(
        v.items.iter().map(|i| i.lock.clone()).collect::<Vec<_>>(),
        vec![
            LockView::Unlocked {
                achievement: 1,
                text: Some("t1".into()),
                page: None
            },
            LockView::Free,
            LockView::Locked {
                achievement: 2,
                text: Some("t2".into()),
                page: None
            },
        ]
    );
    let unread = view(Some(&SLOTS), None);
    assert_eq!(
        unread
            .items
            .iter()
            .map(|i| i.lock.clone())
            .collect::<Vec<_>>(),
        vec![
            LockView::Unknown {
                achievement: 1,
                text: Some("t1".into()),
                page: None
            },
            LockView::Free,
            LockView::Unknown {
                achievement: 2,
                text: Some("t2".into()),
                page: None
            },
        ]
    );
    assert!(unread
        .diagnostics
        .contains(&CollectionDiagnostic::NoAchievementSection));
}

#[test]
fn quality_pools_and_origin_come_from_the_catalog() {
    let v = view(Some(&SLOTS), Some(&DONE));
    assert_eq!(
        v.items.iter().map(|i| i.quality).collect::<Vec<_>>(),
        vec![Some(4), Some(1), None]
    );
    assert_eq!(v.items[0].pools, vec!["treasure", "boss"]);
    assert_eq!(v.items[1].pools, vec!["treasure"]);
    assert!(v.items[2].pools.is_empty());
    assert_eq!(
        v.pools,
        vec!["treasure", "boss"],
        "angel holds no listed item"
    );
    assert_eq!(
        to_value(v.items[0].origin).expect("serializes"),
        json!("rebirth")
    );
}

#[test]
fn without_a_catalog_only_the_totals_speak() {
    let flags = [false, true, true];
    let v = collection_view(None, None, Some(&flags), None, |_| None);
    assert!(v.items.is_empty() && v.pools.is_empty());
    assert_eq!(
        (v.totals.slots, v.totals.items, v.totals.in_collection),
        (3, 0, 2)
    );
    assert_eq!(v.diagnostics, vec![CollectionDiagnostic::NoCatalog]);
}

/// The lock links to the achievement's page, and only when the dataset has it. `free` carries
/// no page because nothing unlocks the item — a different sentence from "no page".
#[test]
fn a_lock_carries_the_achievement_page_only_when_the_dataset_has_it() {
    let mut ds = wiki::for_tests::empty_dataset();
    // Achievement 2 has a page, achievement 1 has none: one of each, in one view.
    ds.achievements.insert(
        2,
        wiki::for_tests::entry(
            "t2",
            Infobox::Achievement {
                quote: vec![],
                requirements: vec![],
                notes: vec![],
                unlocks: None,
            },
        ),
    );

    let v = collection_view(
        Some(&catalog()),
        Some(&ds),
        Some(&SLOTS),
        Some(&DONE),
        |_| None,
    );
    let lock = |id: u32| {
        to_value(
            &v.items
                .iter()
                .find(|i| i.id == id)
                .expect("the item is in the catalog")
                .lock,
        )
        .expect("serializes")
    };
    assert_eq!(lock(5)["kind"], "locked");
    assert_eq!(
        lock(5)["page"],
        json!({ "kind": "achievement", "id": 2 }),
        "the badge's menu opens the achievement that unlocks the item"
    );
    assert_eq!(
        lock(1)["page"],
        json!(null),
        "the dataset has no page for achievement 1: the name shows, and does not link"
    );
    assert_eq!(lock(2)["kind"], "free", "nothing unlocks item 2");
    assert_eq!(
        lock(2).get("page"),
        None,
        "a free item carries no page key at all: there is nothing to open"
    );

    // No dataset at all: every lock reads the same, and nothing links.
    let without = collection_view(Some(&catalog()), None, Some(&SLOTS), Some(&DONE), |_| None);
    assert!(without.items.iter().all(|i| match &i.lock {
        LockView::Free => true,
        LockView::Unlocked { page, .. }
        | LockView::Locked { page, .. }
        | LockView::Unknown { page, .. } => page.is_none(),
    }));
}
