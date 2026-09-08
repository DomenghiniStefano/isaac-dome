//! The icon reference: what a row carries instead of a base64 image.
//!
//! The Tauri crate renders it into a URL and its protocol handler parses it back, so the
//! **round trip is the whole contract**: a reference that renders to something the handler
//! can't parse is an image that silently never appears. `unpack` taught this lesson once
//! already — a wrong path doesn't raise an error, it goes quiet.

use catalog::Catalog;
use ipc::{icon_source, IconRef, ItemKindView};

const ITEMS: &[u8] = b"<items gfxroot=\"gfx/items/\"><passive id=\"2\" gfx=\"a.png\" name=\"A\" achievement=\"1\" /></items>";
const ACH: &[u8] = b"<achievements gfxroot=\"gfx/ui/achievement/\"><achievement id=\"1\" text=\"t1\" gfx=\"1.png\" /></achievements>";

fn catalog() -> Catalog {
    Catalog::build(|p| match p {
        "items.xml" => Some(ITEMS.to_vec()),
        "achievements.xml" => Some(ACH.to_vec()),
        _ => None,
    })
}

#[test]
fn every_reference_survives_the_round_trip() {
    let all = [
        IconRef::Achievement { id: 19 },
        IconRef::Item {
            kind: ItemKindView::Passive,
            id: 92,
        },
        IconRef::Item {
            kind: ItemKindView::Active,
            id: 105,
        },
        IconRef::Item {
            kind: ItemKindView::Familiar,
            id: 207,
        },
        IconRef::Item {
            kind: ItemKindView::Trinket,
            id: 49,
        },
    ];
    for r in all {
        let path = r.to_path();
        assert_eq!(IconRef::parse(&path), Some(r), "round trip of {path:?}");
    }
}

#[test]
fn a_path_that_is_not_ours_parses_to_nothing() {
    // Never a guess and never a panic: the handler answers "no image" and the UI draws the
    // placeholder brief §5.6 asks for.
    for bad in [
        "",
        "achievement",
        "achievement/",
        "achievement/x",
        "achievement/19/extra",
        "item/passive",
        "item/wizard/1",
        "item/passive/x",
        "character/3",
        "../../secret",
    ] {
        assert_eq!(IconRef::parse(bad), None, "{bad:?} should not parse");
    }
}

#[test]
fn a_reference_resolves_to_the_file_the_catalog_names() {
    let c = catalog();
    assert_eq!(
        icon_source(&c, &IconRef::Achievement { id: 1 }).map(|s| s.path.as_str()),
        Some("gfx/ui/achievement/1.png")
    );
    assert_eq!(
        icon_source(
            &c,
            &IconRef::Item {
                kind: ItemKindView::Passive,
                id: 2
            }
        )
        .map(|s| s.path.as_str()),
        // The subfolder comes from the kind, which the XML doesn't state: `kind.folder()`.
        Some("gfx/items/collectibles/a.png")
    );
}

#[test]
fn the_kind_is_part_of_the_key_not_decoration() {
    // 186 ids are shared between a collectible and a trinket — item 1 is both *The Sad
    // Onion* and *Swallowed Penny*. A reference that ignored the kind would serve the
    // wrong picture, plausibly, and nobody would notice.
    let c = catalog();
    assert!(icon_source(
        &c,
        &IconRef::Item {
            kind: ItemKindView::Trinket,
            id: 2
        }
    )
    .is_none());
}

#[test]
fn an_id_the_catalog_does_not_know_resolves_to_nothing() {
    let c = catalog();
    assert!(icon_source(&c, &IconRef::Achievement { id: 999 }).is_none());
}
