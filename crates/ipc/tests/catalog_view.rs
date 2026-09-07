use catalog::Catalog;
use ipc::{catalog_view, item_views, ItemKindView};

const ITEMS_WITH_STRING_KEYS: &[u8] = b"<items gfxroot=\"gfx/items/\">
<passive id=\"1\" gfx=\"a.png\" name=\"#A_NAME\" description=\"#A_D\" />
<active\tid=\"2\"\tgfx=\"b.png\" name=\"#B_NAME\" description=\"#B_D\" />
<trinket id=\"1\" gfx=\"t.png\" name=\"#T_NAME\" description=\"#T_D\" />
</items>";
// A_D, B_D and T_D (the descriptions) are here: without them, unresolvedNames would
// climb to 2, since it counts every UnresolvedKey (names and descriptions), not just
// names. T_NAME is deliberately left missing: it's the test's one unresolved key.
const TABLE: &[u8] = br#"<stringtable><languages><language id="21" index="0" name="Key"/><language id="0" index="1" name="English"/></languages>
<category name="Items"><key name="A_NAME"><string>Apple</string></key><key name="A_D"><string>A fruit</string></key>
<key name="B_NAME"><string>Bee</string></key><key name="B_D"><string>A bug</string></key>
<key name="T_D"><string>Shiny</string></key></category></stringtable>"#;

/// Test catalog for the name views: the items point into the stringtable, and
/// `T_NAME` is deliberately left without a string. Not the catalog from `graph.rs`,
/// which instead exists to link achievements and unlocks: two different fixtures,
/// two different names.
fn catalog_with_string_keys() -> Catalog {
    Catalog::build(|p| match p {
        "items.xml" => Some(ITEMS_WITH_STRING_KEYS.to_vec()),
        "stringtable.sta" => Some(TABLE.to_vec()),
        _ => None,
    })
}

#[test]
fn counts_total_and_unresolved_names_come_from_the_catalog() {
    let v = catalog_view(&catalog_with_string_keys());
    assert_eq!(
        (
            v.counts.passives,
            v.counts.actives,
            v.counts.familiars,
            v.counts.trinkets
        ),
        (1, 1, 0, 1)
    );
    assert_eq!(v.total, 3);
    assert_eq!(v.unresolved_names, 1, "T_NAME has no string");
    assert_eq!(v.languages, vec!["english"]);
}

#[test]
fn item_views_carry_english_names_and_only_the_sprites_that_exist() {
    let items = item_views(
        &catalog_with_string_keys(),
        |p| (p == "gfx/items/collectibles/a.png").then(|| vec![0x89, b'P', b'N', b'G']),
        10,
    );
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].name, "Apple");
    assert!(items[0]
        .data_url
        .as_deref()
        .unwrap()
        .starts_with("data:image/png;base64,"));
    assert_eq!(items[1].name, "Bee");
    assert_eq!(
        items[1].data_url, None,
        "sprite missing: the row stays, without an image"
    );
    assert_eq!(
        items[2].name, "T_NAME",
        "unresolved key: the key itself is shown"
    );
    assert_eq!(items[1].kind, ItemKindView::Active);
}

#[test]
fn limit_caps_the_sample() {
    assert_eq!(
        item_views(&catalog_with_string_keys(), |_| None, 2).len(),
        2
    );
}

#[test]
fn json_shape_is_pinned() {
    let items = item_views(&catalog_with_string_keys(), |_| None, 1);
    let v = serde_json::to_value(&items[0]).unwrap();
    // `ItemKindView` has no fields: on the wire it's a bare string, not a tagged object.
    assert_eq!(v["kind"], "passive");
    assert_eq!(v["dataUrl"], serde_json::Value::Null);
    let c = serde_json::to_value(catalog_view(&catalog_with_string_keys())).unwrap();
    assert_eq!(c["counts"]["passives"], 1);
    assert_eq!(c["unresolvedNames"], 1);
}
