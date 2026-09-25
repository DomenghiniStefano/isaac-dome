//! The preview, against the real series. Properties rather than values: the denominator is
//! read from the file, so it is the first thing a patch moves — a 2024 save declares 638
//! achievements and a 2025 one 641 (`docs/STATUS.md`), which is `of` 637 and 640 here.
//!
//! Each series is walked **on its own**. The two editions are two profiles, and a jump
//! between them would read as progress — the lesson `marks_real.rs` carries since B58.

use core_save::Save;
use ipc::{preview_of, CandidatePreview, PreviewCount};
use test_support::{dated_series, SERIES};

fn previews(suffix: &str) -> Vec<CandidatePreview> {
    dated_series(suffix)
        .iter()
        .filter_map(|p| Save::open(p).ok())
        .map(|s| preview_of(&s))
        .collect()
}

fn read(count: PreviewCount) -> Option<(u32, u32)> {
    match count {
        PreviewCount::Read { done, of } => Some((done, of)),
        PreviewCount::Unread => None,
    }
}

#[test]
fn no_count_ever_exceeds_what_the_file_declares() {
    let mut seen = 0;
    for suffix in SERIES {
        for p in previews(suffix) {
            for count in [p.achievements, p.items, p.marks] {
                if let Some((done, of)) = read(count) {
                    assert!(done <= of, "{done} of {of}");
                    seen += 1;
                }
            }
        }
    }
    if seen == 0 {
        test_support::skip("no dated series in samples/: nothing to hold the property over");
    }
}

#[test]
fn a_series_never_regresses_and_at_least_one_window_moves() {
    for suffix in SERIES {
        let dones: Vec<u32> = previews(suffix)
            .iter()
            .filter_map(|p| read(p.achievements))
            .map(|(done, _)| done)
            .collect();
        if dones.len() < 2 {
            test_support::skip(&format!(
                "{suffix}: fewer than two snapshots, no window to walk"
            ));
            continue;
        }
        for pair in dones.windows(2) {
            assert!(pair[1] >= pair[0], "{} then {}", pair[0], pair[1]);
        }
        // The vacuity guard: "never regresses" holds trivially over a flat series, and a
        // test that cannot fail reports coverage that is not there.
        assert!(
            dones.last() > dones.first(),
            "{suffix}: the series never moves, so the property held over nothing"
        );
    }
}

#[test]
fn the_two_editions_declare_different_totals_and_the_later_one_declares_more() {
    // **A series is not an era.** This test used to require one declared total per series, and
    // that is the very thing the module's own header says a patch moves: the June 2025 `rep+`
    // saves declare 641 and the 2026 ones 642 (`docs/save-format.md`). It held only while
    // `samples/` happened to stop short of the patch — a fixture of a machine, asserted as a
    // property. What holds over any series is that the declared total never *shrinks*: a patch
    // adds an achievement, it does not take one away.
    let declared = |suffix: &str| -> Option<u32> {
        let all: Vec<u32> = previews(suffix)
            .iter()
            .filter_map(|p| read(p.achievements))
            .map(|(_, of)| of)
            .collect();
        for pair in all.windows(2) {
            assert!(
                pair[1] >= pair[0],
                "{suffix}: declares {} and then {}",
                pair[0],
                pair[1]
            );
        }
        all.last().copied()
    };
    // The latest of each series, which is where the two editions are furthest apart and the
    // comparison this test exists for is sharpest.
    match (declared(SERIES[0]), declared(SERIES[1])) {
        (Some(rep), Some(rep_plus)) => assert!(
            rep_plus > rep,
            "Repentance+ declares {rep_plus}, Repentance {rep}"
        ),
        _ => test_support::skip("both series are needed to compare two eras"),
    }
}
