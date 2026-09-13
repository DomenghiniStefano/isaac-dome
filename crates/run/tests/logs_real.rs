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
fn no_sample_log_contains_a_death() {
    // **A gap, asserted so it cannot be forgotten.** All four logs in `samples/logs/` are wins
    // or open runs: `Event::Died` is covered only by `crates/run/tests/rules.rs`, against the
    // line shape recorded in M0, and by the fold's hand-written sequence. When a log with a
    // death arrives this test fails — and that failure is the signal to write the real one.
    let rules = Rules::embedded();
    let mut with_a_death = Vec::new();
    for path in test_support::log_samples() {
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        let mut tail = Tail::default();
        if tail
            .advance(&bytes)
            .iter()
            .filter_map(|line| rules.event(line))
            .any(|e| matches!(e, Event::Died { .. }))
        {
            with_a_death.push(path);
        }
    }
    assert!(
        with_a_death.is_empty(),
        "a log with a death exists now: write the real test for it — {with_a_death:?}"
    );
}
