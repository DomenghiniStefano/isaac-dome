use core_save::{Kind, Save};
use ipc::{profile_id, save_summary, SaveDiagnostic, SaveSummary, Settings};
use std::path::Path;
use test_support::sample;

/// The latest snapshot in the historical series under `samples/`. The samples are
/// named by date because the numbers pinned here are a fixture of known origin; the
/// first of the series, from a year earlier, serves the test that compares two eras.
const RECENTE: &str = "20260905.rep+persistentgamedata1.dat";
const ANNO_PRIMA: &str = "20250626.rep+persistentgamedata1.dat";

#[test]
fn summary_reports_the_counts_declared_by_the_file() {
    let Some(path) = sample(RECENTE) else { return };
    let bytes = std::fs::read(&path).expect("a present sample must be readable");
    let save = Save::parse(&bytes).unwrap();
    let summary = save_summary(&profile_id(&path), &save);

    assert_eq!(summary.sections.len(), 10, "ten sections");
    let counters = summary
        .sections
        .iter()
        .find(|s| s.kind == Kind::Counters)
        .unwrap();
    assert_eq!(
        counters.count,
        save.section(Kind::Counters).unwrap().count,
        "the count is read from the file, not hardcoded"
    );
    assert!(
        summary.diagnostics.is_empty(),
        "a real save produces no diagnostics"
    );
}

#[test]
fn summary_carries_no_raw_bytes() {
    let Some(path) = sample(RECENTE) else { return };
    let bytes = std::fs::read(&path).expect("a present sample must be readable");
    let save = Save::parse(&bytes).unwrap();
    let json = serde_json::to_string(&save_summary(&profile_id(&path), &save)).unwrap();
    assert!(
        !json.contains("bytes"),
        "raw bytes don't cross the IPC boundary"
    );
    assert!(
        !json.contains("offset"),
        "offsets don't cross the IPC boundary"
    );
    assert!(
        json.len() < 2_000,
        "the payload is small: {} bytes",
        json.len()
    );
}

#[test]
fn settings_round_trip_and_default() {
    let s = Settings::default();
    assert!(s.active_profile_id.is_none());
    let json = serde_json::to_string(&s).unwrap();
    let back: Settings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.active_profile_id, None);
}

#[test]
fn malformed_settings_are_not_an_error_for_the_caller() {
    assert!(serde_json::from_str::<Settings>("{ non è json").is_err());
    assert_eq!(
        serde_json::from_str::<Settings>("{}")
            .unwrap()
            .active_profile_id,
        None,
        "an empty file is equivalent to no choice"
    );
}

/// Pins `SaveDiagnostic`'s JSON tags and fields. No other test serializes it:
/// without this, removing `rename_all` wouldn't fail anything.
#[test]
fn save_diagnostic_json_tags_and_fields_are_pinned() {
    assert_eq!(
        serde_json::to_value(SaveDiagnostic::UnexpectedKind {
            expected: 2,
            found: 5,
        })
        .unwrap(),
        serde_json::json!({"kind": "unexpectedKind", "expected": 2, "found": 5})
    );
    assert_eq!(
        serde_json::to_value(SaveDiagnostic::SectionOverrun { section: 4 }).unwrap(),
        serde_json::json!({"kind": "sectionOverrun", "section": 4})
    );
    assert_eq!(
        serde_json::to_value(SaveDiagnostic::TrailingBytes).unwrap(),
        serde_json::json!({"kind": "trailingBytes"})
    );
}

/// Two snapshots a year apart declare a different number of achievements, because a
/// patch added one: this is the one test that proves in the field why hardcoded counts
/// are forbidden. Skips with a note if a sample is missing, like the other tests
/// against real data.
#[test]
fn summary_reports_the_counts_each_snapshot_declares_and_they_differ() {
    let (Some(recente_path), Some(prima_path)) = (sample(RECENTE), sample(ANNO_PRIMA)) else {
        return;
    };
    let leggi = |p: &Path| std::fs::read(p).expect("a present sample must be readable");
    let (recente_bytes, prima_bytes) = (leggi(&recente_path), leggi(&prima_path));

    let summary_of = |path: &Path, bytes: &[u8]| -> SaveSummary {
        let save = Save::parse(bytes).unwrap();
        let summary = save_summary(&profile_id(path), &save);
        summary.sections.iter().for_each(|s| {
            assert_eq!(
                s.count,
                save.section(s.kind).unwrap().count,
                "{:?}: the count is read from the file, not hardcoded",
                s.kind
            );
        });
        summary
    };

    let recente = summary_of(&recente_path, &recente_bytes);
    let prima = summary_of(&prima_path, &prima_bytes);

    let count_of = |summary: &SaveSummary, kind: Kind| {
        summary
            .sections
            .iter()
            .find(|s| s.kind == kind)
            .unwrap()
            .count
    };

    assert_ne!(
        count_of(&recente, Kind::Achievements),
        count_of(&prima, Kind::Achievements),
        "between the two snapshots a patch added an achievement"
    );
    // The counters, on the other hand, haven't changed between these two dates: that's
    // the whole point of the rule, not an exception. The number is read from each
    // file, and whether it's the same or different is discovered, never assumed.
    assert_eq!(
        count_of(&recente, Kind::Counters),
        count_of(&prima, Kind::Counters),
        "the counters are the same in both snapshots"
    );
}
