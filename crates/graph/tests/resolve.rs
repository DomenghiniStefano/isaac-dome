//! From a generated ref to a typed requirement, against a catalog built from tiny XML
//! fixtures. The one thing these tests are really about: resolution goes **by name**,
//! because the wiki's entity ids and our boss ids are different numbering spaces.

use catalog::{BossId, Catalog, CharacterId, ItemId, ItemKind};
use graph::model::Requirement;
use graph::resolve::requirement;
use graph::rules::{Corrections, RefRow, Requirements, Rules};
use wiki::Target;

const PLAYERS: &str = r#"<players root="gfx/" portraitroot="gfx/ui/stage/">
  <player id="0" name="Isaac" portrait="PlayerPortrait_Isaac.png" />
  <player id="19" name="Jacob &amp; Esau" portrait="PlayerPortrait_Jacob.png" achievement="7" />
</players>"#;

// Gish is gated by the game itself (`achievement=`, "beat the depths 20 times"); Satan
// isn't, and never could be — which is the difference the resolver turns on.
const BOSSES: &str = r#"<bosses root="gfx/ui/boss/">
  <boss id="19" name="Gish" portrait="Portrait_Gish.png" achievement="18" />
  <boss id="84" name="Satan" portrait="Portrait_Satan.png" />
</bosses>"#;

const ITEMS: &str = r#"<items gfxroot="gfx/">
  <active id="35" name="The Bible" gfx="b.png" />
</items>"#;

fn catalog() -> Catalog {
    Catalog::build(|p| match p {
        "players.xml" => Some(PLAYERS.as_bytes().to_vec()),
        "bossportraits.xml" => Some(BOSSES.as_bytes().to_vec()),
        "items.xml" => Some(ITEMS.as_bytes().to_vec()),
        _ => None,
    })
}

fn rules(corrections: &str) -> Rules {
    let r: Requirements = serde_json::from_str(
        r#"{"schemaVersion":1,
            "generatedFrom":{"snapshotAt":"","maxRevid":0},
            "achievements":{},"targets":[]}"#,
    )
    .expect("requirements parse");
    let c: Corrections = serde_json::from_str(corrections).expect("corrections parse");
    Rules::build(r, c).expect("rules build")
}

fn row(target: Target, label: &str) -> RefRow {
    RefRow {
        target,
        label: label.into(),
    }
}

fn entity(id: u32, label: &str) -> RefRow {
    row(
        Target::Entity {
            id,
            variant: 0,
            subtype: 0,
        },
        label,
    )
}

#[test]
fn an_entity_ref_resolves_to_a_boss_by_name_not_by_id() {
    let c = catalog();
    let rules = rules(r#"{"schemaVersion":1}"#);
    assert_eq!(
        requirement(&c, &rules, &entity(43, "Gish")),
        Requirement::Boss { id: BossId(19) },
        "entity 43 is boss 19: the id spaces differ, the name is the bridge"
    );
}

#[test]
fn a_boss_the_game_does_not_gate_is_judged_not_assumed() {
    // Satan resolves by name, and the game gates him with nothing. Stopping at
    // `Requirement::Boss` here would produce no edge and no unknown, and the node would
    // read as "nothing in the way" — which is how Delirium looked available.
    let c = catalog();
    let unjudged = rules(r#"{"schemaVersion":1}"#);
    assert_eq!(
        requirement(&c, &unjudged, &entity(84, "Satan")),
        Requirement::Unknown {
            label: "Satan".into()
        },
        "a boss with no unlocker and no verdict is unknown, never 'available'"
    );
    let judged =
        rules(r#"{"schemaVersion":1,"verdicts":{"entity:Satan":{"alwaysAvailable":true}}}"#);
    assert_eq!(
        requirement(&c, &judged, &entity(84, "Satan")),
        Requirement::None,
        "judged as fought on night one: it gates nothing, and that is a decision on record"
    );
}

#[test]
fn an_alias_is_applied_before_the_lookup() {
    let c = catalog();
    let rules = rules(r#"{"schemaVersion":1,"aliases":{"Jacob and Esau":"Jacob & Esau"}}"#);
    assert_eq!(
        requirement(
            &c,
            &rules,
            &row(Target::Character { id: 19 }, "Jacob and Esau")
        ),
        Requirement::Character {
            id: CharacterId(19)
        }
    );
}

#[test]
fn always_available_drops_the_requirement() {
    let c = catalog();
    let rules =
        rules(r#"{"schemaVersion":1,"verdicts":{"room:Boss Rush":{"alwaysAvailable":true}}}"#);
    assert_eq!(
        requirement(
            &c,
            &rules,
            &row(
                Target::Room {
                    name: "Boss Rush".into()
                },
                "Boss Rush"
            )
        ),
        Requirement::None,
        "content reachable on night one gates nothing"
    );
}

#[test]
fn a_target_with_no_verdict_stays_unknown() {
    let c = catalog();
    let rules = rules(r#"{"schemaVersion":1}"#);
    assert_eq!(
        requirement(
            &c,
            &rules,
            &row(
                Target::Stage {
                    name: "Nowhere".into()
                },
                "Nowhere"
            )
        ),
        Requirement::Unknown {
            label: "Nowhere".into()
        },
        "an uncurated target must never read as 'no prerequisite'"
    );
}

#[test]
fn a_verdict_of_unknown_behaves_like_no_verdict_but_was_judged() {
    let c = catalog();
    let rules = rules(
        r#"{"schemaVersion":1,"verdicts":{
             "transformation:Guppy":{"unknown":{"reason":"three items"}}}}"#,
    );
    assert_eq!(
        requirement(&c, &rules, &row(Target::Transformation { id: 0 }, "Guppy")),
        Requirement::Unknown {
            label: "Guppy".into()
        },
        "judged inexpressible and never judged land in the same place at runtime"
    );
}

#[test]
fn a_name_the_catalog_does_not_know_is_unknown_not_dropped() {
    let c = catalog();
    let rules = rules(r#"{"schemaVersion":1}"#);
    assert_eq!(
        requirement(&c, &rules, &entity(999, "Nobody")),
        Requirement::Unknown {
            label: "Nobody".into()
        }
    );
}

#[test]
fn an_entity_that_is_not_a_boss_takes_its_verdict() {
    let c = catalog();
    let rules =
        rules(r#"{"schemaVersion":1,"verdicts":{"entity:Red Heart":{"notAPrerequisite":true}}}"#);
    assert_eq!(
        requirement(&c, &rules, &entity(5, "Red Heart")),
        Requirement::None,
        "an entity the catalog has no boss for falls through to the verdict table"
    );
}

#[test]
fn an_item_resolves_across_the_collectible_kinds() {
    let c = catalog();
    let rules = rules(r#"{"schemaVersion":1}"#);
    assert_eq!(
        requirement(&c, &rules, &row(Target::Item { id: 35 }, "The Bible")),
        Requirement::Item {
            kind: ItemKind::Active,
            id: ItemId(35)
        },
        "the wiki has one collectible id space; ours is keyed by (kind, id)"
    );
}
