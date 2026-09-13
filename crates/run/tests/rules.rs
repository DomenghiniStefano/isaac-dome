//! The rules turn a line into an event and make no judgment. Every input here is a real line,
//! copied with its `[INFO] - ` prefix: the prefix is why no pattern is anchored.

use run::{Event, Rules, SeedKind};

#[test]
fn the_seed_line_starts_a_run() {
    let rules = Rules::embedded();
    let line = "[INFO] - RNG Start Seed: YKF6 QDN6 (2913253616) [New, 1]";
    assert_eq!(
        rules.event(line),
        Some(Event::RunStarted {
            seed_words: "YKF6 QDN6".into(),
            seed_numeric: 2_913_253_616,
            kind: SeedKind::New,
        })
    );
}

#[test]
fn the_three_seed_kinds_the_logs_actually_contain() {
    // Measured 2026-09-13 across the four logs in `samples/logs/`: `New`, `Continue` and
    // `Net`. `Continue` is a run resumed from an earlier launch and `Net` is an online one —
    // both matter to the fold, which is why the kind is typed rather than kept as text.
    let rules = Rules::embedded();
    let kind_of = |line: &str| match rules.event(line) {
        Some(Event::RunStarted { kind, .. }) => Some(kind),
        _ => None,
    };
    assert_eq!(
        kind_of("[INFO] - RNG Start Seed: YKF6 QDN6 (2913253616) [New, 1]"),
        Some(SeedKind::New)
    );
    assert_eq!(
        kind_of("[INFO] - RNG Start Seed: 2ZTM KF7E (3398290282) [Continue, 1]"),
        Some(SeedKind::Continue)
    );
    assert_eq!(
        kind_of("[INFO] - RNG Start Seed: YJMB 3YT2 (2918802223) [Net, 1]"),
        Some(SeedKind::Net)
    );
}

#[test]
fn an_unseen_seed_kind_is_kept_and_not_dropped() {
    // A word we have never met must not cost the run that follows it: the run is real either
    // way, and only the reading of the label is uncertain.
    let rules = Rules::embedded();
    assert_eq!(
        rules.event("[INFO] - RNG Start Seed: AAAA BBBB (1) [Daily, 1]"),
        Some(Event::RunStarted {
            seed_words: "AAAA BBBB".into(),
            seed_numeric: 1,
            kind: SeedKind::Unknown,
        })
    );
}

#[test]
fn a_frame_marker_between_the_prefix_and_the_line_changes_nothing() {
    let rules = Rules::embedded();
    let line = "[INFO] - [Frame 74] Starting room transition (type 0) ";
    assert_eq!(rules.event(line), Some(Event::RoomTransition));
}

#[test]
fn the_collectible_line_carries_everything_the_fold_needs() {
    let rules = Rules::embedded();
    let line = "[INFO] - Adding collectible 34 (The Book of Belial) to player 0 (Judas) from pool treasure";
    assert_eq!(
        rules.event(line),
        Some(Event::ItemAdded {
            id: 34,
            name: "The Book of Belial".into(),
            player: 0,
            character: "Judas".into(),
            pool: "treasure".into(),
        })
    );
}

#[test]
fn the_floor_line_carries_the_stage_and_its_type() {
    let rules = Rules::embedded();
    assert_eq!(
        rules.event("[INFO] - Level::Init m_Stage 2, m_StageType 1 Seed 408474304"),
        Some(Event::FloorEntered {
            stage: 2,
            stage_type: 1,
            seed: 408_474_304,
        })
    );
}

#[test]
fn a_room_with_no_name_is_still_a_room() {
    let rules = Rules::embedded();
    assert_eq!(
        rules.event("[INFO] - Room 4.21()"),
        Some(Event::RoomEntered {
            id: "4.21".into(),
            name: String::new(),
        })
    );
}

#[test]
fn the_death_line_names_the_killer_and_what_spawned_it() {
    // No log in `samples/logs/` contains this line — see `logs_real.rs`. The shape comes from
    // M0 and from `CLAUDE.md`, and this is the only place it is exercised against text.
    let rules = Rules::embedded();
    assert_eq!(
        rules.event("[INFO] - Game Over. Killed by (9.0) spawned by (84.0) damage flags (0)"),
        Some(Event::Died {
            killer: "9.0".into(),
            spawner: "84.0".into(),
        })
    );
}

#[test]
fn the_cutscene_and_the_achievement_and_the_save() {
    let rules = Rules::embedded();
    assert_eq!(
        rules.event("[INFO] - playing cutscene 19 (Mega Satan)."),
        Some(Event::Ended {
            cutscene: 19,
            name: "Mega Satan".into(),
        })
    );
    assert_eq!(
        rules.event("[INFO] - unlock steam achievement '19'"),
        Some(Event::AchievementUnlocked { id: 19 })
    );
    assert_eq!(
        rules.event(
            "[INFO] - Saving PersistentGameData to Steam Cloud: rep+persistentgamedata1.dat."
        ),
        Some(Event::SaveWritten {
            file: "rep+persistentgamedata1.dat".into(),
        })
    );
}

#[test]
fn an_unknown_line_is_not_an_event_and_not_an_error() {
    let rules = Rules::embedded();
    // 87% of a real log is this kind of thing, and third-party mods add their own.
    assert_eq!(
        rules.event("[INFO] - Lua Debug: something a mod said"),
        None
    );
    assert_eq!(rules.event("Point Scale: 2.000000 "), None);
    assert_eq!(rules.event(""), None);
}

#[test]
fn a_rules_file_that_does_not_parse_says_so_instead_of_panicking() {
    assert!(Rules::parse("{ not json").is_err());
    assert!(Rules::parse(r#"{"version":1,"patterns":{"died":"("}}"#).is_err());
}

#[test]
fn the_embedded_file_is_the_one_that_ships() {
    // A pattern that stopped compiling would make its rule silently absent; `embedded()`
    // building at all is what turns that into a failure.
    assert_eq!(Rules::embedded().version(), 1);
}
