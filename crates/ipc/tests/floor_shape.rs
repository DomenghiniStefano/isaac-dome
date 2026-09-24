//! The wire shape. A missing `rename_all` does not fail a Rust test, it makes TypeScript read
//! `undefined` in silence — so the JSON is asserted here, key by key.

use ipc::{floor_view, RoomKindView};
use serde_json::Value;

fn empty_cells() -> Vec<Option<RoomKindView>> {
    vec![None; 169]
}

#[test]
fn an_empty_grid_says_so_instead_of_answering() {
    let v = floor_view(empty_cells());
    let json = serde_json::to_value(&v).expect("serializes");
    assert_eq!(json["painted"], Value::from(0));
    let kinds: Vec<&str> = json["diagnostics"]
        .as_array()
        .expect("an array")
        .iter()
        .filter_map(|d| d["kind"].as_str())
        .collect();
    assert!(kinds.contains(&"gridEmpty"));
}

#[test]
fn a_candidate_crosses_in_camel_case_and_carries_its_quotation() {
    // A plus of normal rooms around cell 84 leaves its four diagonals with two neighbours
    // each and makes the cell itself painted; the cell above the plus has three.
    let mut cells = empty_cells();
    for cell in [84usize, 83, 85, 71, 97] {
        cells[cell] = Some(RoomKindView::Normal);
    }
    cells[84] = Some(RoomKindView::Start);
    let v = floor_view(cells);
    let json = serde_json::to_value(&v).expect("serializes");
    let solution = &json["solutions"][0];
    assert!(
        solution["target"].is_string(),
        "a fieldless enum is a bare string"
    );
    let first = &solution["candidates"][0];
    assert!(first["cell"].is_number());
    assert!(first["neighbours"].is_number());
    assert!(first["rank"].is_number());
    let applied = &first["applied"][0];
    assert!(applied["quote"].as_str().is_some_and(|q| !q.is_empty()));
    assert!(applied["url"]
        .as_str()
        .is_some_and(|u| u.starts_with("https://")));
}

#[test]
fn a_grid_of_the_wrong_length_is_a_diagnostic_and_not_a_panic() {
    let v = floor_view(vec![None; 3]);
    let json = serde_json::to_value(&v).expect("serializes");
    assert_eq!(json["solutions"].as_array().map(Vec::len), Some(0));
}

#[test]
fn every_room_kind_of_the_domain_has_a_view() {
    // Adding a kind to `floor` has to break this, not silently drop a paintable room.
    let all = [
        RoomKindView::Start,
        RoomKindView::Normal,
        RoomKindView::Boss,
        RoomKindView::Treasure,
        RoomKindView::Shop,
        RoomKindView::Curse,
        RoomKindView::Challenge,
        RoomKindView::Sacrifice,
        RoomKindView::Arcade,
        RoomKindView::Library,
        RoomKindView::Miniboss,
        RoomKindView::Secret,
        RoomKindView::SuperSecret,
        RoomKindView::UltraSecret,
    ];
    assert_eq!(all.len(), 14);
    for k in all {
        let json = serde_json::to_value(k).expect("serializes");
        assert!(json.as_str().is_some(), "a fieldless enum is a bare string");
    }
}

/// Card #81, V3: the rules are embedded at build time, so the text of a `serde_json` error is a
/// developer's message and not a translatable one. The diagnostic carries nothing.
#[test]
fn unreadable_rules_cross_as_a_bare_tag() {
    assert_eq!(
        serde_json::to_value(ipc::FloorDiagnostic::RulesUnreadable).unwrap(),
        serde_json::json!({ "kind": "rulesUnreadable" })
    );
}
