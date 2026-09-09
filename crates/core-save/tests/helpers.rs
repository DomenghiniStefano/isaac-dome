//! Shared access to the dated samples, for every real-data test in this crate.
//!
//! Extracted when the bestiary tests arrived: the alternative was a second copy of
//! `series`, and a helper that declares on stderr which file it used is exactly the
//! thing that must not exist twice with two behaviours.

#![allow(dead_code)]

use core_save::Save;
use test_support::dated_series;

/// The historical series in `samples/`, one suffix per edition, slot 1. The files come
/// from the dated backups the game leaves in `save_backups\`, copied under the name they
/// already had — `YYYYMMDD.` plus one of these suffixes.
///
/// There are two because a comparison only means something **inside** one profile:
/// `rep_` snapshots are a Repentance profile, `rep+` a Repentance+ one, and laying them
/// end to end would read a change of profile as progress. Which is also why this is a
/// list and not a single constant: pinned to `rep+` alone, the three dated Repentance
/// saves sitting in `samples/` were read by nothing at all.
pub const SERIES: [&str; 2] = ["rep_persistentgamedata1.dat", "rep+persistentgamedata1.dat"];

/// One series, already parsed, in chronological order — name, raw bytes, parsed save.
/// Empty if `samples/` holds none: the folder is ignored by git, so whoever clones the
/// repo has none.
pub fn series(suffix: &str) -> Vec<(String, Vec<u8>, Save)> {
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
pub fn every_dated_save() -> Vec<(String, Save)> {
    SERIES
        .iter()
        .flat_map(|s| series(s))
        .map(|(name, _, save)| (name, save))
        .collect()
}

/// The same, keeping the raw bytes: for the properties stated in terms of the file's own
/// length, where the parsed view alone can't answer.
pub fn every_dated_save_with_bytes() -> Vec<(String, Vec<u8>, Save)> {
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
pub fn comparable_series() -> Vec<Vec<(String, Save)>> {
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
