use core_save::{diff, Kind, Save};
use test_support::{dated_series, sample_bytes};

/// The historical series' suffix: Repentance+ profile, slot 1. The files come from the
/// dated backups the game leaves in `save_backups\`, copied into `samples/` under the
/// name they already had — `YYYYMMDD.` plus this suffix.
const SERIE: &str = "rep+persistentgamedata1.dat";

/// The series' first snapshot and its last. Used by the tests that compare two eras
/// of the game: between the two, a patch added an achievement.
const JUN_2025: &str = "20250626.rep+persistentgamedata1.dat";
const SEP_2026: &str = "20260905.rep+persistentgamedata1.dat";

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
    ("20240118.rep_persistentgamedata1.dat", 638, 496),
    // Repentance+, January 2025: the achievements are already 641, the counters not yet 523.
    ("20250112.rep+persistentgamedata1.dat", 641, 521),
    // Repentance+ 2026, the era the table in `CLAUDE.md` describes.
    (SEP_2026, 642, 523),
];

/// The historical series, already parsed, in chronological order. Empty if `samples/`
/// contains none: the folder is ignored by git, so whoever clones the repo has none.
fn series() -> Vec<(String, Save)> {
    dated_series(SERIE)
        .into_iter()
        .map(|p| {
            let name = p
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            let bytes = std::fs::read(&p).expect("a sample that is present must read");
            (name, Save::parse(&bytes).expect("a real sample must parse"))
        })
        .collect()
}

#[test]
fn every_real_save_has_ten_sections_in_order() {
    let saves = series();
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

/// The achievement count is read from the file. This isn't a theoretical precaution: between
/// 2025 and 2026 a patch added one, and any hardcoded number would have broken on its own.
/// Both values were verified by reading the header byte by byte
/// (`od -j 16 -t u4`), not from this parser's output.
#[test]
fn achievement_count_is_read_from_file_not_hardcoded() {
    let (Some(earlier), Some(later)) = (sample_bytes(JUN_2025), sample_bytes(SEP_2026)) else {
        return;
    };
    let earlier = Save::parse(&earlier).unwrap();
    let later = Save::parse(&later).unwrap();
    assert_eq!(earlier.section(Kind::Achievements).unwrap().count, 641);
    assert_eq!(later.section(Kind::Achievements).unwrap().count, 642);
}

#[test]
fn some_counts_do_not_change_with_the_game_version() {
    let saves = series();
    if saves.is_empty() {
        return;
    }
    saves.iter().for_each(|(name, save)| {
        let count = |k: Kind| save.section(k).unwrap().count;
        assert_eq!(count(Kind::PerChar), 14, "{name}");
        assert_eq!(count(Kind::Items), 733, "{name}");
        assert_eq!(count(Kind::Unknown5), 7, "{name}");
        assert_eq!(count(Kind::CardsPills), 104, "{name}");
        assert_eq!(count(Kind::Challenges), 46, "{name}");
        assert_eq!(count(Kind::Unknown8), 27, "{name}");
        assert_eq!(count(Kind::Unknown9), 2, "{name}");
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
    let saves = series();
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
    let saves = series();
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

#[test]
fn the_last_section_reaches_exactly_the_checksum() {
    let Some(bytes) = sample_bytes(SEP_2026) else {
        return;
    };
    let save = Save::parse(&bytes).unwrap();
    let bestiary = save.section(Kind::Bestiary).unwrap();
    let end = bestiary.offset + bestiary.bytes.len();
    assert_eq!(
        end,
        bytes.len() - 4,
        "no leftover bytes before the checksum"
    );
}

/// The diff reports what changed between two snapshots, and on a real historical series
/// it's verified without pinning numbers: the indices it reports are exactly the ones that
/// flipped from off to on, derived from the two sections rather than from the diff itself.
#[test]
fn diff_reports_exactly_the_bits_that_turned_on() {
    let saves = series();
    if saves.len() < 2 {
        return;
    }
    saves.windows(2).for_each(|w| {
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
    let saves = series();
    if saves.len() < 2 {
        return;
    }
    saves.windows(2).for_each(|w| {
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

#[test]
fn diff_of_a_save_with_itself_is_empty() {
    let Some(bytes) = sample_bytes(SEP_2026) else {
        return;
    };
    let save = Save::parse(&bytes).unwrap();
    assert_eq!(diff(&save, &save), core_save::SaveDiff::default());
}
