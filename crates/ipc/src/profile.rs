use discovery::{
    Diagnostic as DiscoveryDiagnostic, Discovery, Dlc, Edition, SaveCandidate, SavePrefix,
    SaveSource,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::UNIX_EPOCH;

/// Opaque matching key between a persisted choice and a candidate.
/// Newtype to prevent a path from ending up in here by mistake.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileId(String);

impl ProfileId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Deterministic id for the save, stable across launches.
/// Not a cryptographic requirement: it's a matching key.
pub fn profile_id(path: &Path) -> ProfileId {
    ProfileId(format!("{:016x}", fnv1a_64(normalized(path).as_bytes())))
}

/// On Windows the same file shows up written differently depending on how it
/// was discovered: case and separators need normalizing before hashing.
fn normalized(path: &Path) -> String {
    path.to_string_lossy().to_lowercase().replace('\\', "/")
}

fn fnv1a_64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

/// The save's origin, without the data that `discovery`'s type carries along with it
/// (Steam account id, folder): those don't cross the IPC boundary. A fieldless enum:
/// on the wire it's a bare camelCase string (`"steamCloud"`), not a tagged object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CandidateSource {
    SteamCloud,
    Documents,
    Manual,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidateView {
    pub id: ProfileId,
    pub prefix: SavePrefix,
    pub slot: u8,
    pub source: CandidateSource,
    pub modified_unix: Option<u64>,
    pub size_bytes: u64,
    pub suggested: bool,
    /// Display-only detail. No command accepts it as input.
    pub path_hint: String,
}

/// Presentable candidates, most recent to least recent.
/// Exactly one is `suggested`, if the list isn't empty.
pub fn candidates(saves: &[SaveCandidate]) -> Vec<CandidateView> {
    let mut views: Vec<CandidateView> = saves.iter().map(view_of).collect();
    // Deterministic order even without dates: on ties, prefix, then slot, then id.
    views.sort_by(|a, b| {
        b.modified_unix
            .cmp(&a.modified_unix)
            .then(a.prefix_rank().cmp(&b.prefix_rank()))
            .then(a.slot.cmp(&b.slot))
            .then(a.id.as_str().cmp(b.id.as_str()))
    });
    views
        .into_iter()
        .enumerate()
        .map(|(i, v)| CandidateView {
            suggested: i == 0,
            ..v
        })
        .collect()
}

fn view_of(save: &SaveCandidate) -> CandidateView {
    CandidateView {
        id: profile_id(&save.path),
        prefix: save.prefix,
        slot: save.slot,
        source: source_of(&save.source),
        modified_unix: save
            .modified
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs()),
        size_bytes: save.size,
        suggested: false,
        path_hint: redacted_path(&save.path, &save.source),
    }
}

/// Redacts the path by removing the data that identifies a person: the Steam account id
/// under `userdata\` and the Windows username under `Users\`.
///
/// The two come from different sources but are treated the same. The account id depends
/// on the origin (only Steam Cloud has one); the username doesn't: **any** path under
/// the user's profile contains it, manual override included. So the username mask
/// always applies, before the origin is even looked at.
fn redacted_path(path: &Path, source: &SaveSource) -> String {
    let masked = mask_user_dir(&path.display().to_string());
    match source {
        SaveSource::SteamCloud { account_id } => masked.replace(account_id.as_str(), "<account>"),
        SaveSource::Documents { .. } | SaveSource::Override => masked,
    }
}

/// Replaces the segment that follows `Users` with `<utente>`, and touches nothing
/// else: `path_hint` exists to say "found here", and a path reduced entirely to a
/// placeholder would no longer orient anyone.
///
/// It works on segments rather than the whole string because the username can be
/// anything — including a substring that also shows up elsewhere in the path.
/// Both separators are accepted: the path comes from disk on Windows, but the tests
/// write it with `/`.
fn mask_user_dir(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    let mut mask_next = false;
    for segment in path.split_inclusive(['/', '\\']) {
        let name = segment.trim_end_matches(['/', '\\']);
        let separator = &segment[name.len()..];
        if name.is_empty() {
            // Consecutive separators (`\\?\`, UNC roots): not a segment, and they must
            // not consume a pending mask.
            out.push_str(separator);
            continue;
        }
        if mask_next {
            out.push_str("<utente>");
            mask_next = false;
        } else {
            out.push_str(name);
            mask_next = name.eq_ignore_ascii_case("users");
        }
        out.push_str(separator);
    }
    out
}

fn source_of(source: &SaveSource) -> CandidateSource {
    match source {
        SaveSource::SteamCloud { .. } => CandidateSource::SteamCloud,
        SaveSource::Documents { .. } => CandidateSource::Documents,
        SaveSource::Override => CandidateSource::Manual,
    }
}

impl CandidateView {
    fn prefix_rank(&self) -> u8 {
        match self.prefix {
            SavePrefix::RepPlus => 0,
            SavePrefix::Rep => 1,
        }
    }
}

/// Where the Steam → game → saves chain broke down. A fieldless enum: on the wire it's
/// a bare camelCase string, `"steamNotFound"` and neither `"SteamNotFound"` nor
/// `{"kind":"steamNotFound"}`. The convention holds for **every** enum on the IPC —
/// `rename_all = "camelCase"` always, the tag only where variants carry data: if two
/// enums followed different conventions, a `switch` on the TypeScript side would fall
/// into no branch at all, and nothing would flag it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MissingReason {
    SteamNotFound,
    GameNotFound,
    NoSaves,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ChoiceReason {
    NeverChosen,
    SavedProfileGone { was: String },
}

#[derive(Debug, Clone, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ActiveProfile {
    None {
        reason: MissingReason,
    },
    NeedsChoice {
        reason: ChoiceReason,
        suggested: Option<ProfileId>,
    },
    Active {
        profile: CandidateView,
        auto_selected: bool,
    },
}

/// No branch returns `Active` without the user having chosen, or without the
/// candidate being the only one. A saved choice that no longer exists never falls back.
pub fn resolve_active(
    saved: Option<&ProfileId>,
    candidates: &[CandidateView],
    steam_found: bool,
    game_found: bool,
) -> ActiveProfile {
    let Some(first) = candidates.first() else {
        return ActiveProfile::None {
            reason: missing_reason(steam_found, game_found),
        };
    };

    let suggested = candidates
        .iter()
        .find(|c| c.suggested)
        .map(|c| c.id.clone());

    match saved {
        Some(id) => match candidates.iter().find(|c| &c.id == id) {
            Some(found) => ActiveProfile::Active {
                profile: found.clone(),
                auto_selected: false,
            },
            None => ActiveProfile::NeedsChoice {
                reason: ChoiceReason::SavedProfileGone {
                    was: id.as_str().to_string(),
                },
                suggested,
            },
        },
        None if candidates.len() == 1 => ActiveProfile::Active {
            profile: first.clone(),
            auto_selected: true,
        },
        None => ActiveProfile::NeedsChoice {
            reason: ChoiceReason::NeverChosen,
            suggested,
        },
    }
}

fn missing_reason(steam_found: bool, game_found: bool) -> MissingReason {
    if !steam_found {
        MissingReason::SteamNotFound
    } else if !game_found {
        MissingReason::GameNotFound
    } else {
        MissingReason::NoSaves
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamView {
    /// Display-only path: tells the user where it was found.
    pub root_hint: String,
    pub libraries: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameView {
    pub dir_hint: String,
    pub edition: Edition,
    pub dlcs: Vec<Dlc>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupState {
    pub steam: Option<SteamView>,
    pub game: Option<GameView>,
    pub candidates: Vec<CandidateView>,
    pub active: ActiveProfile,
    pub diagnostics: Vec<SetupDiagnostic>,
}

/// What failed during discovery. Carries **only the last path component**:
/// the full path contains the Steam account id under `userdata/`
/// and always the Windows username.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SetupDiagnostic {
    SteamNotFound,
    GameNotFound,
    NoSavesFound,
    UnreadablePath { name: String, reason: String },
    MalformedManifest { name: String },
}

/// Last component of a path, for diagnostics: never the full path.
fn last_component(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default()
}

fn setup_diagnostic_of(d: &DiscoveryDiagnostic) -> SetupDiagnostic {
    match d {
        DiscoveryDiagnostic::SteamNotFound => SetupDiagnostic::SteamNotFound,
        DiscoveryDiagnostic::GameNotFound => SetupDiagnostic::GameNotFound,
        DiscoveryDiagnostic::NoSavesFound => SetupDiagnostic::NoSavesFound,
        DiscoveryDiagnostic::UnreadablePath { path, reason } => SetupDiagnostic::UnreadablePath {
            name: last_component(path),
            reason: reason.clone(),
        },
        DiscoveryDiagnostic::MalformedManifest { path } => SetupDiagnostic::MalformedManifest {
            name: last_component(path),
        },
    }
}

pub fn setup_state(d: &Discovery, saved: Option<&ProfileId>) -> SetupState {
    let views = candidates(&d.saves);
    let active = resolve_active(saved, &views, d.steam.is_some(), d.game.is_some());
    SetupState {
        steam: d.steam.as_ref().map(|s| SteamView {
            root_hint: mask_user_dir(&s.root.display().to_string()),
            libraries: s.libraries.len(),
        }),
        game: d.game.as_ref().map(|g| GameView {
            dir_hint: mask_user_dir(&g.dir.display().to_string()),
            edition: g.edition,
            dlcs: g.dlcs.clone(),
        }),
        candidates: views,
        active,
        diagnostics: d.diagnostics.iter().map(setup_diagnostic_of).collect(),
    }
}
