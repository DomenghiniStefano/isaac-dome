//! The logs are reached the same way the saves are: by name, declaring the outcome.

#[test]
fn a_named_log_is_found_or_skipped_with_a_note() {
    // Nothing on `None`: `log_sample` has already declared the skip, and saying it twice
    // counted it twice.
    if let Some(path) = test_support::log_sample("20260912-solo-judas.log.txt") {
        assert!(path.ends_with("20260912-solo-judas.log.txt"));
    }
}

/// A launch that produced no run is a log the game wrote and is **not** a run, so it lives in
/// `samples/launches/` — the separation `windows/` already makes for what is not a point in the
/// series. Whatever walks the run logs must not reach it: `samples/logs/` means "logs of runs",
/// and the guard that catches rules which stopped matching rests on that meaning (B60).
#[test]
fn a_launch_is_not_reachable_from_the_run_logs() {
    let launches = test_support::launch_samples();
    if launches.is_empty() {
        return; // `launch_samples` has already said so on stderr
    }
    let logs = test_support::log_samples();
    for launch in &launches {
        assert!(
            !logs.contains(launch),
            "{} is reachable from samples/logs/, where every log has to hold a run",
            launch.display()
        );
    }
}

#[test]
fn the_probe_files_are_not_logs() {
    // `samples/logs/` also holds `probe*.tsv` and `watch.log`, which are measurements and not
    // logs. A looser filter is exactly the mistake `is_dated` exists to prevent.
    let logs = test_support::log_samples();
    if logs.is_empty() {
        // Without this the loop passes on nothing and says nothing.
        test_support::skip("no run log in samples/logs/: no file name to check");
        return;
    }
    for path in logs {
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        assert!(name.ends_with(".log.txt"), "{name} is not a log");
    }
}
