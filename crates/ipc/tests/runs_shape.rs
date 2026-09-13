//! The shape of the runs on the wire. `camelCase`, tagged where a variant carries data, and
//! nothing in it that could name a folder on this machine.

use ipc::{runs_view, RunSource, RunsInputs};
use run::{Floor, Outcome, Run, SeedKind};

fn a_run(seed: &str, outcome: Outcome) -> Run {
    Run {
        seed_words: seed.to_string(),
        seed_numeric: 1,
        seed_kind: SeedKind::Net,
        character: Some("Judas".to_string()),
        starting_items: vec![34],
        collected: vec![105],
        passives: vec![],
        familiars: vec![],
        held_active: Some(105),
        floors: vec![Floor {
            stage: 1,
            stage_type: 0,
            seed: 7,
        }],
        achievements: vec![19],
        outcome,
    }
}

fn view_of(sources: Vec<(RunSource, Vec<Run>)>) -> ipc::RunsView {
    runs_view(RunsInputs {
        sources,
        catalog: None,
        diagnostics: vec![],
    })
}

#[test]
fn the_fields_are_camel_case_on_the_wire() {
    // The silent failure this rule exists for: without `rename_all`, TypeScript reads
    // `undefined` and nothing errors.
    let view = view_of(vec![(
        RunSource::Session {
            name: "09_12_2026__13_34_26".to_string(),
        },
        vec![a_run("AAAA AAAA", Outcome::Open)],
    )]);
    let json = serde_json::to_string(&view).unwrap();
    assert!(json.contains("\"seedWords\""), "{json}");
    assert!(json.contains("\"heldActive\""), "{json}");
    assert!(json.contains("\"startingItems\""), "{json}");
    assert!(!json.contains("seed_words"), "{json}");
}

#[test]
fn an_outcome_that_carries_something_is_tagged_and_its_fields_are_camel_case() {
    let view = view_of(vec![(
        RunSource::Live,
        vec![a_run(
            "AAAA AAAA",
            Outcome::Died {
                killer: "9.0".to_string(),
            },
        )],
    )]);
    let json = serde_json::to_string(&view).unwrap();
    assert!(json.contains("\"kind\":\"died\""), "{json}");
    assert!(json.contains("\"killer\":\"9.0\""), "{json}");
}

#[test]
fn a_session_is_named_by_its_folder_and_the_live_log_is_not_named_at_all() {
    // The IPC forbids a path. A session's folder name is a date; the live log has no name to
    // give, and inventing one would be inventing a path.
    let view = view_of(vec![
        (RunSource::Live, vec![a_run("AAAA AAAA", Outcome::Open)]),
        (
            RunSource::Session {
                name: "09_12_2026__13_34_26".to_string(),
            },
            vec![a_run("BBBB BBBB", Outcome::Abandoned)],
        ),
    ]);
    let json = serde_json::to_string(&view).unwrap();
    assert!(json.contains("\"kind\":\"live\""), "{json}");
    assert!(json.contains("09_12_2026__13_34_26"), "{json}");
    assert!(!json.contains('\\'), "no path separator anywhere: {json}");
    assert!(!json.contains(":/"), "no path anywhere: {json}");
}

#[test]
fn without_a_catalog_an_item_keeps_its_id_and_says_it_has_no_name() {
    // "The game is not installed" is a state to report, not a blank to paper over.
    let view = view_of(vec![(
        RunSource::Live,
        vec![a_run("AAAA AAAA", Outcome::Open)],
    )]);
    let item = &view.runs[0].starting_items[0];
    assert_eq!(item.id, 34);
    assert_eq!(item.name, None);
    assert!(view.diagnostics.contains(&ipc::RunsDiagnostic::NoCatalog));
}

#[test]
fn the_totals_count_each_outcome_once() {
    let view = view_of(vec![(
        RunSource::Live,
        vec![
            a_run(
                "AAAA AAAA",
                Outcome::Won {
                    ending: "Sheol".to_string(),
                },
            ),
            a_run(
                "BBBB BBBB",
                Outcome::Died {
                    killer: "9.0".to_string(),
                },
            ),
            a_run("CCCC CCCC", Outcome::Abandoned),
            a_run("DDDD DDDD", Outcome::Open),
            a_run(
                "EEEE EEEE",
                Outcome::Won {
                    ending: "Cathedral".to_string(),
                },
            ),
        ],
    )]);
    assert_eq!(view.totals.runs, 5);
    assert_eq!(view.totals.won, 2);
    assert_eq!(view.totals.died, 1);
    assert_eq!(view.totals.abandoned, 1);
    assert_eq!(view.totals.open, 1);
}

#[test]
fn the_ordinal_is_the_position_in_its_own_source() {
    // A run is `(source, ordinal)`: the second run of one session and the second of another are
    // both ordinal 1, and they are different runs.
    let view = view_of(vec![
        (
            RunSource::Session {
                name: "09_12_2026__13_34_26".to_string(),
            },
            vec![
                a_run("AAAA AAAA", Outcome::Open),
                a_run("BBBB BBBB", Outcome::Open),
            ],
        ),
        (
            RunSource::Session {
                name: "09_13_2026__14_48_32".to_string(),
            },
            vec![
                a_run("CCCC CCCC", Outcome::Open),
                a_run("DDDD DDDD", Outcome::Open),
            ],
        ),
    ]);
    let ordinals: Vec<u32> = view.runs.iter().map(|r| r.ordinal).collect();
    assert_eq!(ordinals, vec![0, 1, 0, 1]);
}

#[test]
fn the_game_calling_a_run_online_is_what_online_means() {
    // `Net` is the only free discriminator we have for co-op, and it comes from the seed line.
    let mut solo = a_run("AAAA AAAA", Outcome::Open);
    solo.seed_kind = SeedKind::New;
    let view = view_of(vec![(
        RunSource::Live,
        vec![solo, a_run("BBBB BBBB", Outcome::Open)],
    )]);
    assert!(!view.runs[0].online);
    assert!(view.runs[1].online);
}
