//! View-model for the wiki dataset: the state of the embedded dataset (loaded or not,
//! and how up to date it is against the game installation) and the model types that
//! cross the IPC boundary unchanged — a single set of names shared between `wiki` and
//! the frontend.

use serde::Serialize;

pub use wiki::{Block, Dlc, Entry, Infobox, Inline, ListItem, Section, SectionKind, Style, Target};
use wiki::{Dataset, DatasetError};

/// A game patch, as the wiki knows it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchView {
    pub number: String,
    pub date: String,
}

/// Per-type entry counts in the embedded dataset.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WikiCounts {
    pub items: u32,
    pub trinkets: u32,
    pub achievements: u32,
    pub bosses: u32,
    pub challenges: u32,
    pub characters: u32,
}

/// Why the embedded dataset failed to load. Fieldless: a bare string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WikiMissingReason {
    SchemaMismatch,
    Malformed,
}

/// The state of the wiki dataset: what the verification screen shows, and the basis
/// for saying "is it worth updating?".
#[derive(Debug, Clone, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum WikiInfo {
    Loaded {
        snapshot_at: String,
        last_known_patch: Option<PatchView>,
        counts: WikiCounts,
        unresolved: u32,
        unknown_templates: u32,
        /// `None` when the game's installation date isn't known: there's no way to
        /// tell whether the snapshot is behind.
        game_newer_than_snapshot: Option<bool>,
    },
    Missing {
        reason: WikiMissingReason,
    },
}

/// Translates the outcome of `Dataset::embedded()` into a view. The dataset never
/// crosses the IPC boundary in full: only the summary needed to tell if it's current.
pub fn wiki_info(
    dataset: Result<&Dataset, &DatasetError>,
    game_updated_unix: Option<u64>,
) -> WikiInfo {
    match dataset {
        Ok(ds) => {
            let meta = &ds.meta;
            WikiInfo::Loaded {
                snapshot_at: meta.snapshot_at.clone(),
                last_known_patch: meta.last_known_patch.as_ref().map(|p| PatchView {
                    number: p.number.clone(),
                    date: p.date.clone(),
                }),
                counts: WikiCounts {
                    items: meta.counts.items,
                    trinkets: meta.counts.trinkets,
                    achievements: meta.counts.achievements,
                    bosses: meta.counts.bosses,
                    challenges: meta.counts.challenges,
                    characters: meta.counts.characters,
                },
                unresolved: meta.diagnostics.unresolved.values().sum(),
                unknown_templates: meta.diagnostics.unknown_templates.values().sum(),
                game_newer_than_snapshot: game_updated_unix
                    .zip(rfc3339_to_unix(&meta.snapshot_at))
                    .map(|(g, s)| g > s),
            }
        }
        Err(DatasetError::SchemaMismatch { .. }) => WikiInfo::Missing {
            reason: WikiMissingReason::SchemaMismatch,
        },
        Err(DatasetError::Malformed { .. }) => WikiInfo::Missing {
            reason: WikiMissingReason::Malformed,
        },
    }
}

/// Seconds since the Unix epoch for a `YYYY-MM-DDTHH:MM:SSZ` timestamp. Only this exact
/// format: anything else (no `Z`, no time, a different length) yields `None`.
pub fn rfc3339_to_unix(s: &str) -> Option<u64> {
    let b = s.as_bytes();
    if b.len() != 20
        || b[4] != b'-'
        || b[7] != b'-'
        || b[10] != b'T'
        || b[13] != b':'
        || b[16] != b':'
        || b[19] != b'Z'
    {
        return None;
    }
    let field = |range: std::ops::Range<usize>| -> Option<i64> {
        let slice = s.get(range)?;
        if slice.is_empty() || !slice.bytes().all(|c| c.is_ascii_digit()) {
            return None;
        }
        slice.parse().ok()
    };
    let year = field(0..4)?;
    let month = field(5..7)?;
    let day = field(8..10)?;
    let hour = field(11..13)?;
    let minute = field(14..16)?;
    let second = field(17..19)?;
    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || !(0..=23).contains(&hour)
        || !(0..=59).contains(&minute)
        || !(0..=59).contains(&second)
    {
        return None;
    }
    let days = days_from_civil(year, month, day);
    let secs = days * 86_400 + hour * 3_600 + minute * 60 + second;
    u64::try_from(secs).ok()
}

/// Howard Hinnant's "days from civil": days since 1970-01-01 for any civil date (even
/// before the epoch), with no calendar library and no hand-written leap-year tables.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400; // [0, 399]
    let mp = if m > 2 { m - 3 } else { m + 9 }; // [0, 11], marzo = 0
    let doy = (153 * mp + 2) / 5 + d - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146_097 + doe - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc3339() {
        assert_eq!(rfc3339_to_unix("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(rfc3339_to_unix("2026-09-05T14:20:41Z"), Some(1_788_618_041));
        assert_eq!(rfc3339_to_unix("2026-09-05"), None);
    }

    #[test]
    fn info_shapes() {
        let missing = wiki_info(Err(&DatasetError::Malformed { reason: "x".into() }), None);
        assert_eq!(
            serde_json::to_value(&missing).unwrap(),
            serde_json::json!({"kind":"missing","reason":"malformed"})
        );
        let ds = Dataset::embedded().expect("embedded dataset");
        let loaded = wiki_info(Ok(ds), Some(0));
        let v = serde_json::to_value(&loaded).unwrap();
        assert_eq!(v["kind"], "loaded");
        assert_eq!(v["gameNewerThanSnapshot"], false);
        assert_eq!(v["counts"]["trinkets"], 188);
        let v = serde_json::to_value(wiki_info(Ok(ds), None)).unwrap();
        assert_eq!(v["gameNewerThanSnapshot"], serde_json::Value::Null);
        let v = serde_json::to_value(wiki_info(Ok(ds), Some(u64::MAX / 2))).unwrap();
        assert_eq!(v["gameNewerThanSnapshot"], true);
    }

    #[test]
    fn schema_mismatch_maps_to_its_own_reason() {
        let missing = wiki_info(
            Err(&DatasetError::SchemaMismatch {
                found: 2,
                expected: 1,
            }),
            None,
        );
        assert_eq!(
            serde_json::to_value(&missing).unwrap(),
            serde_json::json!({"kind":"missing","reason":"schemaMismatch"})
        );
    }
}
