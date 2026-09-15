//! Does the archive agree with the game's own numbers?
//!
//! The counters are read from the save with `core_save`; the runs are folded from the log with
//! `run`. Neither knows about the other, which is the only reason this would be worth running.
//!
//! **The answer on today's data is that it cannot be measured, and that is the result.** Of the
//! three sources on disk, two were excluded before the work started: the co-op Greed session of
//! 2026-09-12 did not move `STREAK_COUNTER [22]` at all, and `20260912-solo-judas` is an `Open`
//! run where nothing moving is correct and proves nothing. The one case left was 2026-09-08 —
//! the Mega Satan win with Judas, with the saves of the 7th and the 8th around it — and
//! measuring it on 2026-09-13 showed the window does not contain the run: **no achievement slot
//! turns on across it**, while the log unlocks two, and 2 counters out of 523 move by one each.
//! The dated backup of the 8th was written before that run was played.
//!
//! So these tests keep the two halves of that finding answerable rather than asserting an
//! agreement nobody has seen. What would close it is one solo, non-Greed win with a snapshot
//! either side — `core-save`'s `live_probe` takes them.

use core_save::{Kind, Save};
use run::{ItemKind, ItemKinds, Outcome, Rules, Run, Tail};

/// Item kinds do not change an outcome, and this file is about outcomes. A table keeps the
/// catalog — and the game's installation — out of it.
struct Passives;
impl ItemKinds for Passives {
    fn kind_of(&self, _id: u32) -> ItemKind {
        ItemKind::Passive
    }
}

/// The counters this file is about, by their index in section 2.
const DEATHS: usize = 10;
const STREAK: usize = 22;
const BEST_STREAK: usize = 23;

const BEFORE: &str = "20260907.rep+persistentgamedata1.dat";
const AFTER: &str = "20260908.rep+persistentgamedata1.dat";
const LOG: &str = "20260908-run-megasatan-judas.log.txt";

fn counters(name: &str) -> Option<Vec<u32>> {
    let path = test_support::sample(name)?;
    let bytes = std::fs::read(path).ok()?;
    Save::parse(&bytes).ok()?.u32s(Kind::Counters)
}

fn flags(name: &str) -> Option<Vec<bool>> {
    let path = test_support::sample(name)?;
    let bytes = std::fs::read(path).ok()?;
    Save::parse(&bytes).ok()?.flags(Kind::Achievements)
}

fn runs_of(log_name: &str) -> Option<Vec<Run>> {
    let path = test_support::log_sample(log_name)?;
    let bytes = std::fs::read(path).ok()?;
    let rules = Rules::embedded();
    let mut tail = Tail::default();
    let events: Vec<_> = tail
        .advance(&bytes)
        .into_iter()
        .filter_map(|line| rules.event(&line))
        .collect();
    Some(Run::fold(events.into_iter(), &Passives))
}

#[test]
fn the_log_of_2026_09_08_folds_to_one_run_and_it_was_won() {
    // The half of the comparison that works. Also the vacuity guard for the other half: without
    // a win in the log there would be nothing to look for in the save.
    let Some(runs) = runs_of(LOG) else {
        test_support::skip("20260908-run-megasatan-judas.log.txt is missing");
        return;
    };
    assert_eq!(runs.len(), 1, "one launch, one run");
    assert!(
        matches!(runs[0].outcome, Outcome::Won { .. }),
        "the run this window is about was won: {:?}",
        runs[0].outcome
    );
}

#[test]
fn the_window_of_2026_09_08_does_not_contain_the_run_its_log_holds() {
    // Measured on 2026-09-13, and it is why no agreement with the game's counters is asserted
    // anywhere in this repo yet.
    //
    // The reading rests on the achievements, not on the counters, because it does not depend on
    // any mapping: **no slot at all turns on** between the two saves, while the log calls
    // `unlock steam achievement` twice. Whatever the numbering, two unlocks cannot leave 642
    // slots untouched — so the save of the 8th was written before the run was played.
    let (Some(before), Some(after), Some(fb), Some(fa)) = (
        counters(BEFORE),
        counters(AFTER),
        flags(BEFORE),
        flags(AFTER),
    ) else {
        test_support::skip("the 2026-09-07/08 window is missing");
        return;
    };

    let gained = fb
        .iter()
        .zip(fa.iter())
        .filter(|(b, a)| !**b && **a)
        .count();
    let moved: Vec<usize> = before
        .iter()
        .zip(after.iter())
        .enumerate()
        .filter(|(_, (b, a))| b != a)
        .map(|(i, _)| i)
        .collect();
    eprintln!("achievements gained: {gained}; counters moved: {moved:?}");
    for (name, i) in [
        ("DEATHS", DEATHS),
        ("STREAK", STREAK),
        ("BEST_STREAK", BEST_STREAK),
    ] {
        eprintln!("{name} [{i}]: {} -> {}", before[i], after[i]);
    }

    assert_eq!(
        gained, 0,
        "the reading of this window is that it holds no unlock at all"
    );
    assert!(
        moved.len() < 10,
        "a played run moves far more than this: {moved:?}"
    );
    // The three the agreement would have been about, stated so that a window which *does* hold a
    // run makes this test fail and be rewritten, rather than quietly keep passing.
    for i in [DEATHS, STREAK, BEST_STREAK] {
        assert_eq!(
            before[i], after[i],
            "counter {i} moved: this window holds a run after all, and the agreement can be measured"
        );
    }
}

const W_BEFORE: &str = "20260915-pre.rep+persistentgamedata1.dat";
const W_AFTER: &str = "20260915.rep+persistentgamedata1.dat";
const W_LOG: &str = "20260915-eden-void-solo.log.txt";

/// The half of the window that lives in `samples/windows/`: the snapshot from before the run.
fn window_counters(name: &str) -> Option<Vec<u32>> {
    let path = test_support::window_sample(name)?;
    let bytes = std::fs::read(path).ok()?;
    Save::parse(&bytes).ok()?.u32s(Kind::Counters)
}

fn window_flags(name: &str) -> Option<Vec<bool>> {
    let path = test_support::window_sample(name)?;
    let bytes = std::fs::read(path).ok()?;
    Save::parse(&bytes).ok()?.flags(Kind::Achievements)
}

#[test]
fn the_log_of_2026_09_15_folds_to_one_run_and_it_was_won() {
    // The vacuity guard for the two tests below: without a won run in the log there is nothing
    // to look for in the save. This log is a `Continue` — the run was resumed and this launch
    // holds only its last floor, `Level::Init m_Stage 12` (The Void) — which is exactly the
    // shape `Run::fold` decides by seed rather than by the label on the seed line.
    let Some(runs) = runs_of(W_LOG) else {
        test_support::skip("20260915-eden-void-solo.log.txt is missing");
        return;
    };
    assert_eq!(runs.len(), 1, "one launch, one run: {runs:?}");
    assert!(
        matches!(runs[0].outcome, Outcome::Won { .. }),
        "the run ended on The Void's cutscene: {:?}",
        runs[0].outcome
    );
}

/// **The log's `unlock steam achievement '<id>'` and the save's achievement slot are the same
/// number.** Measured here for the first time on 2026-09-15: the 2026-09-08 window could not say
/// so — it holds two unlocks in the log and **no** slot turning on in the save — and the test
/// that reads it says as much, "whatever the numbering".
///
/// This window holds exactly two unlocks and exactly two slots turning on, and they are the same
/// two numbers. It is the direct link between the two sources the archive is built on, and it
/// cost nothing but a window that actually contains its run.
#[test]
fn the_logs_achievement_ids_are_the_saves_achievement_slots() {
    let (Some(before), Some(after), Some(runs)) =
        (window_flags(W_BEFORE), flags(W_AFTER), runs_of(W_LOG))
    else {
        test_support::skip("the 2026-09-14/15 window is missing");
        return;
    };

    let turned_on: Vec<u32> = before
        .iter()
        .zip(after.iter())
        .enumerate()
        .filter(|(_, (b, a))| !**b && **a)
        .map(|(i, _)| i as u32)
        .collect();
    let mut unlocked: Vec<u32> = runs.iter().flat_map(|r| r.achievements.clone()).collect();
    unlocked.sort_unstable();

    // Vacuity guard: a window where nothing was unlocked would pass this by comparing two empty
    // lists, which is the failure mode of the 2026-09-08 window read carelessly.
    assert!(
        !unlocked.is_empty(),
        "the log unlocks nothing: this test has no subject"
    );
    assert_eq!(
        turned_on, unlocked,
        "the slots the save turned on and the ids the log unlocked"
    );
}

/// The agreement the spec asked for since 2026-09-12 and no window on disk could answer: **one
/// solo, non-Greed win with a snapshot either side.** `STREAK_COUNTER` rises by exactly one and
/// `DEATHS` does not move, which is what one win and no death mean.
///
/// **The window is a day wide, not tight around the run** — the "before" is the dated backup the
/// game wrote at the end of the 14th and the "after" is the live save of the 15th — so it says
/// the counters agree with *this* run because only one run was played in it, which the single
/// `+1` on the streak is itself the evidence for. A tight window would need `live_probe`
/// running, and this one was taken after the fact.
#[test]
fn the_window_of_2026_09_15_holds_one_win_and_no_death() {
    let (Some(before), Some(after), Some(runs)) =
        (window_counters(W_BEFORE), counters(W_AFTER), runs_of(W_LOG))
    else {
        test_support::skip("the 2026-09-14/15 window is missing");
        return;
    };

    let wins = runs
        .iter()
        .filter(|r| matches!(r.outcome, Outcome::Won { .. }))
        .count() as u32;
    let deaths = runs
        .iter()
        .filter(|r| matches!(r.outcome, Outcome::Died { .. }))
        .count() as u32;
    eprintln!(
        "log: {wins} won, {deaths} died | save: STREAK {} -> {}, DEATHS {} -> {}",
        before[STREAK], after[STREAK], before[DEATHS], after[DEATHS]
    );

    // Vacuity guard: the whole point is a window that *does* contain its run.
    assert_eq!(wins, 1, "this window is about one win");
    assert_eq!(
        after[STREAK] - before[STREAK],
        wins,
        "the streak rose by the number of wins the log folds to"
    );
    assert_eq!(
        after[DEATHS], before[DEATHS],
        "nothing died in the log, and DEATHS agrees"
    );
    assert_eq!(deaths, 0, "and the log says so too");
}

/// **Section 8 is indexed by cutscene number.** Index 19 was the identity mapping for cutscene
/// 19; this window is the second point, and it is a different number on a different day: the log
/// plays cutscene 22 (The Void) and cell 22 of section 8 rises by exactly one.
///
/// It does not close the section — index 2 also rose by one here and no cutscene 2 was played,
/// so what that cell counts is still open — but it does close "the mapping might be offset":
/// identity at two points and an offset at a third cannot both hold.
#[test]
fn section_8_counts_the_cutscene_the_log_played() {
    let (Some(before), Some(after), Some(runs)) =
        (window_section8(W_BEFORE), section8(W_AFTER), runs_of(W_LOG))
    else {
        test_support::skip("the 2026-09-14/15 window is missing");
        return;
    };

    let endings: Vec<&str> = runs
        .iter()
        .filter_map(|r| match &r.outcome {
            Outcome::Won { ending } => Some(ending.as_str()),
            Outcome::Died { .. } | Outcome::Open | Outcome::Abandoned => None,
        })
        .collect();
    assert_eq!(
        endings,
        vec!["The Void"],
        "the run this window is about ended on The Void, cutscene 22"
    );
    assert_eq!(
        after[22] - before[22],
        1,
        "cell 22 of section 8 rose once, for the one cutscene 22 the log played"
    );
}

fn section8(name: &str) -> Option<Vec<u32>> {
    let path = test_support::sample(name)?;
    let bytes = std::fs::read(path).ok()?;
    Save::parse(&bytes).ok()?.u32s(Kind::CutsceneCounters)
}

fn window_section8(name: &str) -> Option<Vec<u32>> {
    let path = test_support::window_sample(name)?;
    let bytes = std::fs::read(path).ok()?;
    Save::parse(&bytes).ok()?.u32s(Kind::CutsceneCounters)
}
