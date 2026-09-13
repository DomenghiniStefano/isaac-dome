//! The logs are reached the same way the saves are: by name, declaring the outcome.

#[test]
fn a_named_log_is_found_or_skipped_with_a_note() {
    match test_support::log_sample("20260912-solo-judas.log.txt") {
        Some(path) => assert!(path.ends_with("20260912-solo-judas.log.txt")),
        None => test_support::skip("20260912-solo-judas.log.txt missing from samples/logs/"),
    }
}

#[test]
fn the_probe_files_are_not_logs() {
    // `samples/logs/` also holds `probe*.tsv` and `watch.log`, which are measurements and not
    // logs. A looser filter is exactly the mistake `is_dated` exists to prevent.
    for path in test_support::log_samples() {
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        assert!(name.ends_with(".log.txt"), "{name} is not a log");
    }
}
