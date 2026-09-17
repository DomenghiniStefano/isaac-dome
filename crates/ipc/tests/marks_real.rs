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

use core_save::{Kind, Save};
use ipc::{counter_index, BOSSES, CHARACTERS};
use test_support::dated_series;

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
const SERIES: [&str; 2] = ["rep_persistentgamedata1.dat", "rep+persistentgamedata1.dat"];

/// The columns whose position was derived, and the counter that counts that boss's
/// kills. Only these three: the other nine columns have no single kill counter to check
/// against (Mom's Heart and Greed have none at all), and this test exists for the ones
/// the series itself located.
const KILLS: [(usize, &str, usize); 3] = [
    (9, "Delirium", 187),
    (10, "Mother", 491),
    (11, "The Beast", 492),
];

fn counters(path: &std::path::Path) -> Option<Vec<u32>> {
    Save::open(path).ok()?.u32s(Kind::Counters)
}

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
    for suffix in SERIES {
        let series = series_of(suffix);
        for pair in series.windows(2) {
            let (before_name, before) = &pair[0];
            let (after_name, after) = &pair[1];
            for (column, boss, kill_index) in KILLS {
                let appeared: Vec<&str> = (0..CHARACTERS.len())
                    .filter(|&row| {
                        counter_index(row, column)
                            .and_then(|i| Some((*before.get(i)?, *after.get(i)?)))
                            .is_some_and(|(was, now)| was == 0 && now != 0)
                    })
                    .map(|row| CHARACTERS[row].0)
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
                        counter_index(row, column)
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
                CHARACTERS[row].0, CHARACTERS[winner].0,
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
            let started = (0..CHARACTERS.len())
                .filter_map(|row| counter_index(row, column))
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
            for (row, (character, _)) in CHARACTERS.iter().enumerate() {
                for (column, boss) in BOSSES.iter().enumerate() {
                    let Some(v) = counter_index(row, column).and_then(|i| values.get(i)) else {
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
