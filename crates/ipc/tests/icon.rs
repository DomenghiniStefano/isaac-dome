//! The icon reference: what a row carries instead of a base64 image.
//!
//! The Tauri crate renders it into a URL and its protocol handler parses it back, so the
//! **round trip is the whole contract**: a reference that renders to something the handler
//! can't parse is an image that silently never appears. `unpack` taught this lesson once
//! already — a wrong path doesn't raise an error, it goes quiet.

use catalog::Catalog;
use ipc::{icon_source, IconRef, ItemKindView, MarkTier};

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

#[test]
fn marks_and_heads_survive_the_round_trip() {
    let all = [
        IconRef::Mark {
            column: 0,
            tier: MarkTier::Normal,
        },
        IconRef::Mark {
            column: 9,
            tier: MarkTier::Hard,
        },
        IconRef::Mark {
            column: 11,
            tier: MarkTier::Hard,
        },
        IconRef::Head { row: 0 },
        IconRef::Head { row: 33 },
    ];
    for r in all {
        let path = r.to_path();
        assert_eq!(IconRef::parse(&path), Some(r), "round trip of {path:?}");
    }
    assert_eq!(
        IconRef::Mark {
            column: 9,
            tier: MarkTier::Hard
        }
        .to_path(),
        "mark/9/hard"
    );
    assert_eq!(IconRef::Head { row: 17 }.to_path(), "head/17");
}

#[test]
fn a_mark_or_head_outside_the_matrix_parses_to_nothing() {
    // Twelve columns and 34 rows: past them the handler answers 400 before it opens an
    // archive.
    for bad in [
        "mark",
        "mark/0",
        "mark/12/hard",
        "mark/0/wizard",
        "mark/x/hard",
        "mark/0/hard/extra",
        "head",
        "head/",
        "head/34",
        "head/x",
        "head/0/extra",
    ] {
        assert_eq!(IconRef::parse(bad), None, "{bad:?} must not parse");
    }
}

const PLAYERS: &[u8] = b"<players portraitroot=\"gfx/ui/stage/\">
<player id=\"0\" name=\"#ISAAC_NAME\" portrait=\"PlayerPortrait_Isaac.png\" />
<player id=\"21\" name=\"#ISAAC_NAME\" portrait=\"PlayerPortrait_Isaac_b.png\" />
</players>";

/// `coop menu.anm2` shaped like the game's: frame 0 without a crop (the "?" placeholder),
/// then one 32px cell per frame, eight to a row.
fn coop_menu_anm2() -> Vec<u8> {
    let frames: String = (1..=37)
        .map(|f| {
            format!(
                r#"<Frame XCrop="{}" YCrop="{}" Width="32" Height="32" Visible="true"/>"#,
                32 * (f % 8),
                32 * (f / 8)
            )
        })
        .collect();
    format!(
        r#"<AnimatedActor><Content><Spritesheets><Spritesheet Path="coop menu.png" Id="0"/></Spritesheets><Layers><Layer Name="Main" Id="0" SpritesheetId="0"/></Layers></Content><Animations><Animation Name="Main"><LayerAnimations><LayerAnimation LayerId="0"><Frame Delay="1" Visible="true"/>{frames}</LayerAnimation></LayerAnimations></Animation></Animations></AnimatedActor>"#
    )
    .into_bytes()
}

fn catalog_with_heads() -> Catalog {
    let anm2 = coop_menu_anm2();
    Catalog::build(|p| match p {
        "players.xml" => Some(PLAYERS.to_vec()),
        "gfx/ui/coop menu.anm2" => Some(anm2.clone()),
        _ => None,
    })
}

#[test]
fn a_head_resolves_through_the_matrix_row() {
    let c = catalog_with_heads();
    let at = |row| {
        icon_source(&c, &IconRef::Head { row })
            .map(|s| (s.path.clone(), s.rect.map(|r| (r.x, r.y, r.w, r.h))))
    };
    // Row 0 is Isaac (id 0, frame 1 = column 1, row 0); row 17 is T. Isaac (id 21, frame 21
    // = column 5, row 2).
    assert_eq!(
        at(0),
        Some(("gfx/ui/coop menu.png".to_string(), Some((32, 0, 32, 32))))
    );
    assert_eq!(
        at(17),
        Some(("gfx/ui/coop menu.png".to_string(), Some((160, 64, 32, 32))))
    );
    assert_eq!(
        at(1),
        None,
        "Magdalene isn't in this catalog: no head, not a neighbour's"
    );
}

#[test]
fn a_mark_is_not_in_the_catalog() {
    let c = catalog_with_heads();
    assert!(icon_source(
        &c,
        &IconRef::Mark {
            column: 0,
            tier: MarkTier::Hard
        }
    )
    .is_none());
}
