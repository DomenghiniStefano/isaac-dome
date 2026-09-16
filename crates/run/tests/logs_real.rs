//! The rules, against logs the game actually wrote. Skips with a note when `samples/logs/` is
//! not there: the folder is git-ignored and the suite stays green for anyone who clones.

use run::{Event, ItemKind, ItemKinds, Outcome, Rules, Run, Tail};

/// The fold needs item kinds and this test has no catalog. Every id reads as a passive, which
/// is wrong for actives and right for everything these tests assert: nothing here looks at
/// `held_active`. Said out loud rather than left to be discovered.
struct AllPassive;

impl ItemKinds for AllPassive {
    fn kind_of(&self, _id: u32) -> ItemKind {
        ItemKind::Passive
    }
}

fn events_of(name: &str) -> Option<Vec<Event>> {
    let path = test_support::log_sample(name)?;
    let bytes = std::fs::read(path).ok()?;
    let rules = Rules::embedded();
    let mut tail = Tail::default();
    Some(
        tail.advance(&bytes)
            .iter()
            .filter_map(|line| rules.event(line))
            .collect(),
    )
}

#[test]
fn a_real_log_yields_the_events_its_lines_promise() {
    let Some(events) = events_of("20260908-run-megasatan-judas.log.txt") else {
        return;
    };
    // Measured with `grep -c` on 2026-09-13, on this file: 1 seed line, 10 `Level::Init`,
    // 34 collectibles, 343 room transitions, 2 achievements, 132 save writes.
    let count = |f: fn(&Event) -> bool| events.iter().filter(|e| f(e)).count();
    assert_eq!(count(|e| matches!(e, Event::RunStarted { .. })), 1);
    assert_eq!(count(|e| matches!(e, Event::FloorEntered { .. })), 10);
    assert_eq!(count(|e| matches!(e, Event::ItemAdded { .. })), 34);
    assert_eq!(count(|e| matches!(e, Event::RoomTransition)), 343);
    assert_eq!(count(|e| matches!(e, Event::AchievementUnlocked { .. })), 2);
    assert_eq!(count(|e| matches!(e, Event::SaveWritten { .. })), 132);
}

#[test]
fn the_run_that_was_won_reads_as_won() {
    let Some(events) = events_of("20260908-run-megasatan-judas.log.txt") else {
        return;
    };
    let runs = Run::fold(events.into_iter(), &AllPassive);
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].character.as_deref(), Some("Judas"));
    assert_eq!(
        runs[0].outcome,
        Outcome::Won {
            ending: "Mega Satan".into()
        }
    );
}

#[test]
fn the_intro_cutscene_belongs_to_no_run() {
    // `playing cutscene 1 (Intro).` is at line 77 and the seed line is at 333. An `Ended`
    // before any run must not become an outcome, and it must not invent a run either.
    let Some(events) = events_of("20260908-run-megasatan-judas.log.txt") else {
        return;
    };
    let endings = events
        .iter()
        .filter(|e| matches!(e, Event::Ended { .. }))
        .count();
    assert_eq!(endings, 2, "the intro and the ending");
    assert_eq!(Run::fold(events.into_iter(), &AllPassive).len(), 1);
}

#[test]
fn the_run_that_was_left_open_reads_as_open_and_not_as_a_failure() {
    let Some(events) = events_of("20260912-solo-judas.log.txt") else {
        return;
    };
    let runs = Run::fold(events.into_iter(), &AllPassive);
    assert!(!runs.is_empty());
    assert_eq!(runs.last().map(|r| &r.outcome), Some(&Outcome::Open));
}

#[test]
fn the_online_session_says_it_was_online() {
    // `[Net, 1]`. Whether an online run counts toward the game's own streak is open in the
    // spec; keeping the label is what will let it be measured instead of decided.
    let Some(events) = events_of("20260912-greed-online-coop.log.txt") else {
        return;
    };
    let runs = Run::fold(events.into_iter(), &AllPassive);
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].seed_kind, run::SeedKind::Net);
}

#[test]
fn judas_starts_with_the_book_of_belial_and_it_is_not_a_treasure_find() {
    // The judgment of §2, on the real line that motivated it: id 34, `from pool treasure`,
    // emitted before the first room transition.
    let Some(events) = events_of("20260908-run-megasatan-judas.log.txt") else {
        return;
    };
    let runs = Run::fold(events.into_iter(), &AllPassive);
    assert_eq!(runs[0].starting_items, vec![34]);
    assert!(!runs[0].collected.contains(&34));
}

#[test]
fn a_death_the_game_wrote_becomes_a_run_that_ended_in_one() {
    // **This test replaced the one that asserted the gap**, and the way it arrived is the point.
    // `no_sample_log_contains_a_death` said `samples/logs/` held no `Game Over` and failed the
    // day one did — which happened on 2026-09-13, when the archive was run against this
    // machine's real `online_logs\` and folded **23 deaths out of 28 sessions**. The gap was
    // never in the data: it was in which files had been copied into `samples/`.
    let Some(events) = events_of("20260824-online-deaths.log.txt") else {
        return;
    };
    let deaths: Vec<&Event> = events
        .iter()
        .filter(|e| matches!(e, Event::Died { .. }))
        .collect();
    assert_eq!(deaths.len(), 2, "this session ends two runs in a death");

    // The killer is the entity the game names, kept as it wrote it — `type.variant`, which is
    // the same shape `crates/wiki` indexes bosses by. Nothing here interprets it.
    let Event::Died { killer, spawner } = deaths[0] else {
        unreachable!("filtered above")
    };
    assert!(
        killer.contains('.') && !killer.is_empty(),
        "the killer keeps the game's own pair: {killer}"
    );
    assert!(!spawner.is_empty());

    // And the fold reaches the outcome, which is the half the rules cannot do.
    let runs = Run::fold(events.into_iter(), &AllPassive);
    assert_eq!(
        runs.iter()
            .filter(|r| matches!(r.outcome, Outcome::Died { .. }))
            .count(),
        2
    );
}

/// The line the archive was not reading, and the one that ends an ambiguity the app has been
/// declaring in words: **`Initialized player with Variant 0 and Subtype 3`**, where the
/// subtype is the character's own id. The name is not enough — the game gives a Tainted
/// character the base form's name — and this says which one it is.
///
/// Measured on this machine's logs: the solo Judas run says 3, and the online session says
/// 30 (Tainted Eden) and 7 (Azazel) for the two players of one run.
#[test]
fn the_log_says_which_character_by_id_not_only_by_name() {
    let Some(events) = events_of("20260912-solo-judas.log.txt") else {
        return;
    };
    let subtypes: Vec<u32> = events
        .iter()
        .filter_map(|e| match e {
            Event::PlayerInitialized { variant, subtype } if *variant == 0 => Some(*subtype),
            _ => None,
        })
        .collect();
    assert_eq!(subtypes, vec![3], "Judas is character 3");
}

/// In co-op the line appears once per player, and that is what makes it a *run's* character
/// only for the first one: the second is somebody else at the same table.
#[test]
fn a_co_op_run_initializes_more_than_one_player() {
    let Some(events) = events_of("20260824-online-deaths.log.txt") else {
        return;
    };
    let subtypes: Vec<u32> = events
        .iter()
        .filter_map(|e| match e {
            Event::PlayerInitialized { subtype, .. } => Some(*subtype),
            _ => None,
        })
        .collect();
    assert!(
        subtypes.len() > 1,
        "the online session has more than one player: {subtypes:?}"
    );
    assert!(
        subtypes.contains(&25) || subtypes.contains(&30),
        "a Tainted character has its own subtype: {subtypes:?}"
    );
}

/// The events of a launch that produced no run, from `samples/launches/`.
fn launch_events(name: &str) -> Option<Vec<Event>> {
    let path = test_support::launch_sample(name)?;
    let bytes = std::fs::read(path).ok()?;
    let rules = Rules::embedded();
    let mut tail = Tail::default();
    Some(
        tail.advance(&bytes)
            .iter()
            .filter_map(|line| rules.event(line))
            .collect(),
    )
}

/// A launch nobody played, pinned line by line. Counted on the file **before** it was asserted:
/// `grep -cE` with each of the ten patterns in `crates/run/rules/events.json` finds exactly one
/// matching line — `[INFO] - playing cutscene 1 (Intro).`, line 69 — and zero for the other nine.
///
/// A fixture of the **Repentance v1.7.9b** era, which is the only log in this repo from before
/// the `+`: that the rules match the same way on an older version is measured here and nowhere
/// else (B60).
#[test]
fn the_launch_of_20240305_holds_one_event_and_it_is_the_intro() {
    let Some(events) = launch_events("20240305-rep179b-launch-no-run.log.txt") else {
        return;
    };
    assert_eq!(events.len(), 1, "{events:?}");
    assert!(
        matches!(events[0], Event::Ended { cutscene: 1, .. }),
        "{:?}",
        events[0]
    );
}

/// Every launch in `samples/launches/` speaks and folds into nothing, and the guard has two
/// halves because either one alone proves nothing.
///
/// **It speaks**: the events are not empty. `playing cutscene 1 (Intro).` is an `Ended` like any
/// other, which `the_intro_cutscene_belongs_to_no_run` pins from the other side, on a log that
/// does hold a run. An instrument that reports nothing proves nothing until it has been shown
/// able to report something.
///
/// **And it says zero**: an ending with no run open belongs to no run. That is the whole shape,
/// and nothing in `samples/logs/` has it — which is why the folder exists.
#[test]
fn a_launch_nobody_played_speaks_and_folds_into_no_run() {
    let rules = Rules::embedded();
    for path in test_support::launch_samples() {
        let bytes = std::fs::read(&path).expect("a sample that is present must read");
        let mut tail = Tail::default();
        let events: Vec<Event> = tail
            .advance(&bytes)
            .iter()
            .filter_map(|line| rules.event(line))
            .collect();
        assert!(
            !events.is_empty(),
            "{} matched no line at all: a silent instrument, not a launch with no run",
            path.display()
        );
        assert!(
            Run::fold(events.into_iter(), &AllPassive).is_empty(),
            "{} is filed as a launch nobody played, and it folded into a run",
            path.display()
        );
    }
}
