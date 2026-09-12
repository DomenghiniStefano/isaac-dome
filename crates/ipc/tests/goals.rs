use ipc::{Goal, GoalId, ItemKindView, TargetKey, UnlockTarget};

#[test]
fn goal_json_shape_is_pinned() {
    let g = Goal {
        id: GoalId::from_str_unchecked("g1"),
        target: TargetKey::Item {
            item_kind: ItemKindView::Passive,
            id: 555,
        },
        created_unix: 1_700_000_000,
        note: Some("stasera".into()),
    };
    let v = serde_json::to_value(&g).unwrap();
    assert_eq!(v["id"], "g1");
    assert_eq!(v["target"]["kind"], "item");
    // The field is called `itemKind` because `kind` is `TargetKey`'s tag; its value
    // is a bare string, because `ItemKindView` has no fields.
    assert_eq!(v["target"]["itemKind"], "passive");
    assert_eq!(v["target"]["id"], 555);
    assert_eq!(v["createdUnix"], 1_700_000_000);
    assert_eq!(v["note"], "stasera");
    let back: Goal = serde_json::from_value(v).unwrap();
    assert_eq!(back, g);
}

/// This JSON is `store`'s on-disk format: changing it invalidates the rows already saved.
#[test]
fn target_key_json_shape_is_pinned_it_is_the_on_disk_format() {
    let cases = [
        (
            TargetKey::Item {
                item_kind: ItemKindView::Trinket,
                id: 1,
            },
            serde_json::json!({ "kind": "item", "itemKind": "trinket", "id": 1 }),
        ),
        (
            TargetKey::Character { id: 21 },
            serde_json::json!({ "kind": "character", "id": 21 }),
        ),
        (
            TargetKey::Boss { id: 100 },
            serde_json::json!({ "kind": "boss", "id": 100 }),
        ),
        (
            TargetKey::Challenge { id: 44 },
            serde_json::json!({ "kind": "challenge", "id": 44 }),
        ),
    ];
    for (key, json) in cases {
        let v = serde_json::to_value(&key).unwrap();
        assert_eq!(v, json, "{key:?}");
        assert_eq!(
            serde_json::from_value::<TargetKey>(v).unwrap(),
            key,
            "and the saved row reads back"
        );
    }
    // No name, no icon: those are derived from the catalog and don't live in the file.
    let v = serde_json::to_value(TargetKey::Item {
        item_kind: ItemKindView::Passive,
        id: 1,
    })
    .unwrap();
    assert_eq!(v.as_object().unwrap().len(), 3, "kind, itemKind, id");
}

#[test]
fn every_target_variant_keeps_its_tag_as_a_view_too() {
    for (t, tag) in [
        (
            UnlockTarget::Character {
                id: 21,
                name: "T. Isaac".into(),
                tainted: true,
            },
            "character",
        ),
        (
            UnlockTarget::Boss {
                id: 100,
                name: "The Beast".into(),
            },
            "boss",
        ),
        (
            UnlockTarget::Challenge {
                id: 44,
                name: "Red Redemption".into(),
                rewards: vec![533],
            },
            "challenge",
        ),
    ] {
        let v = serde_json::to_value(&t).unwrap();
        assert_eq!(v["kind"], tag);
        if tag == "challenge" {
            // The reward travels as an achievement id: the frontend already has a node
            // for each one in UnlockView and looks it up by id.
            assert_eq!(v["rewards"], serde_json::json!([533]));
        }
        assert_eq!(serde_json::from_value::<UnlockTarget>(v).unwrap(), t);
    }
}

/// The view discards exactly what isn't saved, and nothing else: same variant, same id.
#[test]
fn the_key_of_a_view_keeps_the_identity_and_drops_name_and_icon() {
    let cases = [
        (
            UnlockTarget::Item {
                item_kind: ItemKindView::Familiar,
                id: 8,
                name: "Brother Bobby".into(),
                icon_url: Some("data:image/png;base64,AA==".into()),
            },
            TargetKey::Item {
                item_kind: ItemKindView::Familiar,
                id: 8,
            },
        ),
        (
            UnlockTarget::Character {
                id: 21,
                name: "T. Isaac".into(),
                tainted: true,
            },
            TargetKey::Character { id: 21 },
        ),
        (
            UnlockTarget::Boss {
                id: 100,
                name: "The Beast".into(),
            },
            TargetKey::Boss { id: 100 },
        ),
        (
            UnlockTarget::Challenge {
                id: 44,
                name: "Red Redemption".into(),
                rewards: vec![533],
            },
            TargetKey::Challenge { id: 44 },
        ),
    ];
    for (view, key) in cases {
        assert_eq!(view.key(), key, "{view:?}");
    }
}

#[test]
fn fresh_goal_ids_are_distinct_and_opaque() {
    let a = GoalId::new();
    let b = GoalId::new();
    assert_ne!(a, b);
    assert!(a.as_str().len() >= 16);
}

// --- target_exists ---------------------------------------------------------------

fn item(item_kind: ItemKindView, id: u32) -> TargetKey {
    TargetKey::Item { item_kind, id }
}

#[test]
fn on_an_empty_catalog_every_target_is_unknown() {
    let c = catalog::Catalog::build(|_| None);
    for t in [
        item(ItemKindView::Passive, 1),
        item(ItemKindView::Active, 33),
        item(ItemKindView::Familiar, 10),
        item(ItemKindView::Trinket, 1),
        TargetKey::Character { id: 0 },
        TargetKey::Boss { id: 1 },
        TargetKey::Challenge { id: 1 },
    ] {
        assert!(!ipc::target_exists(&c, &t), "{t:?}");
    }
}

/// Against the real catalog via `samples/packed`. Skips with a note if it's missing.
#[test]
fn on_the_real_catalog_known_ids_exist_and_absurd_ones_do_not() {
    let Some(packed) = test_support::packed_dir() else {
        return;
    };
    let rs = unpack::ResourceSet::open(&packed);
    let c = catalog::Catalog::build(|p| rs.read(p));
    // The Sad Onion, Brother Bobby, Cartridge, Umbilical Cord, Isaac, Monstro, Pitch Black:
    // every kind is exercised, so a wrong `item_kind` mapping would show up.
    for t in [
        item(ItemKindView::Passive, 1),
        item(ItemKindView::Familiar, 8),
        item(ItemKindView::Trinket, 8),
        item(ItemKindView::Trinket, 33),
        TargetKey::Character { id: 0 },
        TargetKey::Boss { id: 1 },
        TargetKey::Challenge { id: 1 },
    ] {
        assert!(ipc::target_exists(&c, &t), "{t:?}");
    }
    // The catalog's key is (kind, id): the same id with the wrong kind isn't an item.
    // Id 8 is a familiar and a trinket, never a passive; 33 is an active (The Bible)
    // and a trinket, never a passive.
    for t in [
        item(ItemKindView::Active, 1),
        item(ItemKindView::Passive, 8),
        item(ItemKindView::Active, 8),
        item(ItemKindView::Passive, 33),
    ] {
        assert!(!ipc::target_exists(&c, &t), "{t:?}");
    }
    for t in [
        item(ItemKindView::Passive, 999_999),
        TargetKey::Character { id: 999_999 },
        TargetKey::Boss { id: 999_999 },
        TargetKey::Challenge { id: 999_999 },
    ] {
        assert!(!ipc::target_exists(&c, &t), "{t:?}");
    }
}
