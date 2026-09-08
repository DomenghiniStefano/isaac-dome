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
//!
//! The counting doesn't read that stderr, though — see [`DECLARATIONS_VAR`]. Two streams
//! merged into one file can split a line down the middle, and the summary that tells you
//! how much a run *didn't* verify is worth nothing if its own numbers are approximate.

use std::io::Write;
use std::path::{Path, PathBuf};

/// When the runner sets this to a path, every declaration is mirrored into that file as
/// well as printed. `scripts/check` sets it, empties the file first, and counts from
/// there; `cargo test` on its own leaves it unset and nothing is written.
///
/// It exists because **stderr cannot be counted**. The harness writes `test … ok` to
/// stdout while these lines go to stderr, so merging the two with `2>&1` can split a
/// declaration across the middle of a harness line (`skip: ok`, with the rest orphaned);
/// and the tests inside one binary run in parallel, so they interleave with each other
/// too. Either way `grep -c '^skip:'` is noise, which is how a summary meant to say
/// "this run verified less than it looks" ended up saying a number nobody could trust.
/// A file opened in append mode has neither problem: one short atomic write per line.
pub const DECLARATIONS_VAR: &str = "ISAACDOME_TEST_DECLARATIONS";

/// Says a line on stderr, for whoever is reading the run, and mirrors it into the
/// declarations file, for whatever is counting.
fn declare(line: &str) {
    eprintln!("{line}");
    let Ok(path) = std::env::var(DECLARATIONS_VAR) else {
        return;
    };
    // A mirror that can't be written must not fail a test: the declaration is already on
    // stderr, and the summary reports how many it managed to count.
    let _ = append_line(Path::new(&path), line);
}

/// Appends one line. Two details, both of which cost a wrong summary before they were
/// there:
///
/// **Append mode, never truncate.** Every test binary of a run writes to the same file;
/// a truncating open would leave only whatever spoke last.
///
/// **The newline goes in the same buffer as the text, written once.** `writeln!` emits
/// them as two separate writes, and another writer landing in between fuses two
/// declarations into one line — `sample: betaskip: alpha`, counted once instead of twice.
/// One `write_all` of a short buffer in append mode is a single atomic write.
fn append_line(path: &Path, line: &str) -> std::io::Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    file.write_all(format!("{line}\n").as_bytes())
}

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
        declare(&format!("sample: {name}"));
        return Some(path);
    }
    declare(&format!("skip: {name} missing from samples/"));
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
            declare(&format!("skip: {name} present but unreadable ({e})"));
            None
        }
    }
}

/// Skips for a reason that isn't a missing file: a tool absent from the machine, a
/// junction that wasn't created. The text ends up in `scripts/check`'s summary.
pub fn skip(reason: &str) {
    declare(&format!("skip: {reason}"));
}

/// The `samples/packed` folder, if it exists: a junction to the installed game's
/// `resources\packed`. Without it, dozens of `unpack`, `catalog` and `ipc` tests have
/// nothing to run on — and that's exactly the case where they have to say so, not pass
/// silently.
pub fn packed_dir() -> Option<PathBuf> {
    let dir = samples_dir().join("packed");
    if dir.is_dir() {
        declare("sample: packed/ (installed game archives)");
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
        declare(&format!("sample: packed/{name}"));
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
            declare(&format!("sample: {n}"));
            samples_dir().join(n)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{append_line, is_dated};

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

    /// The mirror appends: every declaration of a run has to end up in the file, in the
    /// order it was made. Truncating instead would leave the summary counting only
    /// whatever spoke last.
    #[test]
    fn the_mirror_appends_instead_of_replacing() {
        let path = std::env::temp_dir().join("isaacdome-declarations-append.txt");
        let _ = std::fs::remove_file(&path);
        append_line(&path, "sample: one").expect("first write");
        append_line(&path, "skip: two").expect("second write");
        let written = std::fs::read_to_string(&path).expect("the file is there");
        assert_eq!(written, "sample: one\nskip: two\n");
        let _ = std::fs::remove_file(&path);
    }

    /// Every declaration of a run reaches the same file from many threads and many test
    /// binaries at once, so a line has to be written **whole or not at all**. It isn't
    /// theory: `writeln!` emits the text and the newline as two separate writes, and
    /// another writer slipping between them produced real summary lines like
    /// `skip: …installation)skip: …installation)` — two declarations fused, counted once.
    #[test]
    fn concurrent_appends_never_fuse_two_lines() {
        let path = std::env::temp_dir().join("isaacdome-declarations-concurrent.txt");
        let _ = std::fs::remove_file(&path);
        let expected = ["skip: alpha", "sample: beta", "skip: gamma-is-longer"];
        let target = path.as_path();
        std::thread::scope(|s| {
            for line in expected {
                s.spawn(move || {
                    for _ in 0..200 {
                        append_line(target, line).expect("append");
                    }
                });
            }
        });
        let written = std::fs::read_to_string(&path).expect("the file is there");
        let lines: Vec<&str> = written.lines().collect();
        assert_eq!(lines.len(), 600, "one line per append, none fused or lost");
        assert!(
            lines.iter().all(|l| expected.contains(l)),
            "a line came out mangled: {:?}",
            lines.iter().find(|l| !expected.contains(l))
        );
        let _ = std::fs::remove_file(&path);
    }

    /// A mirror that can't be written is not worth failing a test over: the declaration
    /// is already on stderr, and the summary reports how many it managed to count.
    #[test]
    fn a_mirror_that_cannot_be_written_is_not_an_error_for_the_caller() {
        let unwritable = std::env::temp_dir()
            .join("no-such-dir-isaacdome")
            .join("x.txt");
        assert!(append_line(&unwritable, "skip: nowhere").is_err());
    }
}
