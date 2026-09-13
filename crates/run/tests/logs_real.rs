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
