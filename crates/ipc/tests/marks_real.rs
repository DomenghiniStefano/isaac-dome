//! The three columns located on 2026-09-08, checked against the profile's own history.
//!
//! Delirium for the 19 later characters, Mother and The Beast for the 14 originals were
//! not read off a document: they came out of the historical series, by asking which
//! unnamed cell moves on the day a given boss is killed. A table derived that way has to
//! keep answering to the series, or the next era moves it and nothing says so.
//!
//! The property here is the one that found the cells in the first place, turned around:
//! **a mark cannot appear without a kill of that boss between the two snapshots**. It is
//! not a pinned value — it holds whatever the profile does next, and it fails loudly if a
//! base is off by one, because a wrong base reads a neighbour's cell and lights up on the
//! wrong day.

mod support;

use ipc::{BOSSES, ROSTER};
use support::{cell_at_position, counters};
use test_support::{dated_series, SERIES};

/// The dated series `samples/` can hold, **each walked on its own**. Never one series
/// spanning both: the two editions are two profiles, and a jump between them would read
/// as progress.
///
/// The `rep_` row arrived on 2026-09-17, with B58, and it is the point of this table.
/// The three bases below were derived on the **642** era and were checked nowhere else,
/// so on a machine whose `rep+` series is a single file every property here had no window
/// to walk and returned early: the tables were guarded by nothing at all, and the suite
/// was green. The 638-era series re-derives Mother and The Beast on its own — three
/// windows, and on two of them the kill counter rises by exactly the number of new marks.
///
/// That is what answers B58's question about the era in between. 638 and 642 both put the
/// bases at 423 and 457; a cell cannot be inserted before 423 by one patch and removed by
/// the next, so 641 is bracketed rather than assumed. It is an inference from two measured
/// eras and not a third measurement, which is the most a machine with one 641-era snapshot
/// can say.
/// The columns whose position was derived, and the counter that counts that boss's
/// kills. Only these three: the other nine columns have no single kill counter to check
/// against (Mom's Heart and Greed have none at all), and this test exists for the ones
/// the series itself located.
const KILLS: [(usize, &str, usize); 3] = [
    (9, "Delirium", 187),
    (10, "Mother", 491),
    (11, "The Beast", 492),
];

/// Every snapshot of one series, as `(file name, section 2)`. Empty when the series isn't
/// stocked — `dated_series` has already declared why on stderr.
fn series_of(suffix: &str) -> Vec<(String, Vec<u32>)> {
    dated_series(suffix)
        .iter()
        .filter_map(|p| {
            let name = p.file_name()?.to_string_lossy().into_owned();
            Some((name, counters(p)?))
        })
        .collect()
}

#[test]
fn no_mark_appears_without_a_kill_of_that_boss() {
    let mut windows = 0;
    for suffix in SERIES {
        let series = series_of(suffix);
        for pair in series.windows(2) {
            windows += 1;
            let (before_name, before) = &pair[0];
            let (after_name, after) = &pair[1];
            for (column, boss, kill_index) in KILLS {
                let appeared: Vec<&str> = (0..ROSTER.len())
                    .filter(|&row| {
                        cell_at_position(row, column)
                            .and_then(|i| Some((*before.get(i)?, *after.get(i)?)))
                            .is_some_and(|(was, now)| was == 0 && now != 0)
                    })
                    .map(|row| ROSTER[row].name)
                    .collect();
                let kills = after
                    .get(kill_index)
                    .zip(before.get(kill_index))
                    .map(|(now, was)| now.saturating_sub(*was))
                    .unwrap_or(0);
                assert!(
                    appeared.len() as u32 <= kills,
                    "{before_name} → {after_name}: {} new {boss} mark(s) ({}) but the \
                     kill counter at {kill_index} only rose by {kills}. Either the \
                     block's base is wrong and these cells belong to something else, or \
                     {kill_index} isn't {boss}'s kills.",
                    appeared.len(),
                    appeared.join(", "),
                );
            }
        }
    }
    // A series of one snapshot has no window: the property then holds over nothing, and says
    // so rather than passing as if it had looked.
    if windows == 0 {
        test_support::skip(&format!(
            "none of the {} dated series holds two readable snapshots: no window to walk, \
             the kill property checked nothing",
            SERIES.len()
        ));
    }
}

/// Index 188 of the counters section: a bitmask of the characters that won, bit = the
/// character's id, which for the 14 originals is their row in the matrix. REPENTOGON
/// calls it `CHARACTER_LAST_RUN_WIN`; that it is a mask, and not an id or a count, is
/// what six transitions in this series say — see the test below.
const WINNER_MASK: usize = 188;

/// The check that pins a block's **base**, not just its neighbourhood.
///
/// The property above compares counts, so it survives a base that is off by one: a
/// neighbouring cell lights up on the same day and the arithmetic still works. This one
/// compares *identities*. When a single mark appeared among the 14 originals and the
/// winner mask names a single character, the two have to be the same character — and if
/// a base were shifted, the cell that moved would be read as the character next door.
///
/// Deliberately narrow: the mask holds the most recent run, so a day with several
/// winners says nothing about which of them earned which mark. Six transitions in this
/// series qualify, and all six agree.
#[test]
fn the_character_that_won_is_the_character_whose_mark_appeared() {
    let mut agreed = 0;
    for suffix in SERIES {
        let series = series_of(suffix);
        for pair in series.windows(2) {
            let (before_name, before) = &pair[0];
            let (after_name, after) = &pair[1];
            // Rows among the 14 originals with a mark that went from absent to present,
            // whichever column it was in.
            let rows: std::collections::BTreeSet<usize> = (0..14)
                .filter(|&row| {
                    (0..BOSSES.len()).any(|column| {
                        cell_at_position(row, column)
                            .and_then(|i| Some((*before.get(i)?, *after.get(i)?)))
                            .is_some_and(|(was, now)| was == 0 && now != 0)
                    })
                })
                .collect();
            let mask = after.get(WINNER_MASK).copied().unwrap_or(0);
            let winners: Vec<usize> = (0..14).filter(|b| mask & (1 << b) != 0).collect();
            if rows.len() != 1 || winners.len() != 1 {
                continue;
            }
            let (row, winner) = (*rows.iter().next().unwrap_or(&0), winners[0]);
            assert_eq!(
                row, winner,
                "{before_name} → {after_name}: the mark that appeared belongs to row \
                 {row} ({}), but the winner mask names {winner} ({}). One of the two \
                 tables is off.",
                ROSTER[row].name, ROSTER[winner].name,
            );
            agreed += 1;
        }
    }
    // Two is the floor at which this property pins anything, and falling under it is a
    // statement about `samples/`, not about the tables — so it is **declared**, the way
    // the online bit's absence is below, and not asserted away. It used to be an
    // assertion, which was right while the only series walked was the 2026 one that
    // carries six qualifying windows: it could only go off on a machine that had that
    // series and had lost most of it. Now that a 638-era series is walked too, the same
    // assertion fires on a perfectly ordinary machine — this one, where the whole `rep_`
    // series offers exactly one window with a single winner and a single new mark. The
    // warning is what the skip keeps: on a series that does reach 2026, this line
    // appearing at all is the regression.
    if agreed < 2 {
        test_support::skip(&format!(
            "only {agreed} window(s) pin a base by identity across {} series: a base off \
             by one would not be noticed by this property here",
            SERIES.len()
        ));
    }
}

/// The converse of the property above, and the reason the columns are worth having: on
/// this profile the three of them are not empty. A base pointing at a run of cells that
/// never move would satisfy the property above trivially — this is what keeps it honest.
#[test]
fn the_three_located_columns_are_not_dead_cells() {
    for suffix in SERIES {
        let series = series_of(suffix);
        let Some((name, last)) = series.last() else {
            continue; // already declared on stderr
        };
        for (column, boss, kill_index) in KILLS {
            let started = (0..ROSTER.len())
                .filter_map(|row| cell_at_position(row, column))
                .filter(|&i| last.get(i).is_some_and(|&v| v != 0))
                .count();
            let kills = last.get(kill_index).copied().unwrap_or(0);
            assert!(
                started > 0,
                "{name}: no {boss} mark on the most recent save of the *.{suffix} \
                 series, yet the column is declared located"
            );
            assert!(
                started as u32 <= kills,
                "{name}: {boss}: {started} characters carry the mark but only {kills} \
                 kills are recorded at {kill_index}"
            );
            assert_eq!(
                BOSSES[column], boss,
                "the column order moved: this table names it by position"
            );
        }
    }
}

/// What the third bit is, kept answerable to the series.
///
/// Measured on 2026-09-12, on a matched window around a single online co-op run: Greed
/// Mode with Cain, won, and the cell `Greed × Cain` went 2 → 7. Across the dated series
/// every date on which any cell gained bit 2 has an `online_logs\` session of the same
/// day, and the ~60 marks taken on days without one gained bits 0 and 1 only. Local
/// co-op — recognisable because the winner mask names two characters at once — takes
/// marks the ordinary way and leaves bit 2 alone. So bit 2 reads **won online**.
///
/// The property below is the structural half of that reading, and the half a sample can
/// still check once the logs are gone: no located cell has ever held 4 or 6, so bit 2 has
/// never been seen standing without bit 0. If one ever does, "won online" is the wrong
/// name and the tooltip has to go back to saying so.
///
/// **The reason written here used to be "an online clear is also a clear, so bit 2 can
/// never stand without bit 0", and that reason was wrong** — corrected 2026-09-17 with
/// B58. It reads bit 0 as *the* cleared bit, and the 638-era series says it is not: four
/// located cells go **1 → 2** across it (`Isaac × Greed` and `Cain × Greed` on 2024-02-23,
/// `Isaac × TheLamb` and `BlueBaby × BossRush` on 2024-01-29), which is bit 0 going out as
/// bit 1 comes in. A mark is not a set of flags that accumulate; a higher value replaces a
/// lower one, and 2 without 0 is an ordinary cell, not a contradiction. So the absence of
/// 4 and 6 is an **observation held over every sample**, which is worth pinning, and not a
/// law derived from what the bits mean — nobody has measured what they mean outside Greed,
/// and B22 is where that is owed.
///
/// A series holding no bit 2 at all makes that property vacuous, and a vacuous property
/// reports coverage it does not have — so the absence is **declared**, not asserted away.
/// It is not a regression: bit 2 only exists from the era the profile first won a run
/// online, and a series that stops before it is early, not broken. The failure it used to
/// raise needed neither an empty `samples/` (early return) nor the full series (the bit is
/// there): only the state in between, one machine with some samples and none of that era,
/// which is every second machine. What the skip keeps is the warning — on a series that
/// does reach the era, this line appearing at all is the regression.
#[test]
fn the_online_bit_never_stands_without_the_first_level_bit() {
    let (mut checked, mut online, mut read) = (0, 0, 0);
    for suffix in SERIES {
        let series = series_of(suffix);
        read += series.len();
        for (name, values) in &series {
            for (row, character) in ROSTER.iter().map(|r| r.name).enumerate() {
                for (column, boss) in BOSSES.iter().enumerate() {
                    let Some(v) = cell_at_position(row, column).and_then(|i| values.get(i)) else {
                        continue;
                    };
                    checked += 1;
                    online += u32::from(v & 4 != 0);
                    assert!(
                        v & 4 == 0 || v & 1 != 0,
                        "{name}: {character} × {boss} holds {v}, which sets the online \
                         bit with bit 0 clear — a value of 4 or 6, which no sample has \
                         ever held. Either \"won online\" is the wrong name for bit 2, or \
                         the tables are addressing something that is not a mark.",
                    );
                }
            }
        }
    }
    if read == 0 {
        return; // `dated_series` has already said why on stderr
    }
    assert!(
        checked > 0,
        "no mark cell was read: the series is there but the tables no longer address it"
    );
    if online == 0 {
        test_support::skip(&format!(
            "no cell sets bit 2 in the {read} sample(s) across {} series: the online bit \
             held vacuously and checked nothing",
            SERIES.len()
        ));
    }
}

/// The measurement itself, kept reproducible: the run that named the bit.
///
/// On 2026-09-12 a matched window was taken either side of **one** online co-op run —
/// Greed Mode, Cain, won, the whole session inside the window (`samples/logs/
/// 20260912-greed-online-coop.log.txt` is that session's log). `Greed × Cain` went 2 → 7.
///
/// What this pins is not the pair of numbers — it is that the bit landed on **the cell the
/// run took and on no other**. A table whose base moved would light a neighbour on the same
/// day and the arithmetic of `no_mark_appears_without_a_kill_of_that_boss` would still work;
/// an identity does not survive it. It is the same argument as
/// `the_character_that_won_is_the_character_whose_mark_appeared`, applied to the bit rather
/// than to the mark.
///
/// **What it cannot pin** is the half of the reading that made it "won online" rather than
/// "won in co-op": that came from the dated series against the 22 folders under
/// `online_logs\`, and those folders are not in `samples/`. They are the live game's, they
/// rotate, and a test that read them would pass on one machine and skip on every other. The
/// series half that a sample *can* still carry is the test below.
#[test]
fn the_online_run_lit_the_cell_it_took_and_no_other() {
    let (Some(before), Some(after)) = (
        test_support::window_sample("20260912-pre.rep+persistentgamedata1.dat"),
        test_support::window_sample("20260912-post-online.rep+persistentgamedata1.dat"),
    ) else {
        return; // already declared on stderr
    };
    let (Some(before), Some(after)) = (counters(&before), counters(&after)) else {
        return;
    };

    let gained: Vec<(&str, &str)> = (0..ROSTER.len())
        .flat_map(|row| (0..BOSSES.len()).map(move |column| (row, column)))
        .filter(|&(row, column)| {
            cell_at_position(row, column)
                .and_then(|i| Some((*before.get(i)?, *after.get(i)?)))
                .is_some_and(|(was, now)| was & 4 == 0 && now & 4 != 0)
        })
        .map(|(row, column)| (ROSTER[row].name, BOSSES[column]))
        .collect();

    assert_eq!(
        gained,
        vec![("Cain", "Greed")],
        "the window holds one online Greed run with Cain: the online bit has to appear on \
         that cell and on nothing else"
    );
}

/// The online bit is a property of the **run**, not of the day — and that is what stops
/// "won online" from being a reading of something else entirely.
///
/// The correlation that named the bit (every date that gained one has a session folder of
/// the same day, 6 of 6) needs `online_logs\`, which no sample carries. What the series
/// carries on its own is the sharper half: a window in which one cell gained the bit and
/// another gained a mark **without** it. A day-shaped explanation — a patch, an era, a
/// setting left on — cannot produce that. 2026-08-31 is the case the entry was written
/// from: `Satan × Magdalene` 3 → 7 with the bit and `Greed × Magdalene` 0 → 3 without it,
/// same day and the same character.
///
/// Held over the whole series rather than pinned to that date: a fixture of an era belongs
/// in a file name, and this property holds whatever the profile does next. The series
/// offering no such window is a statement about `samples/`, so it is **declared** and not
/// asserted away — the bit only exists from the era the profile first won online.
#[test]
fn a_mark_taken_the_same_day_can_lack_the_online_bit() {
    let (mut lit, mut mixed, mut example) = (0, 0, String::new());
    for suffix in SERIES {
        let series = series_of(suffix);
        for pair in series.windows(2) {
            let (before_name, before) = &pair[0];
            let (after_name, after) = &pair[1];
            let moved = |keep: fn(u32, u32) -> bool| -> Vec<(&str, &str)> {
                (0..ROSTER.len())
                    .flat_map(|row| (0..BOSSES.len()).map(move |column| (row, column)))
                    .filter(|&(row, column)| {
                        cell_at_position(row, column)
                            .and_then(|i| Some((*before.get(i)?, *after.get(i)?)))
                            .is_some_and(|(was, now)| keep(was, now))
                    })
                    .map(|(row, column)| (ROSTER[row].name, BOSSES[column]))
                    .collect()
            };
            // The bit arriving on some cell, and a level arriving on another without it.
            let with = moved(|was, now| was & 4 == 0 && now & 4 != 0);
            if with.is_empty() {
                continue;
            }
            lit += 1;
            let without = moved(|was, now| was & 3 == 0 && now & 3 != 0 && now & 4 == 0);
            if without.is_empty() {
                continue;
            }
            mixed += 1;
            if example.is_empty() {
                example = format!(
                    "{before_name} → {after_name}: {with:?} with the bit, {without:?} \
                     without"
                );
            }
        }
    }
    if lit == 0 {
        test_support::skip(&format!(
            "no window across {} series lights the online bit: nothing here could tell a \
             property of the run from a property of the day",
            SERIES.len()
        ));
        return;
    }
    assert!(
        mixed > 0,
        "{lit} window(s) light the online bit and in every one of them **every** new mark \
         carries it. That is what a property of the *day* looks like — a patch, an era, a \
         setting left on — and \"won online\" would be the wrong name for it"
    );
    eprintln!("online bit is a property of the run: {example}");
}
