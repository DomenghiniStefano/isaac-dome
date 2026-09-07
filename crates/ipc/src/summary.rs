use core_save::{Diagnostic as SaveParseDiagnostic, Kind, Save};
use serde::Serialize;

use crate::ProfileId;

/// Count for one section. Carries the `Kind`, not a translated label:
/// the human-readable name is a UI string and lives in the i18n files.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectionCount {
    pub kind: Kind,
    pub count: u32,
}

/// What could not be read from the save. Carries no offsets or lengths:
/// the frontend has no knowledge of the file layout.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SaveDiagnostic {
    UnexpectedKind { expected: u32, found: u32 },
    SectionOverrun { section: u32 },
    TrailingBytes,
}

fn save_diagnostic_of(d: &SaveParseDiagnostic) -> SaveDiagnostic {
    match d {
        SaveParseDiagnostic::UnexpectedKind {
            expected, found, ..
        } => SaveDiagnostic::UnexpectedKind {
            expected: *expected,
            found: *found,
        },
        SaveParseDiagnostic::SectionOverrun { kind, .. } => {
            SaveDiagnostic::SectionOverrun { section: *kind }
        }
        SaveParseDiagnostic::TrailingBytes { .. } => SaveDiagnostic::TrailingBytes,
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSummary {
    pub profile: ProfileId,
    pub sections: Vec<SectionCount>,
    pub diagnostics: Vec<SaveDiagnostic>,
}

pub fn save_summary(profile: &ProfileId, save: &Save) -> SaveSummary {
    SaveSummary {
        profile: profile.clone(),
        sections: save
            .sections
            .iter()
            .map(|s| SectionCount {
                kind: s.kind,
                count: s.count,
            })
            .collect(),
        diagnostics: save.diagnostics.iter().map(save_diagnostic_of).collect(),
    }
}
