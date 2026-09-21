//! The icon reference: what a row carries instead of a base64 image.
//!
//! The Tauri crate renders it into a URL and its protocol handler parses it back, so the
//! **round trip is the whole contract**: a reference that renders to something the handler
//! can't parse is an image that silently never appears. `unpack` taught this lesson once
//! already — a wrong path doesn't raise an error, it goes quiet.

use catalog::Catalog;
use ipc::{icon_source, IconRef, ItemKindView, MarkFill, MarkTier, Target};

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

#[test]
fn a_page_reference_survives_the_round_trip_for_every_kind_that_has_a_page() {
    let pages = [
        Target::Item { id: 105 },
        Target::Trinket { id: 97 },
        Target::Achievement { id: 1 },
        Target::Challenge { number: 19 },
        Target::Character { id: 0 },
        Target::Entity {
            id: 20,
            variant: 0,
            subtype: 0,
        },
    ];
    for target in pages {
        let r = IconRef::Page { target };
        let path = r.to_path();
        assert!(path.starts_with("page/"), "{path}");
        assert_eq!(IconRef::parse(&path), Some(r), "round trip of {path:?}");
    }
    assert_eq!(
        IconRef::Page {
            target: Target::Entity {
                id: 20,
                variant: 0,
                subtype: 0
            }
        }
        .to_path(),
        "page/entity/20/0/0"
    );
}

#[test]
fn a_target_with_no_page_has_no_path() {
    // Stages, rooms, pickups and transformations have no page in the dataset, so no figure
    // to serve: the handler refuses the string instead of guessing.
    for bad in [
        "page/stage/Basement",
        "page/room/x",
        "page/pickup/Chest",
        "page/transformation/1",
        "page/item",
        "page/entity/20/0",
        "page/item/1/2",
        "page/none",
    ] {
        assert_eq!(IconRef::parse(bad), None, "{bad:?}");
    }
}

#[test]
fn a_page_icon_resolves_through_target_sprite() {
    let c = catalog();
    let found = icon_source(
        &c,
        &IconRef::Page {
            target: Target::Item { id: 2 },
        },
    );
    assert_eq!(
        found.map(|s| s.path.as_str()),
        Some("gfx/items/collectibles/a.png")
    );
    assert!(icon_source(
        &c,
        &IconRef::Page {
            target: Target::Item { id: 99 }
        }
    )
    .is_none());
    // A challenge the catalog doesn't list has no art, and says nothing.
    assert!(icon_source(
        &c,
        &IconRef::Page {
            target: Target::Challenge { number: 1 }
        }
    )
    .is_none());
}

// Which references are served trimmed to their drawing, and which are served as the anm2
// cut them. It is a reading of the game and not a preference, so it lives in the pure crate
// and `app/icons.rs` only obeys it.
//
// Only the room kinds. Their sheet is the one measured off-centre (`sprite_png::trim_opaque`),
// and the Floor's cell is the one place that draws a sprite at a fixed pixel scale inside a
// 2rem square. Everywhere else a sprite is fitted to a box, so trimming would rescale
// pictures on six screens to fix one — the same drawing bigger on the row whose margin
// happened to be wider.

#[test]
fn only_a_room_icon_is_trimmed_to_its_drawing() {
    assert!(IconRef::Room {
        kind: ipc::RoomKindView::Boss
    }
    .trims_to_drawing());
    let fitted = [
        IconRef::Achievement { id: 19 },
        IconRef::Item {
            kind: ItemKindView::Passive,
            id: 92,
        },
        IconRef::Mark {
            column: 0,
            tier: MarkTier::Hard,
        },
        IconRef::Head { row: 0 },
        IconRef::Page {
            target: Target::Item { id: 105 },
        },
    ];
    for reference in fitted {
        assert!(
            !reference.trims_to_drawing(),
            "{reference:?} is fitted to a box: trimming would rescale it"
        );
    }
}

#[test]
fn every_room_kind_is_trimmed_not_only_the_ones_with_an_icon() {
    // Start and Normal have no icon in the game's file. The rule is about the reference, not
    // about whether it resolves: a kind that gains an icon in a patch must not need a second
    // decision here to be drawn like its thirteen neighbours.
    for kind in ipc::ROOM_KINDS {
        assert!(IconRef::Room { kind }.trims_to_drawing(), "{kind:?}");
    }
}

// The widget's address. It is the only reference that carries **state** rather than
// identity: the picture it names depends on what the profile has done, so the twelve
// columns travel in the path and the handler composes what they ask for.

/// `-`, `n`, `h` per column, in `BOSSES` order, read back into the array.
fn fills(spelled: &str) -> [MarkFill; 12] {
    let mut out = [MarkFill::None; 12];
    for (i, c) in spelled.chars().enumerate() {
        out[i] = match c {
            'n' => MarkFill::Normal,
            'h' => MarkFill::Hard,
            _ => MarkFill::None,
        };
    }
    out
}

#[test]
fn the_widget_survives_the_round_trip() {
    for spelled in [
        "------------",
        "hhhhhhhhhhhh",
        "nnnnnnnnnnnn",
        "hn-hn-hn-hn-",
    ] {
        let r = IconRef::Widget {
            fills: fills(spelled),
        };
        let path = r.to_path();
        assert_eq!(path, format!("widget/{spelled}"));
        assert_eq!(IconRef::parse(&path), Some(r), "round trip of {path:?}");
    }
}

#[test]
fn a_widget_address_of_the_wrong_shape_is_not_ours() {
    // Twelve columns exactly. A shorter string would compose a picture missing a mark and
    // look like a profile that hasn't got it — a plausible wrong answer, which is the kind
    // this protocol refuses on principle.
    for bad in [
        "widget",
        "widget/",
        "widget/hhhhhhhhhhh",
        "widget/hhhhhhhhhhhhh",
        "widget/hhhhhhhhhhhx",
        "widget/hhhhhhhhhhhh/extra",
        "widget/HHHHHHHHHHHH",
    ] {
        assert_eq!(
            IconRef::parse(bad),
            None,
            "{bad:?} is not an address of ours"
        );
    }
}

#[test]
fn the_widget_is_not_trimmed_to_its_drawing() {
    // The paper's margin is where the marks are placed: trimming it would move every one of
    // them, and the offsets are the game's own.
    assert!(!IconRef::Widget {
        fills: fills("hn----------")
    }
    .trims_to_drawing());
}

#[test]
fn the_unknown_sprite_is_a_reference_like_any_other() {
    // It has to survive the round trip for the same reason every other one does: the URL is
    // written on one side of the boundary and read on the other, and a reference the handler
    // refuses is a picture that never appears — which here would mean the app falls silently
    // back to the hole this reference exists to fill (B69).
    let r = IconRef::Unknown;
    assert_eq!(r.to_path(), "unknown");
    assert_eq!(IconRef::parse("unknown"), Some(IconRef::Unknown));
    assert_eq!(IconRef::parse("unknown/please"), None);
}

#[test]
fn the_unknown_sprite_is_served_whole_like_the_items_it_stands_for() {
    // It sits in the same box as an item's icon, at the same size, and is fitted to it the
    // same way: trimming it would make it bigger than the pictures around it.
    assert!(!IconRef::Unknown.trims_to_drawing());
}

#[test]
fn the_unknown_sprite_is_the_question_mark_the_game_draws_for_a_hidden_item() {
    // The file, not a picture of ours: measured 2026-09-22 in `graphics.a`, 3047 bytes. It
    // is what Curse of the Blind puts on a pedestal, so the app says "unknown" in the word
    // its user already reads.
    let s = ipc::unknown_source();
    assert_eq!(s.path, "gfx/items/collectibles/questionmark.png");
    assert_eq!(s.rect, None, "the whole file: it is one picture");
}
