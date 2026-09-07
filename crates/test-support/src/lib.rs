//! Helpers shared by the tests that read real data from `samples/`.
//!
//! It exists for one reason, and it comes from a real bug: the `unpack` tests were all
//! running on `config.a`, the only archive the decompressor knew how to open. No skip,
//! no red, main function broken. A test on real data has to say **which slice of the
//! domain it actually ran on**, not just that it passed.
//!
//! So every function in here always prints a line to stderr:
//!
//! ```text
//! sample: 20250626.rep+persistentgamedata1.dat
//! skip: 20240118.rep_persistentgamedata1.dat missing from samples/
//! ```
//!
//! `scripts/check` counts the two families and reports them at the end. Watch out:
//! `cargo test` **hides** the output of tests that pass, so those lines are only visible
//! with `cargo test -- --nocapture`, which is what the script runs.

use std::path::{Path, PathBuf};

/// The `samples/` folder at the repo root. It's ignored by git: whoever clones doesn't
/// have it, and the suite still has to come out green.
pub fn samples_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../samples")
}

/// The path to a sample, if it exists. Always declares the outcome: which file is being
/// used, or that it's being skipped because it's missing.
pub fn sample(name: &str) -> Option<PathBuf> {
    let path = samples_dir().join(name);
    if path.exists() {
        eprintln!("sample: {name}");
        return Some(path);
    }
    eprintln!("skip: {name} missing from samples/");
    None
}

/// The bytes of a sample, with the same declaration as [`sample`].
pub fn sample_bytes(name: &str) -> Option<Vec<u8>> {
    let path = sample(name)?;
    match std::fs::read(&path) {
        Ok(bytes) => Some(bytes),
        // The file is there but can't be read (permissions, disk): it's a skip, not a
        // red, but of a different kind — and it must be said, otherwise it looks like
        // the file was never there.
        Err(e) => {
            eprintln!("skip: {name} present but unreadable ({e})");
            None
        }
    }
}

/// Skips for a reason that isn't a missing file: a tool absent from the machine, a
/// junction that wasn't created. The text ends up in `scripts/check`'s summary.
pub fn skip(reason: &str) {
    eprintln!("skip: {reason}");
}

/// The `samples/packed` folder, if it exists: a junction to the installed game's
/// `resources\packed`. Without it, dozens of `unpack`, `catalog` and `ipc` tests have
/// nothing to run on — and that's exactly the case where they have to say so, not pass
/// silently.
pub fn packed_dir() -> Option<PathBuf> {
    let dir = samples_dir().join("packed");
    if dir.is_dir() {
        eprintln!("sample: packed/ (installed game archives)");
        return Some(dir);
    }
    skip("samples/packed missing (requires the game installation)");
    None
}

/// An archive inside `samples/packed`, declared by name. Used by tests that run against
/// a specific `.a`: which archive they touched is half the result, because for months
/// they all ran on `config.a` and nobody had noticed.
pub fn packed_file(name: &str) -> Option<PathBuf> {
    let path = samples_dir().join("packed").join(name);
    if path.is_file() {
        eprintln!("sample: packed/{name}");
        return Some(path);
    }
    skip(&format!(
        "samples/packed/{name} missing (requires the game installation)"
    ));
    None
}

/// A dated sample name: `YYYYMMDD.` followed by the requested suffix. This is the format
/// used in `samples/` — the numbers pinned in the tests are a fixture from a known era,
/// and a file named "live" invites overwriting it.
pub fn is_dated(name: &str, suffix: &str) -> bool {
    let Some(rest) = name.strip_suffix(suffix) else {
        return false;
    };
    let Some(date) = rest.strip_suffix('.') else {
        return false;
    };
    date.len() == 8 && date.bytes().all(|b| b.is_ascii_digit())
}

/// The historical series: all dated samples with that suffix, in chronological order.
///
/// Ordering by name **is** chronological order, because the name starts with
/// `YYYYMMDD`. Every file used is declared, one per line.
pub fn dated_series(suffix: &str) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(samples_dir()) else {
        skip(&format!("samples/ not readable (series {suffix})"));
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| is_dated(n, suffix))
        .collect();
    names.sort();
    if names.is_empty() {
        skip(&format!("no dated sample *.{suffix} in samples/"));
    }
    names
        .iter()
        .map(|n| {
            eprintln!("sample: {n}");
            samples_dir().join(n)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::is_dated;

    const SUFFIX: &str = "rep+persistentgamedata1.dat";

    #[test]
    fn a_dated_name_is_eight_digits_a_dot_and_the_suffix() {
        assert!(is_dated("20250626.rep+persistentgamedata1.dat", SUFFIX));
    }

    /// `live` is deliberately excluded: it's the file that gets overwritten without
    /// anyone noticing, so it's not part of a historical series.
    #[test]
    fn live_and_short_dates_are_not_part_of_the_series() {
        assert!(!is_dated("live.rep+persistentgamedata1.dat", SUFFIX));
        assert!(!is_dated("202506.rep+persistentgamedata1.dat", SUFFIX));
        assert!(!is_dated("2025062x.rep+persistentgamedata1.dat", SUFFIX));
    }

    /// Slot 2 is a different profile: same date, different series.
    #[test]
    fn another_slot_is_another_series() {
        assert!(!is_dated("20250626.rep+persistentgamedata2.dat", SUFFIX));
    }

    #[test]
    fn the_separating_dot_is_required() {
        assert!(!is_dated("20250626rep+persistentgamedata1.dat", SUFFIX));
    }
}
