//! Hand-written event sequences: no log, no game, no catalog. That is the point of the crate
//! being pure.

use run::{Event, ItemKind, ItemKinds, Outcome, Run, SeedKind};

/// The catalog stands in as a table, so a test says which kind each id is.
struct Kinds(&'static [(u32, ItemKind)]);

impl ItemKinds for Kinds {
    fn kind_of(&self, id: u32) -> ItemKind {
        self.0
            .iter()
            .find(|(known, _)| *known == id)
            .map(|(_, kind)| *kind)
            .unwrap_or(ItemKind::Passive)
    }
}

fn seed(numeric: u32, kind: SeedKind) -> Event {
    Event::RunStarted {
        seed_words: "YKF6 QDN6".into(),
        seed_numeric: numeric,
        kind,
    }
}

fn started() -> Event {
    seed(2_913_253_616, SeedKind::New)
}

fn item(id: u32, character: &str) -> Event {
    Event::ItemAdded {
        id,
        name: format!("item {id}"),
        player: 0,
        character: character.into(),
        pool: "treasure".into(),
    }
}

#[test]
fn the_character_arrives_with_the_first_item_and_not_before() {
    let runs = Run::fold([started(), item(34, "Judas")].into_iter(), &Kinds(&[]));
    assert_eq!(runs[0].character.as_deref(), Some("Judas"));
}

#[test]
fn a_run_with_no_item_line_never_guesses_a_character() {
    let runs = Run::fold([started()].into_iter(), &Kinds(&[]));
    assert_eq!(runs[0].character, None);
}

#[test]
fn an_item_before_the_first_room_transition_is_the_starting_gift() {
    // `from pool treasure` is a lie on this line: Judas starts with Book of Belial and the
    // line is shaped exactly like a real pickup. Position is the only discriminator, and
    // getting it wrong over-counts treasure finds by one in every run, silently.
    let runs = Run::fold(
        [
            started(),
            item(34, "Judas"),
            Event::RoomTransition,
            item(225, "Judas"),
        ]
        .into_iter(),
        &Kinds(&[]),
    );
    assert_eq!(runs[0].starting_items, vec![34]);
    assert_eq!(runs[0].collected, vec![225]);
}

#[test]
fn several_starting_items_are_all_starting_items() {
    // Eden and the Tainted emit more than one line in that window; the rule is unchanged.
    let runs = Run::fold(
        [
            started(),
            item(1, "Eden"),
            item(2, "Eden"),
            Event::RoomTransition,
        ]
        .into_iter(),
        &Kinds(&[]),
    );
    assert_eq!(runs[0].starting_items, vec![1, 2]);
}

#[test]
fn only_the_first_room_transition_closes_the_starting_window() {
    let runs = Run::fold(
        [
            started(),
            item(1, "Isaac"),
            Event::RoomTransition,
            Event::RoomTransition,
            item(2, "Isaac"),
        ]
        .into_iter(),
        &Kinds(&[]),
    );
    assert_eq!(runs[0].starting_items, vec![1]);
    assert_eq!(runs[0].collected, vec![2]);
}

#[test]
fn an_active_replaces_the_active_before_it() {
    // B8 watched a run pick up five actives and end holding one. Summing the lines gives a
    // player carrying five books.
    let kinds = Kinds(&[
        (10, ItemKind::Active),
        (11, ItemKind::Active),
        (12, ItemKind::Passive),
    ]);
    let runs = Run::fold(
        [
            started(),
            Event::RoomTransition,
            item(10, "Judas"),
            item(12, "Judas"),
            item(11, "Judas"),
        ]
        .into_iter(),
        &kinds,
    );
    assert_eq!(runs[0].held_active, Some(11));
    assert_eq!(runs[0].passives, vec![12]);
}

#[test]
fn a_familiar_accumulates_like_a_passive_and_is_kept_apart_from_one() {
    let kinds = Kinds(&[(20, ItemKind::Familiar), (21, ItemKind::Familiar)]);
    let runs = Run::fold(
        [
            started(),
            Event::RoomTransition,
            item(20, "Isaac"),
            item(21, "Isaac"),
        ]
        .into_iter(),
        &kinds,
    );
    assert_eq!(runs[0].familiars, vec![20, 21]);
    assert!(runs[0].passives.is_empty());
}

#[test]
fn a_death_is_an_outcome_and_so_is_an_ending() {
    let died = Run::fold(
        [
            started(),
            Event::Died {
                killer: "9.0".into(),
                spawner: "84.0".into(),
            },
        ]
        .into_iter(),
        &Kinds(&[]),
    );
    assert_eq!(
        died[0].outcome,
        Outcome::Died {
            killer: "9.0".into()
        }
    );

    let won = Run::fold(
        [
            started(),
            Event::Ended {
                cutscene: 21,
                name: "Greed Mode".into(),
            },
        ]
        .into_iter(),
        &Kinds(&[]),
    );
    assert_eq!(
        won[0].outcome,
        Outcome::Won {
            ending: "Greed Mode".into()
        }
    );
}

#[test]
fn a_run_the_stream_stops_inside_is_open_and_not_a_failure() {
    let runs = Run::fold([started(), Event::RoomTransition].into_iter(), &Kinds(&[]));
    assert_eq!(runs[0].outcome, Outcome::Open);
}

#[test]
fn a_run_interrupted_by_a_different_seed_is_abandoned() {
    // The only inferred outcome: a run with neither a death nor an ending, ended by the next
    // run starting.
    let runs = Run::fold(
        [started(), seed(999, SeedKind::New)].into_iter(),
        &Kinds(&[]),
    );
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].outcome, Outcome::Abandoned);
    assert_eq!(runs[1].outcome, Outcome::Open);
}

#[test]
fn the_same_seed_again_is_the_same_run_resumed_and_not_an_abandonment() {
    // The game logs `[Continue, 1]` when a saved run is resumed, and it logs it with the seed
    // the run already had. Reading that as a new start would abandon a run that is still being
    // played — and would count it twice. The seed decides, not the label: a label can be a word
    // we have never met.
    let runs = Run::fold(
        [
            started(),
            item(34, "Judas"),
            Event::RoomTransition,
            seed(2_913_253_616, SeedKind::Continue),
            item(225, "Judas"),
        ]
        .into_iter(),
        &Kinds(&[]),
    );
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].outcome, Outcome::Open);
    assert_eq!(runs[0].starting_items, vec![34]);
    assert_eq!(runs[0].collected, vec![225]);
}

#[test]
fn a_replayed_seed_after_the_run_ended_is_a_second_run() {
    // A seed can be replayed deliberately. The first run is closed by its ending, so the same
    // number arriving again is a new run rather than a resumption.
    let runs = Run::fold(
        [
            started(),
            Event::Ended {
                cutscene: 19,
                name: "Mega Satan".into(),
            },
            started(),
        ]
        .into_iter(),
        &Kinds(&[]),
    );
    assert_eq!(runs.len(), 2);
    assert!(matches!(runs[0].outcome, Outcome::Won { .. }));
    assert_eq!(runs[1].outcome, Outcome::Open);
}

#[test]
fn an_online_run_says_so() {
    // The only discriminator for co-op that costs nothing. Whether co-op counts toward the
    // game's own streak is an open question in the spec; keeping the label is what will let
    // that be measured rather than decided.
    let runs = Run::fold([seed(1, SeedKind::Net)].into_iter(), &Kinds(&[]));
    assert_eq!(runs[0].seed_kind, SeedKind::Net);
}

#[test]
fn events_before_the_first_run_belong_to_no_run() {
    // A log begins mid-session: menu lines, an intro cutscene, then the first seed.
    let runs = Run::fold(
        [
            Event::Ended {
                cutscene: 1,
                name: "Intro".into(),
            },
            started(),
        ]
        .into_iter(),
        &Kinds(&[]),
    );
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].outcome, Outcome::Open);
}

#[test]
fn the_floors_are_kept_in_the_order_they_were_entered() {
    let runs = Run::fold(
        [
            started(),
            Event::FloorEntered {
                stage: 1,
                stage_type: 0,
                seed: 1,
            },
            Event::FloorEntered {
                stage: 2,
                stage_type: 1,
                seed: 2,
            },
        ]
        .into_iter(),
        &Kinds(&[]),
    );
    assert_eq!(runs[0].floors.len(), 2);
    assert_eq!(runs[0].floors[1].stage, 2);
}

#[test]
fn an_achievement_unlocked_mid_run_belongs_to_that_run() {
    let runs = Run::fold(
        [started(), Event::AchievementUnlocked { id: 19 }].into_iter(),
        &Kinds(&[]),
    );
    assert_eq!(runs[0].achievements, vec![19]);
}
