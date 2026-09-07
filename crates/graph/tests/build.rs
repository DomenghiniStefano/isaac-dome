//! Requirements become edges — and the edge is read from the game's own `unlocked_by`
//! links, never from the wiki. The wiki only says *what* is needed.

use catalog::Catalog;
use graph::build::{Graph, GraphDiagnostic};
use graph::rules::{Corrections, Requirements, Rules};

const PLAYERS: &str = r#"<players root="gfx/" portraitroot="gfx/ui/stage/">
  <player id="0" name="Isaac" portrait="PlayerPortrait_Isaac.png" />
  <player id="1" name="Magdalene" portrait="PlayerPortrait_Magdalene.png" achievement="1" />
</players>"#;

const ACHIEVEMENTS: &str = r#"<achievements gfxroot="gfx/ui/achievement">
  <achievement id="1" name="Magdalene" text="Magdalene" gfx="1.png" />
  <achievement id="2" name="Second" text="Second" gfx="2.png" />
</achievements>"#;

fn catalog() -> Catalog {
    Catalog::build(|p| match p {
        "players.xml" => Some(PLAYERS.as_bytes().to_vec()),
        "achievements.xml" => Some(ACHIEVEMENTS.as_bytes().to_vec()),
        _ => None,
    })
}

fn rules(requirements: &str, corrections: &str) -> Rules {
    let r: Requirements = serde_json::from_str(requirements).expect("requirements parse");
    let c: Corrections = serde_json::from_str(corrections).expect("corrections parse");
    Rules::build(r, c).expect("rules build")
}

/// A requirements file with one achievement carrying one ref.
fn one_ref(achievement: u32, target: &str, targets: &str) -> String {
    format!(
        r#"{{"schemaVersion":1,
            "generatedFrom":{{"snapshotAt":"","maxRevid":0}},
            "achievements":{{"{achievement}":{{"refs":[{target}]}}}},
            "targets":[{targets}]}}"#
    )
}

#[test]
fn a_character_requirement_becomes_an_edge_to_its_unlocking_achievement() {
    let c = catalog();
    let g = Graph::build(
        &c,
        &rules(
            &one_ref(
                2,
                r#"{"target":{"kind":"character","id":1},"label":"Magdalene"}"#,
                "",
            ),
            r#"{"schemaVersion":1}"#,
        ),
    );
    let node = g.node(2).expect("node 2");
    assert_eq!(
        node.prerequisites,
        vec![1],
        "Magdalene is unlocked by achievement 1: that is the edge, and it comes from the game"
    );
    assert_eq!(node.unknown, 0);
}

#[test]
fn content_available_from_the_start_produces_no_edge() {
    let c = catalog();
    let g = Graph::build(
        &c,
        &rules(
            &one_ref(
                2,
                r#"{"target":{"kind":"character","id":0},"label":"Isaac"}"#,
                "",
            ),
            r#"{"schemaVersion":1}"#,
        ),
    );
    let node = g.node(2).expect("node 2");
    assert!(
        node.prerequisites.is_empty(),
        "Isaac has no unlocked_by: no edge, and that is a fact, not a gap"
    );
    assert_eq!(node.unknown, 0, "no edge is not the same as unknown");
}

#[test]
fn an_uncurated_target_counts_as_unknown_on_its_node() {
    let c = catalog();
    let g = Graph::build(
        &c,
        &rules(
            &one_ref(
                2,
                r#"{"target":{"kind":"stage","name":"Nowhere"},"label":"Nowhere"}"#,
                r#"{"key":"stage:Nowhere","label":"Nowhere","uses":1,"verdictRequired":true}"#,
            ),
            r#"{"schemaVersion":1}"#,
        ),
    );
    assert_eq!(g.node(2).expect("node 2").unknown, 1);
}

#[test]
fn a_gate_edge_comes_from_the_verdict() {
    let c = catalog();
    let g = Graph::build(
        &c,
        &rules(
            &one_ref(
                2,
                r#"{"target":{"kind":"stage","name":"The Void"},"label":"The Void"}"#,
                r#"{"key":"stage:The Void","label":"The Void","uses":1,"verdictRequired":true}"#,
            ),
            r#"{"schemaVersion":1,
                "verdicts":{"stage:The Void":{"behind":{"achievement":1}}}}"#,
        ),
    );
    assert_eq!(g.node(2).expect("node 2").prerequisites, vec![1]);
}

#[test]
fn a_ref_straight_to_an_achievement_is_an_edge_on_its_own() {
    let c = catalog();
    let g = Graph::build(
        &c,
        &rules(
            &one_ref(
                2,
                r#"{"target":{"kind":"achievement","id":1},"label":"Magdalene"}"#,
                "",
            ),
            r#"{"schemaVersion":1}"#,
        ),
    );
    assert_eq!(
        g.node(2).expect("node 2").prerequisites,
        vec![1],
        "an achievement named directly needs no verdict: it is already a node"
    );
}

#[test]
fn a_verdict_pointing_at_an_achievement_that_does_not_exist_is_diagnosed() {
    let c = catalog();
    let g = Graph::build(
        &c,
        &rules(
            &one_ref(
                2,
                r#"{"target":{"kind":"stage","name":"The Void"},"label":"The Void"}"#,
                r#"{"key":"stage:The Void","label":"The Void","uses":1,"verdictRequired":true}"#,
            ),
            r#"{"schemaVersion":1,
                "verdicts":{"stage:The Void":{"behind":{"achievement":999}}}}"#,
        ),
    );
    assert!(
        g.diagnostics().iter().any(|d| matches!(
            d,
            GraphDiagnostic::EdgeOutsideCatalog {
                achievement: 999,
                ..
            }
        )),
        "an edge to an id the catalog doesn't have must be named, got {:?}",
        g.diagnostics()
    );
    assert!(
        g.node(2).expect("node 2").prerequisites.is_empty(),
        "a dangling edge is dropped, not followed"
    );
}

#[test]
fn the_same_prerequisite_named_twice_is_one_edge() {
    let c = catalog();
    let g = Graph::build(
        &c,
        &rules(
            &one_ref(
                2,
                r#"{"target":{"kind":"character","id":1},"label":"Magdalene"},
                   {"target":{"kind":"achievement","id":1},"label":"Magdalene"}"#,
                "",
            ),
            r#"{"schemaVersion":1}"#,
        ),
    );
    assert_eq!(
        g.node(2).expect("node 2").prerequisites,
        vec![1],
        "prerequisites are a set: 'blocked by 2' would be counting the same run twice"
    );
}

#[test]
fn every_achievement_in_the_catalog_is_a_node_even_with_no_requirements() {
    let c = catalog();
    let g = Graph::build(&c, &rules(&one_ref(2, "", ""), r#"{"schemaVersion":1}"#));
    assert_eq!(
        g.nodes().len(),
        2,
        "the graph covers the catalog, not just what the wiki wrote about"
    );
    assert!(g.node(1).is_some());
}
