//! What the IPC says about a profile nobody ever played (B61).
//!
//! The app has to open at a stranger's house on the evening they install the game, and until this
//! file there was no sample of that state: every profile in `samples/` has progress on it, because
//! every one of them came from somebody playing.

use core_save::Save;
use ipc::{profile_id, save_summary};

/// An untouched profile is **described, not degraded**: the summary reports all ten sections with
/// the counts the file declares, and **no diagnostic at all**.
///
/// That distinction is the whole point. "Degrade, never fail" means a profile the app cannot read
/// still produces a screen with a reason on it — and a brand-new profile is not that case. If a
/// diagnostic ever appears here, the first player of the evening is being shown a fault where
/// there is only an empty file.
#[test]
fn an_untouched_profile_is_described_and_not_diagnosed() {
    for path in test_support::empty_profile_samples() {
        let bytes = std::fs::read(&path).expect("a sample that is present must read");
        let save = Save::parse(&bytes).expect("a real sample must parse");
        let summary = save_summary(&profile_id(&path), &save);

        assert_eq!(summary.sections.len(), 10, "{}", path.display());
        assert!(
            summary.diagnostics.is_empty(),
            "{} is a profile nobody played, and the app reports {:?} about it",
            path.display(),
            summary.diagnostics
        );
        assert!(
            summary.sections.iter().all(|s| s.count > 0),
            "{}: a section declares no entries, which is a short file and not an empty profile",
            path.display()
        );
    }
}

/// The summary crosses the IPC, so what the frontend actually receives has to be checked as JSON
/// and not as a Rust value — an empty profile is exactly where a field could quietly serialize to
/// nothing and nobody would look.
#[test]
fn the_untouched_profile_crosses_the_ipc_whole() {
    for path in test_support::empty_profile_samples() {
        let bytes = std::fs::read(&path).expect("a sample that is present must read");
        let save = Save::parse(&bytes).expect("a real sample must parse");
        let json = serde_json::to_value(save_summary(&profile_id(&path), &save))
            .expect("a summary serializes");

        assert!(json["profile"].is_string(), "{json}");
        assert_eq!(
            json["sections"].as_array().map(|s| s.len()),
            Some(10),
            "{json}"
        );
        assert_eq!(json["diagnostics"].as_array().map(|d| d.len()), Some(0));
        let counts: Vec<u64> = json["sections"]
            .as_array()
            .expect("sections is an array")
            .iter()
            .filter_map(|s| s["count"].as_u64())
            .collect();
        assert_eq!(counts.len(), 10, "every section carries a numeric count");
    }
}
