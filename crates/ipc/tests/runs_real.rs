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
