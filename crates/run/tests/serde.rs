//! Events and runs are stored as JSON rows. The archive is only as good as this round trip.

use run::{Event, Floor, Outcome, Run, SeedKind};

fn round_trip_event(e: &Event) -> Event {
    let json = serde_json::to_string(e).expect("an event serializes");
    serde_json::from_str(&json).expect("and reads back")
}

#[test]
fn every_event_survives_the_round_trip() {
    // One of each variant: a variant added without a case here is a variant nobody proved
    // could be stored.
    let all = vec![
        Event::RunStarted {
            seed_words: "FYQ8 QQ8G".to_string(),
            seed_numeric: 586_324_166,
            kind: SeedKind::New,
        },
        Event::FloorEntered {
            stage: 2,
            stage_type: 1,
            seed: 408_474_304,
        },
        Event::RoomEntered {
            id: "1.2".to_string(),
            name: "Start Room".to_string(),
        },
        Event::RoomTransition,
        Event::ItemAdded {
            id: 225,
            name: "Gimpy".to_string(),
            player: 0,
            character: "Cain".to_string(),
            pool: "treasure".to_string(),
        },
        Event::Died {
            killer: "9.0".to_string(),
            spawner: "84.0".to_string(),
        },
        Event::Ended {
            cutscene: 15,
            name: "Sheol".to_string(),
        },
        Event::AchievementUnlocked { id: 19 },
        Event::SaveWritten {
            file: "rep+persistentgamedata1.dat".to_string(),
        },
    ];
    for e in &all {
        assert_eq!(&round_trip_event(e), e);
    }
}

#[test]
fn the_three_seed_kinds_and_the_word_we_have_not_met_survive() {
    for k in [
        SeedKind::New,
        SeedKind::Continue,
        SeedKind::Net,
        SeedKind::Unknown,
    ] {
        let e = Event::RunStarted {
            seed_words: "AAA BBB".to_string(),
            seed_numeric: 1,
            kind: k,
        };
        assert_eq!(round_trip_event(&e), e);
    }
}

#[test]
fn a_folded_run_survives_the_round_trip() {
    let run = Run {
        seed_words: "FYQ8 QQ8G".to_string(),
        seed_numeric: 586_324_166,
        seed_kind: SeedKind::Net,
        character: Some("Judas".to_string()),
        starting_items: vec![34],
        collected: vec![225, 105],
        passives: vec![225],
        familiars: vec![],
        held_active: Some(105),
        floors: vec![Floor {
            stage: 2,
            stage_type: 1,
            seed: 408_474_304,
        }],
        achievements: vec![19],
        outcome: Outcome::Won {
            ending: "Mega Satan".to_string(),
        },
    };
    let json = serde_json::to_string(&run).expect("a run serializes");
    let back: Run = serde_json::from_str(&json).expect("and reads back");
    assert_eq!(back, run);
}

#[test]
fn each_outcome_survives_including_the_one_that_is_not_a_failure() {
    for o in [
        Outcome::Won {
            ending: "Sheol".to_string(),
        },
        Outcome::Died {
            killer: "9.0".to_string(),
        },
        Outcome::Abandoned,
        Outcome::Open,
    ] {
        let json = serde_json::to_string(&o).expect("an outcome serializes");
        let back: Outcome = serde_json::from_str(&json).expect("and reads back");
        assert_eq!(back, o);
    }
}
