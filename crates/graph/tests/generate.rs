//! One pass from the wiki dataset to the requirements file. The tests build tiny
//! datasets by hand: this is about what the walk keeps and what it puts in the
//! inventory, not about the real snapshot.

use graph::generate::generate;
use std::collections::BTreeMap;
use wiki::{Dataset, Entry, Infobox, Inline, Style, Target};

fn achievement(id: u32, requirements: Vec<Inline>) -> (u32, Entry) {
    (
        id,
        Entry {
            title: format!("A{id}"),
            revid: 1,
            infobox: Infobox::Achievement {
                description: String::new(),
                requirements,
                unlocks: None,
            },
            sections: Vec::new(),
        },
    )
}

fn dataset(entries: Vec<(u32, Entry)>) -> Dataset {
    let mut d = wiki::for_tests::empty_dataset();
    d.achievements = entries.into_iter().collect::<BTreeMap<_, _>>();
    d
}

fn text(t: &str) -> Inline {
    Inline::Text {
        text: t.into(),
        style: Style::Plain,
    }
}

#[test]
fn a_typed_ref_becomes_a_requirement_row() {
    let d = dataset(vec![achievement(
        7,
        vec![
            text("Defeat "),
            Inline::Ref {
                target: Target::Entity {
                    id: 84,
                    variant: 0,
                    subtype: 0,
                },
                label: "Satan".into(),
            },
        ],
    )]);
    let r = generate(&d);
    let refs = &r
        .achievements
        .get(&7)
        .expect("achievement 7 is present")
        .refs;
    assert_eq!(
        refs.len(),
        1,
        "the plain text around the ref is not a requirement"
    );
    assert_eq!(refs[0].label, "Satan");
}

#[test]
fn refs_nested_in_an_edition_block_are_not_lost() {
    let d = dataset(vec![achievement(
        8,
        vec![Inline::Edition {
            only: vec![wiki::Dlc::Repentance],
            inline: vec![Inline::Ref {
                target: Target::Character { id: 19 },
                label: "Jacob and Esau".into(),
            }],
        }],
    )]);
    let r = generate(&d);
    assert_eq!(
        r.achievements.get(&8).expect("present").refs.len(),
        1,
        "an Edition block wraps inline content: its refs still count"
    );
}

#[test]
fn only_unreducible_targets_reach_the_inventory() {
    let d = dataset(vec![
        achievement(
            1,
            vec![Inline::Ref {
                target: Target::Character { id: 2 },
                label: "Cain".into(),
            }],
        ),
        achievement(
            2,
            vec![Inline::Ref {
                target: Target::Stage {
                    name: "The Void".into(),
                },
                label: "The Void".into(),
            }],
        ),
        achievement(
            3,
            vec![Inline::Concept {
                page: "Hard mode".into(),
                label: "Hard mode".into(),
            }],
        ),
    ]);
    let r = generate(&d);
    let keys: Vec<&str> = r.targets.iter().map(|t| t.key.as_str()).collect();
    assert!(
        !keys.contains(&"character:Cain"),
        "a character reduces through the catalog: it needs no verdict"
    );
    assert!(keys.contains(&"stage:The Void"), "got {keys:?}");
    assert!(
        keys.contains(&"pickup:Hard mode"),
        "a concept has no id: {keys:?}"
    );
}

#[test]
fn the_inventory_counts_uses_and_is_deterministic() {
    let stage = |label: &str| Inline::Ref {
        target: Target::Stage { name: label.into() },
        label: label.into(),
    };
    let d = dataset(vec![
        achievement(1, vec![stage("Basement")]),
        achievement(2, vec![stage("Basement"), stage("The Void")]),
    ]);
    let r = generate(&d);
    let basement = r
        .targets
        .iter()
        .find(|t| t.key == "stage:Basement")
        .expect("present");
    assert_eq!(basement.uses, 2);
    let keys: Vec<&str> = r.targets.iter().map(|t| t.key.as_str()).collect();
    let mut sorted = keys.clone();
    sorted.sort_unstable();
    assert_eq!(
        keys, sorted,
        "the inventory is sorted: the file must not churn between runs"
    );
}

#[test]
fn the_snapshot_the_file_was_generated_from_travels_with_it() {
    let mut d = dataset(vec![achievement(1, Vec::new())]);
    d.meta.snapshot_at = "2026-09-04T17:33:31Z".into();
    d.meta.max_revid = 269057;
    let r = generate(&d);
    assert_eq!(r.generated_from.snapshot_at, "2026-09-04T17:33:31Z");
    assert_eq!(r.generated_from.max_revid, 269057);
}
