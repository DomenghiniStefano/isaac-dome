//! Which folders under `online_logs\` hold a run. The layout is measured, not assumed: see the
//! module's own doc comment for what it looks like on a real machine.

use std::fs;
use std::path::Path;

fn session(root: &Path, rel: &str) {
    let dir = root.join(rel);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("log.txt"),
        b"[INFO] - RNG Start Seed: AAA AAA (1) [New, 1]",
    )
    .unwrap();
}

fn names(found: &[std::path::PathBuf]) -> Vec<String> {
    let mut n: Vec<String> = found
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    n.sort();
    n
}

#[test]
fn the_sessions_folder_is_where_the_sessions_are() {
    let tmp = tempfile::tempdir().unwrap();
    session(tmp.path(), "sessions/09_12_2026__13_34_26");
    session(tmp.path(), "sessions/09_13_2026__14_48_32");

    let found = log_watch::sessions(tmp.path());
    assert_eq!(
        names(&found),
        vec!["09_12_2026__13_34_26", "09_13_2026__14_48_32"]
    );
}

#[test]
fn the_two_sessions_nested_under_desyncs_are_found_too() {
    // Measured on this machine: `online_logs\desyncs\sessions\` holds two real sessions from
    // 2026-06-29. A walk that stops at `sessions\` loses them.
    let tmp = tempfile::tempdir().unwrap();
    session(tmp.path(), "sessions/09_12_2026__13_34_26");
    session(tmp.path(), "desyncs/sessions/06_29_2026__18_40_54");

    assert_eq!(
        names(&log_watch::sessions(tmp.path())),
        vec!["06_29_2026__18_40_54", "09_12_2026__13_34_26"]
    );
}

#[test]
fn a_desync_report_is_not_a_session() {
    // `desync_log.txt` is a different file with a different shape. Ingesting one would put a
    // crash report in the run archive.
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("desyncs/09_13_2026__14_48_32__Cupola");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("desync_log.txt"), b"[INFO] - desync").unwrap();

    assert!(log_watch::sessions(tmp.path()).is_empty());
}

#[test]
fn a_session_folder_without_a_log_is_not_a_session() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("sessions/09_13_2026__15_32_22");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("sharedsave_end.dat"), b"not a log").unwrap();

    assert!(log_watch::sessions(tmp.path()).is_empty());
}

#[test]
fn a_folder_that_is_not_there_is_no_sessions_and_no_panic() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(log_watch::sessions(&tmp.path().join("nothing here")).is_empty());
}
