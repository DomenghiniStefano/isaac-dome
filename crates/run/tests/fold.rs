//! Hand-written event sequences: no log, no game, no catalog. That is the point of the crate
//! being pure.

use run::{Event, Generated, ItemKind, ItemKinds, Outcome, Pass, Pickup, Run, SeedKind};

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

/// The ids of what was picked up, in order: most tests are about which items, not where from.
fn ids(collected: &[Pickup]) -> Vec<u32> {
    collected.iter().map(|p| p.id).collect()
}

fn item_from(id: u32, pool: &str) -> Event {
    Event::ItemAdded {
        id,
        name: format!("item {id}"),
        player: 0,
        character: "Isaac".into(),
        pool: pool.into(),
    }
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
    assert_eq!(ids(&runs[0].collected), vec![225]);
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
    assert_eq!(ids(&runs[0].collected), vec![2]);
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
    // The killer is often a shot and the spawner the monster that fired it: both are kept.
    assert_eq!(
        died[0].outcome,
        Outcome::Died {
            killer: "9.0".into(),
            spawner: "84.0".into()
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
    assert_eq!(ids(&runs[0].collected), vec![225]);
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

/// `Initialized player with Variant 0 and Subtype N` is the only line that says **which**
/// character is being played: the item line gives a name, and the game gives a Tainted
/// character the base form's name. The subtype is the id, so the app stops having to say
/// "Cain or Tainted Cain".
#[test]
fn a_run_takes_the_character_id_the_log_states() {
    let runs = Run::fold(
        [
            Event::PlayerInitialized {
                variant: 0,
                subtype: 3,
            },
            seed(1, SeedKind::New),
        ]
        .into_iter(),
        &Kinds(&[]),
    );
    assert_eq!(runs[0].character_id, Some(3));
}

/// **The line comes before the seed on a solo run and after it online**, measured on this
/// machine's logs. A fold that only looked forward would miss every solo run, and one that
/// only looked back would miss every online one, so the pending init is kept until a run
/// starts — and cleared by the run that takes it, or nobody would ever play a second
/// character.
#[test]
fn the_line_is_taken_whether_it_arrives_before_or_after_the_seed() {
    let after = Run::fold(
        [
            seed(1, SeedKind::New),
            Event::PlayerInitialized {
                variant: 0,
                subtype: 7,
            },
        ]
        .into_iter(),
        &Kinds(&[]),
    );
    assert_eq!(after[0].character_id, Some(7));

    let two = Run::fold(
        [
            Event::PlayerInitialized {
                variant: 0,
                subtype: 3,
            },
            seed(1, SeedKind::New),
            seed(2, SeedKind::New),
        ]
        .into_iter(),
        &Kinds(&[]),
    );
    assert_eq!(two[0].character_id, Some(3));
    assert_eq!(
        two[1].character_id, None,
        "the second run did not say who was playing it"
    );
}

/// In co-op the line appears once per player. The run keeps the **first**: the others are
/// other people at the same table, and the app speaks about the profile it reads.
#[test]
fn a_second_player_does_not_take_the_runs_character() {
    let runs = Run::fold(
        [
            seed(1, SeedKind::Net),
            Event::PlayerInitialized {
                variant: 0,
                subtype: 30,
            },
            Event::PlayerInitialized {
                variant: 0,
                subtype: 7,
            },
        ]
        .into_iter(),
        &Kinds(&[]),
    );
    assert_eq!(runs[0].character_id, Some(30));
}

/// The floor the generation summary attaches to, and `Level::Init` is the only thing that says
/// where one floor ends and the next begins.
fn floor(stage: u32, stage_type: u32) -> Event {
    Event::FloorEntered {
        stage,
        stage_type,
        seed: 1,
    }
}

fn generated(rooms: u32, loops: u32) -> Event {
    Event::RoomsGenerated { rooms, loops }
}

#[test]
fn a_floor_the_log_did_not_describe_says_so_and_does_not_say_zero() {
    // Greed mode: seven floors and not one `generate...` line, measured on
    // `samples/logs/20260912-greed-online-coop.log.txt`. A floor with no summary must not read
    // as a floor of no rooms — that is the same error `Unmodelled` exists for in `floor` and
    // `Partial` in `graph`.
    let runs = Run::fold([started(), floor(1, 0)].into_iter(), &Kinds(&[]));
    assert_eq!(runs[0].floors[0].generated, Generated::NotSaid);
}

#[test]
fn one_generation_pass_belongs_to_the_floor_above_it() {
    let runs = Run::fold(
        [started(), floor(1, 0), generated(19, 12)].into_iter(),
        &Kinds(&[]),
    );
    assert_eq!(
        runs[0].floors[0].generated,
        Generated::Once {
            rooms: 19,
            loops: 12
        }
    );
}

#[test]
fn a_second_pass_under_one_floor_is_kept_beside_the_first_and_not_instead_of_it() {
    // The one multi-pass floor in the whole corpus — `m_Stage 4, m_StageType 4`, Mines II,
    // which is also the only floor in it that has an area of its own. Both passes report 19
    // rooms, so the corpus cannot say which of them is the floor that was walked: keeping both
    // is the only reading that does not invent the answer.
    let runs = Run::fold(
        [started(), floor(4, 4), generated(19, 12), generated(19, 14)].into_iter(),
        &Kinds(&[]),
    );
    assert_eq!(
        runs[0].floors[0].generated,
        Generated::Several {
            passes: vec![
                Pass {
                    rooms: 19,
                    loops: 12
                },
                Pass {
                    rooms: 19,
                    loops: 14
                }
            ]
        }
    );
}

#[test]
fn a_third_pass_follows_the_first_two_in_the_order_the_log_wrote_them() {
    // No floor in the corpus has three; the order is the log's, which is the one thing a
    // reader that has to pick a pass can rely on.
    let runs = Run::fold(
        [
            started(),
            floor(4, 4),
            generated(19, 12),
            generated(19, 14),
            generated(20, 3),
        ]
        .into_iter(),
        &Kinds(&[]),
    );
    assert_eq!(
        runs[0].floors[0].generated,
        Generated::Several {
            passes: vec![
                Pass {
                    rooms: 19,
                    loops: 12
                },
                Pass {
                    rooms: 19,
                    loops: 14
                },
                Pass {
                    rooms: 20,
                    loops: 3
                },
            ]
        }
    );
}

#[test]
fn a_summary_attaches_to_the_last_floor_announced_and_not_to_the_next_one() {
    let runs = Run::fold(
        [
            started(),
            floor(1, 0),
            generated(11, 8),
            floor(2, 0),
            generated(14, 8),
        ]
        .into_iter(),
        &Kinds(&[]),
    );
    assert_eq!(
        runs[0].floors[0].generated,
        Generated::Once {
            rooms: 11,
            loops: 8
        }
    );
    assert_eq!(
        runs[0].floors[1].generated,
        Generated::Once {
            rooms: 14,
            loops: 8
        }
    );
}

#[test]
fn a_summary_before_any_floor_belongs_to_no_floor_and_invents_none() {
    // A read can begin anywhere: `log.txt` is tailed from an offset and a session folder can
    // hold a floor whose `Level::Init` the previous read already took. A summary with no floor
    // to attach to is dropped, because the alternative is a floor the game never announced.
    let runs = Run::fold([started(), generated(19, 12)].into_iter(), &Kinds(&[]));
    assert!(runs[0].floors.is_empty());
}

/// Card #80, P1: on a solo launch the line for the **next** run arrives while the fold still
/// holds the finished one — the run is closed only by the next seed. It used to reach the
/// finished run, which already had its character, and be dropped there: from the second run
/// of a launch on, every run went without its id.
#[test]
fn the_init_after_a_run_ends_is_the_next_runs() {
    let runs = Run::fold(
        [
            Event::PlayerInitialized {
                variant: 0,
                subtype: 3,
            },
            seed(1, SeedKind::New),
            Event::Died {
                killer: "Monstro".to_string(),
                spawner: String::new(),
            },
            Event::PlayerInitialized {
                variant: 0,
                subtype: 8,
            },
            seed(2, SeedKind::New),
        ]
        .into_iter(),
        &Kinds(&[]),
    );
    assert_eq!(runs[0].character_id, Some(3));
    assert_eq!(runs[1].character_id, Some(8), "run 2 is played by 8");
}

fn init(subtype: u32) -> Event {
    Event::PlayerInitialized {
        variant: 0,
        subtype,
    }
}

fn died() -> Event {
    Event::Died {
        killer: "Monstro".to_string(),
        spawner: String::new(),
    }
}

/// Review of card #80, P1: a run resumed with `Continue` logs its player line before its seed,
/// like a fresh one. That line is the **resumed** run's, and it must not be kept for the run
/// after: the next run is played by whoever its own line names.
#[test]
fn the_init_of_a_resumed_run_is_not_kept_for_the_next_one() {
    let runs = Run::fold(
        [
            init(8),
            seed(1, SeedKind::New),
            init(8),
            seed(1, SeedKind::Continue),
            died(),
            init(5),
            seed(2, SeedKind::New),
        ]
        .into_iter(),
        &Kinds(&[]),
    );
    assert_eq!(runs.len(), 2, "the Continue resumed run 1");
    assert_eq!(runs[0].character_id, Some(8));
    assert_eq!(runs[1].character_id, Some(5), "run 2 is played by 5");
}

/// A second player line in the middle of a solo run — Esau beside Jacob, a Strawman — is not the
/// next run's either: the next run's own line comes after it, and that one names who plays.
#[test]
fn the_last_init_before_a_seed_names_the_run() {
    let runs = Run::fold(
        [
            init(19),
            seed(1, SeedKind::New),
            init(20),
            died(),
            init(0),
            seed(2, SeedKind::New),
        ]
        .into_iter(),
        &Kinds(&[]),
    );
    assert_eq!(runs[0].character_id, Some(19));
    assert_eq!(runs[1].character_id, Some(0), "run 2 is played by 0");
}

#[test]
fn a_pickup_keeps_the_pool_it_came_from_and_the_floor_it_was_taken_on() {
    // A read can begin inside a floor whose `Level::Init` an earlier read took: an item taken
    // before any floor this read saw has no floor, rather than the first one's.
    let runs = Run::fold(
        [
            started(),
            Event::RoomTransition,
            item_from(33, "treasure"),
            floor(2, 0),
            item_from(105, "shop"),
            floor(3, 4),
            item_from(51, "devil"),
        ]
        .into_iter(),
        &Kinds(&[]),
    );
    assert_eq!(
        runs[0].collected,
        vec![
            Pickup {
                id: 33,
                pool: "treasure".into(),
                floor: None
            },
            Pickup {
                id: 105,
                pool: "shop".into(),
                floor: Some(0)
            },
            Pickup {
                id: 51,
                pool: "devil".into(),
                floor: Some(1)
            },
        ]
    );
}

// Greed writes the normal path's `m_Stage, m_StageType` pairs — measured on
// `20260912-greed-online-coop.log.txt`: `1,1`, `2,0`, `3,2` … `7,0` — so a floor's numbers cannot
// say which mode it is in. The run has to, from what only Greed writes.
#[test]
fn a_greed_wave_says_the_run_is_greed_mode() {
    let runs = Run::fold(
        [started(), floor(1, 1), Event::GreedWave].into_iter(),
        &Kinds(&[]),
    );
    assert!(runs[0].greed);
}

// The archive keeps events, not lines: a Greed run read before the wave had an event of its own
// still says so through its items, which Greed draws from pools of its own.
#[test]
fn an_item_from_a_greed_pool_says_the_run_is_greed_mode() {
    let runs = Run::fold(
        [
            started(),
            Event::RoomTransition,
            item_from(33, "greedTreasure"),
        ]
        .into_iter(),
        &Kinds(&[]),
    );
    assert!(runs[0].greed);
}

#[test]
fn a_run_with_neither_is_not_greed_mode() {
    let runs = Run::fold(
        [
            started(),
            floor(1, 0),
            Event::RoomTransition,
            item_from(33, "treasure"),
        ]
        .into_iter(),
        &Kinds(&[]),
    );
    assert!(!runs[0].greed);
}

// The owner's rule, 2026-10-04: locally only one run can be open, and starting a new one in the
// same mode abandons the one before — but starting an online run leaves the local one as it was,
// open and resumable. So the fold keeps one open run per mode, not one in all.
const LOCAL_A: u32 = 1_111_111_111;
const ONLINE_B: u32 = 2_222_222_222;
const LOCAL_C: u32 = 3_333_333_333;

fn seeds(runs: &[Run]) -> Vec<u32> {
    runs.iter().map(|r| r.seed_numeric).collect()
}

#[test]
fn an_online_run_leaves_the_open_local_run_open() {
    let runs = Run::fold(
        [seed(LOCAL_A, SeedKind::New), seed(ONLINE_B, SeedKind::Net)].into_iter(),
        &Kinds(&[]),
    );
    assert_eq!(seeds(&runs), vec![LOCAL_A, ONLINE_B]);
    assert_eq!(runs[0].outcome, Outcome::Open);
}

// The same local run resumed after the online one is that run, not a third: counting it twice is
// the failure this exists for. And the events after the resume are the local run's.
#[test]
fn the_local_run_resumed_after_an_online_one_is_the_same_run() {
    let runs = Run::fold(
        [
            seed(LOCAL_A, SeedKind::New),
            seed(ONLINE_B, SeedKind::Net),
            seed(LOCAL_A, SeedKind::Continue),
            floor(2, 0),
        ]
        .into_iter(),
        &Kinds(&[]),
    );
    assert_eq!(seeds(&runs), vec![LOCAL_A, ONLINE_B]);
    assert_eq!(runs[0].floors.len(), 1, "the floor after the resume is A's");
    assert!(runs[1].floors.is_empty());
}

// A new local run abandons the local one before it, and the online run is not the local one's
// business.
#[test]
fn a_new_local_run_abandons_the_local_one_and_not_the_online_one() {
    let runs = Run::fold(
        [
            seed(LOCAL_A, SeedKind::New),
            seed(ONLINE_B, SeedKind::Net),
            seed(LOCAL_C, SeedKind::New),
        ]
        .into_iter(),
        &Kinds(&[]),
    );
    assert_eq!(seeds(&runs), vec![LOCAL_A, ONLINE_B, LOCAL_C]);
    assert_eq!(runs[0].outcome, Outcome::Abandoned);
    assert_eq!(runs[1].outcome, Outcome::Open);
    assert_eq!(runs[2].outcome, Outcome::Open);
}

// The same rule on the online side: a second online run abandons the first.
#[test]
fn a_new_online_run_abandons_the_online_one_before_it() {
    let runs = Run::fold(
        [
            seed(ONLINE_B, SeedKind::Net),
            seed(LOCAL_A, SeedKind::New),
            seed(LOCAL_C, SeedKind::Net),
        ]
        .into_iter(),
        &Kinds(&[]),
    );
    assert_eq!(seeds(&runs), vec![ONLINE_B, LOCAL_A, LOCAL_C]);
    assert_eq!(runs[0].outcome, Outcome::Abandoned);
    assert_eq!(runs[1].outcome, Outcome::Open);
}
