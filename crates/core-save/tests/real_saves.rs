use core_save::{diff, Kind, Save};
use test_support::{dated_series, sample_bytes};

/// The historical series in `samples/`, one suffix per edition, slot 1. The files come
/// from the dated backups the game leaves in `save_backups\`, copied under the name they
/// already had — `YYYYMMDD.` plus one of these suffixes.
///
/// There are two because a comparison only means something **inside** one profile:
/// `rep_` snapshots are a Repentance profile, `rep+` a Repentance+ one, and laying them
/// end to end would read a change of profile as progress. Which is also why this is a
/// list and not a single constant: pinned to `rep+` alone, the three dated Repentance
/// saves sitting in `samples/` were read by nothing at all.
const SERIES: [&str; 2] = ["rep_persistentgamedata1.dat", "rep+persistentgamedata1.dat"];

/// One file per era, named so the tests that compare two of them say which.
const REP_2024: &str = "20240118.rep_persistentgamedata1.dat";
const REP_PLUS_2025: &str = "20250112.rep+persistentgamedata1.dat";
const REP_PLUS_2026: &str = "20260905.rep+persistentgamedata1.dat";

/// The eras `samples/` can hold, and the counts each one's header declares:
/// `(file, achievements, counters)`. Every number here was read with
/// `od -A d -j <section offset> -N 12 -t u4` on the header, never taken from this
/// parser's output — an expectation derived from the code under test proves nothing.
///
/// This is the **only** place a count is pinned, and it is pinned per file. It must
/// never move inside the loop over the series: the series spans eras by construction,
/// so an era's value asserted in there turns "samples/ holds a save from a different
/// era" into a red. That is not a hypothesis — it is what happened when the 2026
/// snapshots left this machine and a January 2025 profile, which declares 521 counters,
/// was the only file left for an assertion that demanded 523.
const ERAS: [(&str, u32, u32); 3] = [
    // Repentance, before the + edition.
    (REP_2024, 638, 496),
    // Repentance+, January 2025: the achievements are already 641, the counters not yet 523.
    (REP_PLUS_2025, 641, 521),
    // Repentance+ 2026, the era the table in `CLAUDE.md` describes.
    (REP_PLUS_2026, 642, 523),
];

/// One series, already parsed, in chronological order — name, raw bytes, parsed save.
/// Empty if `samples/` holds none: the folder is ignored by git, so whoever clones the
/// repo has none.
fn series(suffix: &str) -> Vec<(String, Vec<u8>, Save)> {
    dated_series(suffix)
        .into_iter()
        .map(|p| {
            let name = p
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            let bytes = std::fs::read(&p).expect("a sample that is present must read");
            let save = Save::parse(&bytes).expect("a real sample must parse");
            (name, bytes, save)
        })
        .collect()
}

/// Every dated save present, from every series. For the properties that hold of a save
/// on its own, where which profile it came from doesn't enter into it.
fn every_dated_save() -> Vec<(String, Save)> {
    SERIES
        .iter()
        .flat_map(|s| series(s))
        .map(|(name, _, save)| (name, save))
        .collect()
}

/// The same, keeping the raw bytes: for the properties stated in terms of the file's own
/// length, where the parsed view alone can't answer.
fn every_dated_save_with_bytes() -> Vec<(String, Vec<u8>, Save)> {
    SERIES.iter().flat_map(|s| series(s)).collect()
}

/// The series that can actually answer a question about *change*, one entry each:
/// comparing two snapshots needs two of them, from the same profile.
///
/// A series of one isn't a series, and the shortfall is invisible from the outside —
/// `dated_series` prints `sample:` for the file it found, the test walks a `windows(2)`
/// that yields nothing, and the run reports a pass. That's the failure mode
/// `test-support` exists to prevent, one level up: the helper declared which file it
/// used, while the test quietly stopped verifying anything. So it gets said out loud,
/// the way `graph`'s `series_evals` already says it — and per suffix, because "one
/// series is long enough" must not cover for the other being empty.
fn comparable_series() -> Vec<Vec<(String, Save)>> {
    SERIES
        .iter()
        .filter_map(|suffix| {
            let saves = series(suffix);
            if saves.len() < 2 {
                test_support::skip(&format!(
                    "the *.{suffix} series has {}: comparing two snapshots needs two",
                    saves.len()
                ));
                return None;
            }
            Some(
                saves
                    .into_iter()
                    .map(|(name, _, save)| (name, save))
                    .collect(),
            )
        })
        .collect()
}

#[test]
fn every_real_save_has_ten_sections_in_order() {
    let saves = every_dated_save();
    if saves.is_empty() {
        return;
    }
    saves.iter().for_each(|(name, save)| {
        let kinds: Vec<u32> = save.sections.iter().map(|s| s.kind.number()).collect();
        assert_eq!(kinds, (1..=10).collect::<Vec<_>>(), "{name}");
        assert!(
            save.diagnostics.is_empty(),
            "{name}: a real save must not produce diagnostics: {:?}",
            save.diagnostics
        );
    });
}

/// The achievement count is read from the file. This isn't a theoretical precaution: a
/// patch has added slots more than once, and any hardcoded number would have broken on
/// its own.
///
/// What proves it is **two eras that disagree**, and any two will do. This pair used to
/// be 641 → 642, the newest jump, and that cost the test its data: neither snapshot is
/// on every machine, so it skipped every run and proved nothing. 638 → 641 asks the same
/// question of files that are actually here. Both numbers read from the header with
/// `od -A d -j 16 -N 12 -t u4`, not from this parser's output.
#[test]
fn achievement_count_is_read_from_file_not_hardcoded() {
    let (Some(earlier), Some(later)) = (sample_bytes(REP_2024), sample_bytes(REP_PLUS_2025)) else {
        return;
    };
    let earlier = Save::parse(&earlier).unwrap();
    let later = Save::parse(&later).unwrap();
    let count = |s: &Save| s.section(Kind::Achievements).unwrap().count;
    assert_eq!(count(&earlier), 638, "{REP_2024}");
    assert_eq!(count(&later), 641, "{REP_PLUS_2025}");
    assert_ne!(
        count(&earlier),
        count(&later),
        "two eras that agree would prove nothing about where the number comes from"
    );
}

#[test]
fn some_counts_do_not_change_with_the_game_version() {
    let saves = every_dated_save();
    if saves.is_empty() {
        return;
    }
    saves.iter().for_each(|(name, save)| {
        let count = |k: Kind| save.section(k).unwrap().count;
        assert_eq!(count(Kind::LevelCounters), 14, "{name}");
        assert_eq!(count(Kind::Items), 733, "{name}");
        assert_eq!(count(Kind::Unknown5), 7, "{name}");
        assert_eq!(count(Kind::Bosses), 104, "{name}");
        assert_eq!(count(Kind::Challenges), 46, "{name}");
        assert_eq!(count(Kind::Unknown8), 27, "{name}");
        assert_eq!(count(Kind::Unknown9), 2, "{name}");
    });
}

/// Section 3 is a table of **stages**, and "one value per original character" is refuted
/// by index 0.
///
/// A per-character table's first row is Isaac — the character everyone starts with and
/// plays most. On a profile with hundreds of runs it cannot be zero. It is zero in every
/// save we hold, in both editions, while its neighbours carry hundreds; the game numbers
/// stages from 1 in `Level::Init m_Stage`, so index 0 is the slot that numbering leaves
/// unused.
///
/// Stated as a property, not as the twelve values one profile happened to have: this is
/// what earns the section its name, and it has to keep holding on saves nobody has
/// collected yet. The other half of the evidence — a matched window in which the indices
/// that moved are exactly the stages the log declared — needs a live window and can't be
/// run from a sample; it is written up in `docs/BACKLOG.md`, B9.
#[test]
fn stage_counters_leave_index_0_unused() {
    let saves = every_dated_save();
    if saves.is_empty() {
        return;
    }
    saves.iter().for_each(|(name, save)| {
        let values = save
            .u32s(Kind::LevelCounters)
            .expect("section 3 is present");
        assert_eq!(
            values[0], 0,
            "{name}: index 0 carries a count, which a table of stages numbered from 1 never does"
        );
        assert!(
            values[1..].iter().any(|&v| v > 0),
            "{name}: every cell is zero, so index 0 being zero proves nothing"
        );
    });
}

/// Counts are a fixture of an era, so each one is pinned next to the file that produced
/// it, in [`ERAS`]. A sample that isn't there declares its own skip and takes its row
/// with it — an era we have no save for is missing coverage, never a failure.
#[test]
fn each_era_declares_its_own_counts() {
    ERAS.iter().for_each(|&(name, achievements, counters)| {
        let Some(bytes) = sample_bytes(name) else {
            return;
        };
        let save = Save::parse(&bytes).expect("a real sample must parse");
        assert_eq!(
            save.section(Kind::Achievements).unwrap().count,
            achievements,
            "{name}: achievements"
        );
        assert_eq!(
            save.section(Kind::Counters).unwrap().count,
            counters,
            "{name}: counters"
        );
    });
}

/// The era-independent half of the same subject: however many counters the header
/// declares, that's how many the parser hands back. True in every edition, so it belongs
/// in the loop over the series — unlike the number itself, which lives in [`ERAS`].
#[test]
fn counters_length_follows_the_header() {
    let saves = every_dated_save();
    if saves.is_empty() {
        return;
    }
    saves.iter().for_each(|(name, save)| {
        let declared = save.section(Kind::Counters).unwrap().count as usize;
        assert_eq!(save.u32s(Kind::Counters).unwrap().len(), declared, "{name}");
    });
}

#[test]
fn bestiary_length_follows_bytes_not_header_count() {
    let saves = every_dated_save();
    if saves.is_empty() {
        return;
    }
    saves.iter().for_each(|(name, save)| {
        let bestiary = save.section(Kind::Bestiary).unwrap();
        assert!(
            bestiary.bytes.len().is_multiple_of(8),
            "{name}: 8-byte records"
        );
        assert!(
            bestiary.bytes.len() as u32 / 8 != bestiary.count,
            "{name}: the header's count is not the number of records"
        );
    });
}

/// The file has no slack: the last section ends exactly where the checksum's four bytes
/// begin. A property of the format, so it's asked of every save present rather than of
/// one named file — which is also what keeps it running when that file isn't there.
#[test]
fn the_last_section_reaches_exactly_the_checksum() {
    every_dated_save_with_bytes()
        .iter()
        .for_each(|(name, bytes, save)| {
            let bestiary = save.section(Kind::Bestiary).unwrap();
            let end = bestiary.offset + bestiary.bytes.len();
            assert_eq!(
                end,
                bytes.len() - 4,
                "{name}: no leftover bytes before the checksum"
            );
        });
}

/// The diff reports what changed between two snapshots, and on a real historical series
/// it's verified without pinning numbers: the indices it reports are exactly the ones that
/// flipped from off to on, derived from the two sections rather than from the diff itself.
#[test]
fn diff_reports_exactly_the_bits_that_turned_on() {
    comparable_series()
        .iter()
        .flat_map(|s| s.windows(2))
        .for_each(|w| {
            let (before_name, before) = &w[0];
            let (after_name, after) = &w[1];
            let d = diff(before, after);
            let (Some(a), Some(b)) = (
                before.flags(Kind::Achievements),
                after.flags(Kind::Achievements),
            ) else {
                return;
            };
            // We walk the whole of `b`, not just the common part: a slot that didn't exist in
            // the first snapshot is off, not "outside the comparison". This is the real case
            // of the 641 → 642 slot transition, where the new achievement shows up as
            // unlocked right away and must appear in the diff.
            let expected: Vec<usize> = (0..b.len())
                .filter(|&i| b[i] && !a.get(i).copied().unwrap_or(false))
                .collect();
            assert_eq!(
                d.achievements, expected,
                "{before_name} → {after_name}: indices turned on between the two snapshots"
            );
            assert!(
                d.achievements.windows(2).all(|w| w[0] < w[1]),
                "{before_name} → {after_name}: indices ordered and distinct"
            );
        });
}

/// The property the self-updating plan rests on: progression never regresses. An
/// unlocked achievement stays unlocked, and sections never shrink — at most they
/// grow, as when a patch adds a slot.
#[test]
fn the_series_never_regresses() {
    comparable_series()
        .iter()
        .flat_map(|s| s.windows(2))
        .for_each(|w| {
            let (before_name, before) = &w[0];
            let (after_name, after) = &w[1];
            let (Some(a), Some(b)) = (
                before.flags(Kind::Achievements),
                after.flags(Kind::Achievements),
            ) else {
                return;
            };
            assert!(
                b.len() >= a.len(),
                "{before_name} → {after_name}: slots don't disappear"
            );
            (0..a.len()).for_each(|i| {
                assert!(
                    !a[i] || b[i],
                    "{before_name} → {after_name}: achievement {i} became re-locked"
                );
            });
            assert!(
                after.sections.len() >= before.sections.len(),
                "{before_name} → {after_name}: sections don't disappear"
            );
        });
}

/// Nothing changed between a save and itself. True of every save, so every save present
/// gets asked.
#[test]
fn diff_of_a_save_with_itself_is_empty() {
    every_dated_save().iter().for_each(|(name, save)| {
        assert_eq!(diff(save, save), core_save::SaveDiff::default(), "{name}");
    });
}
