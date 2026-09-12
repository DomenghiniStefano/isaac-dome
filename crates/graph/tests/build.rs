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
    assert!(node.unknown.is_empty());
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
    assert!(
        node.unknown.is_empty(),
        "no edge is not the same as unknown"
    );
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
    assert_eq!(
        g.node(2).expect("node 2").unknown,
        vec!["Nowhere".to_string()]
    );
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

#[test]
fn a_node_is_never_its_own_prerequisite() {
    // Real case, 17 of them on the live catalog (the Tainted block, 474-489): the
    // achievement that unlocks Tainted Isaac lists Tainted Isaac among its requirements.
    // That is not a cycle to declare, it is an edge with no meaning — and left in, it
    // poisons every node downstream with "not knowable".
    let c = Catalog::build(|p| match p {
        "players.xml" => Some(
            br#"<players root="gfx/" portraitroot="gfx/ui/stage/">
                  <player id="1" name="Magdalene" portrait="m.png" achievement="1" />
                </players>"#
                .to_vec(),
        ),
        "achievements.xml" => Some(ACHIEVEMENTS.as_bytes().to_vec()),
        _ => None,
    });
    let g = Graph::build(
        &c,
        &rules(
            &one_ref(
                1,
                r#"{"target":{"kind":"character","id":1},"label":"Magdalene"}"#,
                "",
            ),
            r#"{"schemaVersion":1}"#,
        ),
    );
    let node = g.node(1).expect("node 1");
    assert!(
        node.prerequisites.is_empty(),
        "achievement 1 unlocks Magdalene and requires her: the edge is dropped, got {:?}",
        node.prerequisites
    );
    assert!(
        g.diagnostics()
            .iter()
            .any(|d| matches!(d, GraphDiagnostic::SelfPrerequisite { node: 1 })),
        "dropping an edge is a decision: it gets named, got {:?}",
        g.diagnostics()
    );
}

// --- the character a sentence names (spec 2026-09-12, §4.2) --------------------------

const TAINTED_PLAYERS: &str = r#"<players root="gfx/" portraitroot="gfx/ui/stage/">
  <player id="0" name="Isaac" portrait="PlayerPortrait_Isaac.png" />
  <player id="21" name="Isaac" portrait="PlayerPortrait_Isaac_b.png" achievement="2" />
</players>"#;

const TAINTED_ACHIEVEMENTS: &str = r#"<achievements gfxroot="gfx/ui/achievement">
  <achievement id="1" name="One" text="One" gfx="1.png" />
  <achievement id="2" name="Two" text="Two" gfx="2.png" />
</achievements>"#;

/// The game gives a Tainted character the **same name** as its base form and tells them
/// apart by a flag, so a lookup by name alone cannot find "Tainted Isaac" — the key does
/// not exist, and one of the two "Isaac" rows wins the index outright.
///
/// That must not quietly become a tally. A reference naming a character asks about that
/// character's cell; answering "has anyone ever beaten Mother" instead is a different,
/// weaker question, and it would have been wrong for 141 of the 396 character references
/// in the shipped rules.
#[test]
fn a_tainted_character_is_found_by_id_when_its_name_is_shared() {
    let c = Catalog::build(|p| match p {
        "players.xml" => Some(TAINTED_PLAYERS.as_bytes().to_vec()),
        "achievements.xml" => Some(TAINTED_ACHIEVEMENTS.as_bytes().to_vec()),
        _ => None,
    });
    let requirements = r#"{"schemaVersion":1,
        "generatedFrom":{"snapshotAt":"","maxRevid":0},
        "achievements":{"1":{"refs":[
            {"target":{"kind":"character","id":21},"label":"Tainted Isaac"},
            {"target":{"kind":"entity","id":0,"variant":0,"subtype":0},"label":"Mother"}
        ]}},
        "targets":[]}"#;
    let corrections = r#"{"schemaVersion":1,"verdicts":{"entity:Mother":{"progress":{
        "mark":{"column":"mother","level":"base"},
        "counter":{"name":"motherKills","atLeast":1}}}}}"#;
    let g = Graph::build(&c, &rules(requirements, corrections));
    let node = g.node(1).expect("node 1");
    assert!(
        node.requirements
            .contains(&graph::model::Requirement::Mark {
                character: catalog::CharacterId(21),
                column: graph::rules::MarkColumn::Mother,
                level: graph::rules::MarkLevel::Base,
            }),
        "the sentence names Tainted Isaac, so it asks about his cell; got {:?}",
        node.requirements
    );
}

/// The mirror of the test above, and the harder half: a **base** character's name resolves
/// fine — to whichever of the two forms won the name index. Asked for Isaac, it can hand
/// back Tainted Isaac, and a mark requirement built on that points at a different row of
/// the completion matrix, with nothing to show anything went wrong.
///
/// Measured 2026-09-12: name-first, "Ultra Greedier as Keeper" picked row 29, which is
/// T. Keeper. The wiki's id is the only thing that separates the two, so it goes first.
#[test]
fn a_base_character_is_found_by_id_even_though_its_name_also_resolves() {
    let c = Catalog::build(|p| match p {
        "players.xml" => Some(TAINTED_PLAYERS.as_bytes().to_vec()),
        "achievements.xml" => Some(TAINTED_ACHIEVEMENTS.as_bytes().to_vec()),
        _ => None,
    });
    // Character 0 is base Isaac; 21 is Tainted Isaac, and both are named "Isaac".
    let requirements = r#"{"schemaVersion":1,
        "generatedFrom":{"snapshotAt":"","maxRevid":0},
        "achievements":{"1":{"refs":[
            {"target":{"kind":"character","id":0},"label":"Isaac"},
            {"target":{"kind":"entity","id":0,"variant":0,"subtype":0},"label":"Mother"}
        ]}},
        "targets":[]}"#;
    let corrections = r#"{"schemaVersion":1,"verdicts":{"entity:Mother":{"progress":{
        "mark":{"column":"mother","level":"base"}}}}}"#;
    let g = Graph::build(&c, &rules(requirements, corrections));
    let node = g.node(1).expect("node 1");
    assert!(
        node.requirements
            .contains(&graph::model::Requirement::Mark {
                character: catalog::CharacterId(0),
                column: graph::rules::MarkColumn::Mother,
                level: graph::rules::MarkLevel::Base,
            }),
        "the sentence names character 0, not whichever \"Isaac\" the index kept; got {:?}",
        node.requirements
    );
}
