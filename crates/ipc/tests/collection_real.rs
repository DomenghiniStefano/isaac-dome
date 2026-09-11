//! The Collection against the real catalog (`samples/packed`) and the live profile. Properties,
//! not values: they hold on any profile and any patch, and skip with a note without the game.

use catalog::Catalog;
use core_save::{Kind, Save};
use unpack::ResourceSet;

#[test]
fn every_listed_collectible_reads_its_own_slot() {
    let Some(packed) = test_support::packed_dir() else {
        return;
    };
    let Some(sample) = test_support::sample("live.rep+persistentgamedata1.dat") else {
        return;
    };
    let rs = ResourceSet::open(&packed);
    let c = Catalog::build(|p| rs.read(p));
    let Ok(save) = Save::open(&sample) else {
        test_support::skip("the live profile exists but doesn't read");
        return;
    };
    let items = save.flags(Kind::Items).expect("section 4");
    let achievements = save.flags(Kind::Achievements);
    let v = ipc::collection_view(Some(&c), Some(&items), achievements.as_deref(), |_| None);
    for item in &v.items {
        assert_eq!(
            item.in_collection,
            items.get(item.id as usize).copied(),
            "item {} against its slot",
            item.id
        );
    }
    let set = v
        .items
        .iter()
        .filter(|i| i.in_collection == Some(true))
        .count() as u32;
    assert_eq!(v.totals.in_collection, set);
    assert!(v.items.iter().all(|i| i.kind != ipc::ItemKindView::Trinket));
}
