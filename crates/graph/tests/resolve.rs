//! From a generated ref to a typed requirement, against a catalog built from tiny XML
//! fixtures. The one thing these tests are really about: resolution goes **by name**,
//! because the wiki's entity ids and our boss ids are different numbering spaces.

use catalog::{BossId, Catalog, CharacterId, ItemId, ItemKind};
use graph::model::{Requirement, ThresholdItem};
use graph::resolve::requirement;
use graph::rules::{Corrections, CounterName, MarkColumn, MarkLevel, RefRow, Requirements, Rules};
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
        r#"{"schemaVersion":2,
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
    let rules = rules(r#"{"schemaVersion":2}"#);
    assert_eq!(
        requirement(&c, &rules, &entity(43, "Gish"), None),
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
    let unjudged = rules(r#"{"schemaVersion":2}"#);
    assert_eq!(
        requirement(&c, &unjudged, &entity(84, "Satan"), None),
        Requirement::Unknown {
            label: "Satan".into()
        },
        "a boss with no unlocker and no verdict is unknown, never 'available'"
    );
    let judged =
        rules(r#"{"schemaVersion":2,"verdicts":{"entity:Satan":{"alwaysAvailable":true}}}"#);
    assert_eq!(
        requirement(&c, &judged, &entity(84, "Satan"), None),
        Requirement::None,
        "judged as fought on night one: it gates nothing, and that is a decision on record"
    );
}

#[test]
fn an_alias_is_applied_before_the_lookup() {
    let c = catalog();
    let rules = rules(r#"{"schemaVersion":2,"aliases":{"Jacob and Esau":"Jacob & Esau"}}"#);
    assert_eq!(
        requirement(
            &c,
            &rules,
            &row(Target::Character { id: 19 }, "Jacob and Esau"),
            None
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
        rules(r#"{"schemaVersion":2,"verdicts":{"room:Boss Rush":{"alwaysAvailable":true}}}"#);
    assert_eq!(
        requirement(
            &c,
            &rules,
            &row(
                Target::Room {
                    name: "Boss Rush".into()
                },
                "Boss Rush"
            ),
            None
        ),
        Requirement::None,
        "content reachable on night one gates nothing"
    );
}

#[test]
fn a_target_with_no_verdict_stays_unknown() {
    let c = catalog();
    let rules = rules(r#"{"schemaVersion":2}"#);
    assert_eq!(
        requirement(
            &c,
            &rules,
            &row(
                Target::Stage {
                    name: "Nowhere".into()
                },
                "Nowhere"
            ),
            None
        ),
        Requirement::Unknown {
            label: "Nowhere".into()
        },
        "an uncurated target must never read as 'no prerequisite'"
    );
}

/// A stage and not a transformation: a transformation used to reach the verdict table and
/// no longer does — it resolves through `transformations` in the rules file — so testing
/// this on one would have quietly stopped testing the verdict.
#[test]
fn a_verdict_of_unknown_behaves_like_no_verdict_but_was_judged() {
    let c = catalog();
    let rules = rules(
        r#"{"schemaVersion":2,"verdicts":{
             "stage:Home":{"unknown":{"reason":"all endings"}}}}"#,
    );
    assert_eq!(
        requirement(
            &c,
            &rules,
            &row(
                Target::Stage {
                    name: "Home".into()
                },
                "Home"
            ),
            None
        ),
        Requirement::Unknown {
            label: "Home".into()
        },
        "judged inexpressible and never judged land in the same place at runtime"
    );
}

/// The rules file with one transformation in it. `items` are the wiki's targets, exactly as
/// the generator writes them: this file knows no id of ours.
fn rules_with_transformation(at_least: &str, items: &str) -> Rules {
    let r: Requirements = serde_json::from_str(&format!(
        r#"{{"schemaVersion":2,
            "generatedFrom":{{"snapshotAt":"","maxRevid":0}},
            "achievements":{{}},"targets":[],
            "transformations":{{"0":{{"label":"Guppy","atLeast":{at_least},"items":{items}}}}}}}"#
    ))
    .expect("requirements parse");
    let c: Corrections = serde_json::from_str(r#"{"schemaVersion":2}"#).expect("corrections parse");
    Rules::build(r, c).expect("rules build")
}

const BIBLE: &str = r#"{"kind":"item","id":35}"#;
const UNKNOWN_ITEM: &str = r#"{"kind":"item","id":999}"#;

#[test]
fn a_transformation_resolves_to_a_threshold_over_the_items_the_catalog_has() {
    let c = catalog();
    let rules = rules_with_transformation("2", &format!("[{BIBLE},{BIBLE}]"));
    let Requirement::Threshold {
        at_least,
        of,
        unresolved,
        label,
        ..
    } = requirement(
        &c,
        &rules,
        &row(Target::Transformation { id: 0 }, "Guppy"),
        None,
    )
    else {
        panic!("expected a threshold")
    };
    assert_eq!(label, "Guppy");
    assert_eq!(at_least, 2);
    assert_eq!(
        of,
        vec![
            ThresholdItem {
                kind: ItemKind::Active,
                id: ItemId(35),
                unlocked_by: None
            };
            2
        ]
    );
    assert_eq!(unresolved, 0);
}

/// A count the wiki did not state is not a threshold of zero, which everything meets.
#[test]
fn a_transformation_without_a_count_is_unknown_not_a_threshold_of_zero() {
    let c = catalog();
    let rules = rules_with_transformation("null", &format!("[{BIBLE}]"));
    assert!(matches!(
        requirement(
            &c,
            &rules,
            &row(Target::Transformation { id: 0 }, "Guppy"),
            None
        ),
        Requirement::Unknown { .. }
    ));
}

/// Stompy's shape, measured on the real snapshot: two collectibles and a count of three,
/// because its third contributor is a pill and a pill is not something this model carries.
/// A threshold nothing can ever meet would report "you are one item away" forever, so the
/// crate that has to answer declines to.
#[test]
fn a_set_smaller_than_its_count_is_unknown_and_not_an_unreachable_threshold() {
    let c = catalog();
    let rules = rules_with_transformation("3", &format!("[{BIBLE},{BIBLE}]"));
    assert!(matches!(
        requirement(
            &c,
            &rules,
            &row(Target::Transformation { id: 0 }, "Guppy"),
            None
        ),
        Requirement::Unknown { .. }
    ));
}

/// An item the catalog does not have is counted, not dropped: evaluation needs to know the
/// tally it holds is incomplete, because an unresolved item can only ever add to it.
#[test]
fn contributors_outside_the_catalog_are_counted_not_dropped() {
    let c = catalog();
    let rules = rules_with_transformation("2", &format!("[{BIBLE},{BIBLE},{UNKNOWN_ITEM}]"));
    let Requirement::Threshold { of, unresolved, .. } = requirement(
        &c,
        &rules,
        &row(Target::Transformation { id: 0 }, "Guppy"),
        None,
    ) else {
        panic!("expected a threshold")
    };
    assert_eq!(of.len(), 2);
    assert_eq!(unresolved, 1);
}

/// A transformation the rules file has never heard of is unknown, and is **not** the same
/// as one whose count could not be read: only the second is in the file at all.
#[test]
fn a_transformation_absent_from_the_rules_is_unknown() {
    let c = catalog();
    let rules = rules(r#"{"schemaVersion":2}"#);
    assert!(matches!(
        requirement(
            &c,
            &rules,
            &row(Target::Transformation { id: 0 }, "Guppy"),
            None
        ),
        Requirement::Unknown { .. }
    ));
}

#[test]
fn a_name_the_catalog_does_not_know_is_unknown_not_dropped() {
    let c = catalog();
    let rules = rules(r#"{"schemaVersion":2}"#);
    assert_eq!(
        requirement(&c, &rules, &entity(999, "Nobody"), None),
        Requirement::Unknown {
            label: "Nobody".into()
        }
    );
}

#[test]
fn an_entity_that_is_not_a_boss_takes_its_verdict() {
    let c = catalog();
    let rules =
        rules(r#"{"schemaVersion":2,"verdicts":{"entity:Red Heart":{"notAPrerequisite":true}}}"#);
    assert_eq!(
        requirement(&c, &rules, &entity(5, "Red Heart"), None),
        Requirement::None,
        "an entity the catalog has no boss for falls through to the verdict table"
    );
}

#[test]
fn an_item_resolves_across_the_collectible_kinds() {
    let c = catalog();
    let rules = rules(r#"{"schemaVersion":2}"#);
    assert_eq!(
        requirement(&c, &rules, &row(Target::Item { id: 35 }, "The Bible"), None),
        Requirement::Item {
            kind: ItemKind::Active,
            id: ItemId(35)
        },
        "the wiki has one collectible id space; ours is keyed by (kind, id)"
    );
}

// --- answered by the profile (spec 2026-09-12, §4.2) ---------------------------------

const PROGRESS: &str = r#"{
  "schemaVersion": 2,
  "verdicts": {
    "entity:Hush": { "progress": {
      "mark": { "column": "hush", "level": "base" },
      "counter": { "name": "hushKills", "atLeast": 1 }
    } },
    "entity:Ultra Greedier": { "progress": {
      "mark": { "column": "greed", "level": "second" }
    } }
  }
}"#;

/// Which half answers is decided by the **reference**, not by the target: a sentence that
/// names a character is asking about that character's cell.
#[test]
fn a_boss_the_reference_pairs_with_a_character_resolves_to_a_mark() {
    let c = catalog();
    let rules = rules(PROGRESS);
    assert_eq!(
        requirement(
            &c,
            &rules,
            &entity(0, "Ultra Greedier"),
            Some(CharacterId(0))
        ),
        Requirement::Mark {
            character: CharacterId(0),
            column: MarkColumn::Greed,
            level: MarkLevel::Second,
        }
    );
}

/// The boss alone asks a different question — "ever, by anyone" — and a tally answers it.
#[test]
fn a_boss_named_alone_resolves_to_a_tally() {
    let c = catalog();
    let rules = rules(PROGRESS);
    assert_eq!(
        requirement(&c, &rules, &entity(0, "Hush"), None),
        Requirement::Counter {
            name: CounterName::HushKills,
            at_least: 1,
        }
    );
}

/// Ultra Greedier has no located tally. Asked the question its curation cannot answer, the
/// resolver says so and keeps the label — it does not fall back to the other half, which
/// would answer a question nobody asked.
#[test]
fn the_half_a_reference_needs_may_be_absent_and_then_it_is_unknown() {
    let c = catalog();
    let rules = rules(PROGRESS);
    assert_eq!(
        requirement(&c, &rules, &entity(0, "Ultra Greedier"), None),
        Requirement::Unknown {
            label: "Ultra Greedier".to_string()
        }
    );
}

/// A boss the game itself gates by an achievement still short-circuits the verdict table:
/// adding `Progress` must not change the order the resolver asks its questions in.
#[test]
fn a_game_gated_boss_still_wins_over_a_progress_verdict() {
    let c = catalog();
    let rules = rules(
        r#"{"schemaVersion":2,"verdicts":{"entity:Gish":{"progress":{
             "counter":{"name":"hushKills","atLeast":1}}}}}"#,
    );
    assert_eq!(
        requirement(&c, &rules, &entity(43, "Gish"), None),
        Requirement::Boss { id: BossId(19) },
        "the game's own unlocked_by is stronger evidence than our curation"
    );
}

/// Card #80, item 04: passives and trinkets have separate id spaces, so 46 is two different
/// things. A reference says which space it means, and resolution stays in it: a trinket is
/// only ever a trinket, an item only one of the three collectible kinds.
const SHARED_ID: &str = r#"<items gfxroot="gfx/">
  <passive id="46" name="Lucky Foot" gfx="a.png" />
  <trinket id="46" name="Polished Bone" gfx="t.png" />
  <trinket id="47" name="Only A Trinket" gfx="u.png" />
</items>"#;

fn catalog_sharing_an_id() -> Catalog {
    Catalog::build(|p| match p {
        "items.xml" => Some(SHARED_ID.as_bytes().to_vec()),
        _ => None,
    })
}

#[test]
fn a_trinket_resolves_to_the_trinket_and_an_item_to_the_collectible() {
    let c = catalog_sharing_an_id();
    let r = rules(r#"{"schemaVersion":2}"#);
    // A label no name index knows, so the id is what decides.
    assert_eq!(
        requirement(
            &c,
            &r,
            &row(Target::Trinket { id: 46 }, "no such name"),
            None
        ),
        Requirement::Item {
            kind: ItemKind::Trinket,
            id: ItemId(46)
        }
    );
    assert_eq!(
        requirement(&c, &r, &row(Target::Item { id: 46 }, "no such name"), None),
        Requirement::Item {
            kind: ItemKind::Passive,
            id: ItemId(46)
        }
    );
}

#[test]
fn an_item_id_that_only_a_trinket_has_is_not_an_item() {
    let c = catalog_sharing_an_id();
    let r = rules(r#"{"schemaVersion":2}"#);
    assert!(
        !matches!(
            requirement(&c, &r, &row(Target::Item { id: 47 }, "no such name"), None),
            Requirement::Item { .. }
        ),
        "item 47 does not exist; trinket 47 is another thing"
    );
}

#[test]
fn a_transformation_contributor_stays_in_its_own_id_space() {
    let c = catalog_sharing_an_id();
    let rules = rules_with_transformation(
        "1",
        r#"[{"kind":"trinket","id":46},{"kind":"item","id":46},{"kind":"item","id":47}]"#,
    );
    let Requirement::Threshold { of, unresolved, .. } = requirement(
        &c,
        &rules,
        &row(Target::Transformation { id: 0 }, "Guppy"),
        None,
    ) else {
        panic!("expected a threshold")
    };
    let kinds: Vec<(ItemKind, u32)> = of.iter().map(|i| (i.kind, i.id.0)).collect();
    assert_eq!(
        kinds,
        vec![(ItemKind::Trinket, 46), (ItemKind::Passive, 46)]
    );
    assert_eq!(
        unresolved, 1,
        "item 47 is not a collectible, and is counted as unresolved"
    );
}

/// The game names a Tainted character exactly like its base form — `players.xml` carries
/// `id="0" name="Isaac"` and `id="21" name="Isaac"` with a `_b` portrait — so the name index
/// keeps one of the two, the later one. A reference to character 0 labelled "Isaac" is base
/// Isaac, and only the wiki's id says so: resolved by name, it became Tainted Isaac and drew
/// the edge to the achievement that unlocks *him* (card #82, review F1).
#[test]
fn a_base_character_reference_is_resolved_by_id_not_by_the_shared_name() {
    let c = Catalog::build(|p| {
        match p {
        "players.xml" => Some(
            br#"<players root="gfx/" portraitroot="gfx/ui/stage/">
                  <player id="0" name="Isaac" portrait="PlayerPortrait_Isaac.png" />
                  <player id="21" name="Isaac" portrait="PlayerPortrait_Isaac_b.png" achievement="474" />
                </players>"#
                .to_vec(),
        ),
        _ => None,
    }
    });
    let rules = rules(r#"{"schemaVersion":2}"#);
    assert_eq!(
        requirement(&c, &rules, &row(Target::Character { id: 0 }, "Isaac"), None),
        Requirement::Character { id: CharacterId(0) },
        "character 0 is base Isaac, whatever the name index kept for \"Isaac\""
    );
}
