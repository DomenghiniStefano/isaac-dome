# Tauri shell and IPC boundary — implementation plan

> **For whoever executes this:** use `superpowers:subagent-driven-development` (recommended) or
> `superpowers:executing-plans` to proceed task by task. Steps use checkboxes (`- [ ]`).

**Goal:** build the first real consumer of the three completed crates — a Tauri app that
shows real data from the user's save — and with it, the boundary between Rust and the interface.

**Architecture:** three pieces. `crates/ipc` holds the pure logic that turns `Discovery` and
`Save` into view-models, and concentrates everything worth testing. `crates/app` is the thin
Tauri crate, which registers four commands and does the only I/O that belongs to it (the
settings file). `ui/` is the Vue frontend, with a single, deliberately rough verification
screen.

**Stack:** Rust 1.95, Tauri 2, serde. Vue 3 + TypeScript, Vite, Tailwind v4, pnpm, Node 22 LTS.

**Spec:** `docs/superpowers/specs/2026-09-02-app-shell-ipc-design.md`

## Global constraints

These apply to every task, without repeating them.

- **Node 22 LTS** (≥ 22.12), pnpm enabled via `corepack enable pnpm`. Node 18 is below the
  minimum required by Vite 7 and the Tailwind v4 toolchain.
- **Read-only on saves.** No opening for writing, for any reason.
- **No hardcoded counts**: the number of entries in a section is read from the file. A
  save from January 2025 declares 521 counters, a recent one 523.
- **Degrade, never fail**: a malformed file produces diagnostics, not a `panic!`. No
  `unwrap()` outside tests on data read from disk.
- **IPC boundary**: commands `Result<T, IpcError>` never `String`; every struct crossing
  the IPC has `#[serde(rename_all = "camelCase")]`; enums tagged with `#[serde(tag = "kind")]`;
  the JSON never carries offsets, raw bytes, Steam account ids, or paths used as an
  identifier.
- **Exhaustiveness**: no `_ =>` branch on closed enums.
- **Frontend**: `docs/frontend-conventions.md`. No `<style>` in SFCs, no hardcoded visual
  values, no `invoke()` outside `src/lib/ipc/`.
- **Commits**: prefix `ipc:`, `app:`, `ui:` or `docs:`; messages in English; **never** a
  `Co-Authored-By` trailer nor references to Claude.
- **Gates, after every Rust task**: `cargo test --workspace`, `cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings`.

## File structure

```
crates/ipc/
  Cargo.toml
  src/lib.rs          re-exports the public API, nothing else
  src/profile.rs      ProfileId, CandidateView, resolve_active
  src/marks.rs        index→(boss, character) tables, MarksMatrix
  src/summary.rs      SaveSummary
  src/settings.rs     Settings (type and serialization only, no I/O)
  tests/profile.rs    id, conversion, resolution
  tests/marks.rs      matrix, known gaps, truncated section
  tests/cross_check.rs  comparison against the Python reference (skips if absent)
  tests/real_saves.rs   summary on real samples (skips if absent)

crates/app/
  Cargo.toml
  build.rs
  tauri.conf.json
  src/lib.rs          commands and Builder
  src/main.rs         entrypoint
  src/error.rs        IpcError
  src/store.rs        reads/writes settings.json

ui/
  package.json  vite.config.ts  tsconfig*.json  .prettierrc.json  eslint.config.js
  scripts/scan-conventions.mjs
  src/main.ts  src/App.vue
  src/assets/main.css          @import tailwindcss, @custom-variant dark, @theme
  src/lib/constants/commands.ts
  src/lib/ipc/setup.ts  src/lib/ipc/save.ts
  src/lib/ipc/types.ts
```

**Phase A (tasks 1–7)** is all Rust and requires no Node: it can be run right away.
**Phase B (tasks 8–9)** requires Tauri. **Phase C (tasks 10–12)** requires Node 22.

---

### Task 1: `ipc` crate, `ProfileId` and `profile_id`

**Files:**
- Create: `crates/ipc/Cargo.toml`, `crates/ipc/src/lib.rs`, `crates/ipc/src/profile.rs`
- Tests: `crates/ipc/tests/profile.rs`

**Interfaces:**
- Consumes: nothing.
- Produces: `pub struct ProfileId(String)` with `ProfileId::as_str(&self) -> &str`;
  `pub fn profile_id(path: &Path) -> ProfileId`.

- [ ] **Step 1: write the failing test**

`crates/ipc/tests/profile.rs`:

```rust
use ipc::profile_id;
use std::path::Path;

#[test]
fn profile_id_is_deterministic() {
    let a = profile_id(Path::new(r"C:\Steam\userdata\1\250900\remote\rep+persistentgamedata1.dat"));
    let b = profile_id(Path::new(r"C:\Steam\userdata\1\250900\remote\rep+persistentgamedata1.dat"));
    assert_eq!(a, b);
}

#[test]
fn profile_id_ignores_case_and_separators() {
    let a = profile_id(Path::new(r"C:\Steam\Remote\rep+persistentgamedata1.dat"));
    let b = profile_id(Path::new("c:/steam/remote/rep+persistentgamedata1.dat"));
    assert_eq!(a, b, "same file written two different ways = same id");
}

#[test]
fn different_paths_give_different_ids() {
    let a = profile_id(Path::new("c:/steam/remote/rep+persistentgamedata1.dat"));
    let b = profile_id(Path::new("c:/steam/remote/rep+persistentgamedata2.dat"));
    assert_ne!(a, b);
}

#[test]
fn profile_id_is_not_the_path() {
    let id = profile_id(Path::new("c:/steam/remote/rep+persistentgamedata1.dat"));
    assert!(!id.as_str().contains('/'), "the id must not contain the path");
    assert!(!id.as_str().contains("remote"));
}
```

- [ ] **Step 2: run it and confirm it fails**

`cargo test -p ipc --test profile`
Expected: compile error, `ipc` doesn't exist.

- [ ] **Step 3: create the crate**

`crates/ipc/Cargo.toml`:

```toml
[package]
name = "ipc"
version = "0.1.0"
edition = "2021"
description = "View-model for the UI: from discovery and core-save to already-resolved JSON"

[dependencies]
serde = { version = "1", features = ["derive"] }
discovery = { path = "../discovery" }
core-save = { path = "../core-save" }

[dev-dependencies]
serde_json = "1"
```

`crates/ipc/src/lib.rs`:

```rust
//! ipc — view-models for the UI. Pure logic: no I/O, no dependency on Tauri.

mod profile;

pub use profile::{profile_id, ProfileId};
```

`crates/ipc/src/profile.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::path::Path;

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
```

- [ ] **Step 4: run it and confirm it passes**

`cargo test -p ipc --test profile`
Expected: 4 tests passed.

- [ ] **Step 5: gates and commit**

```bash
cargo fmt --check
cargo clippy -p ipc --all-targets -- -D warnings
git add crates/ipc Cargo.lock
git commit -m "ipc: scaffold del crate e id opaco del profilo"
```

---

### Task 2: `CandidateView` and conversion from `SaveCandidate`

**Files:**
- Modify: `crates/ipc/src/profile.rs`, `crates/ipc/src/lib.rs`
- Tests: `crates/ipc/tests/profile.rs`

**Interfaces:**
- Consumes: `profile_id`, `ProfileId` from Task 1; from `discovery`: `SaveCandidate`,
  `SavePrefix`, `SaveSource`.
- Produces: `pub struct CandidateView`, `pub enum CandidateSource`,
  `pub fn candidates(saves: &[SaveCandidate]) -> Vec<CandidateView>` — sorted from most
  recent to least recent, with exactly one marked `suggested`.

- [ ] **Step 1: write the failing test**

Add to `crates/ipc/tests/profile.rs`:

```rust
use discovery::{SaveCandidate, SavePrefix, SaveSource};
use ipc::{candidates, CandidateSource};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// **Realistic** path: with Steam Cloud the account id is a folder segment.
/// A fixture missing that segment would verify not having what it never had,
/// and would give a false guarantee about the account-id test.
fn candidate(name: &str, slot: u8, prefix: SavePrefix, secs: Option<u64>) -> SaveCandidate {
    SaveCandidate {
        path: PathBuf::from(format!("c:/steam/userdata/123456789/250900/remote/{name}")),
        slot,
        source: SaveSource::SteamCloud { account_id: "123456789".into() },
        prefix,
        modified: secs.map(|s| SystemTime::UNIX_EPOCH + Duration::from_secs(s)),
        size: 14491,
    }
}

#[test]
fn candidates_are_sorted_newest_first_and_one_is_suggested() {
    let views = candidates(&[
        candidate("rep_persistentgamedata1.dat", 1, SavePrefix::Rep, Some(1_000)),
        candidate("rep+persistentgamedata1.dat", 1, SavePrefix::RepPlus, Some(2_000)),
    ]);
    assert_eq!(views.len(), 2);
    assert_eq!(views[0].prefix, SavePrefix::RepPlus, "the most recent comes first");
    assert!(views[0].suggested);
    assert!(!views[1].suggested);
}

#[test]
fn suggestion_is_stable_when_dates_are_missing() {
    let a = candidates(&[
        candidate("rep+persistentgamedata2.dat", 2, SavePrefix::RepPlus, None),
        candidate("rep+persistentgamedata1.dat", 1, SavePrefix::RepPlus, None),
    ]);
    let b = candidates(&[
        candidate("rep+persistentgamedata1.dat", 1, SavePrefix::RepPlus, None),
        candidate("rep+persistentgamedata2.dat", 2, SavePrefix::RepPlus, None),
    ]);
    assert_eq!(a[0].id, b[0].id, "without dates, the order doesn't depend on the input");
    assert_eq!(a[0].slot, 1, "on a tie, the lowest slot wins");
}

#[test]
fn candidate_view_hides_the_steam_account_id() {
    let views = candidates(&[candidate("rep+persistentgamedata1.dat", 1, SavePrefix::RepPlus, Some(1))]);
    let json = serde_json::to_string(&views[0]).unwrap();
    assert!(!json.contains("123456789"), "the Steam account id doesn't cross the IPC boundary");
    assert!(json.contains("steamCloud"));
    assert!(json.contains("modifiedUnix"), "fields in camelCase");
}

#[test]
fn prefix_keeps_the_snake_case_of_its_domain_crate() {
    let views = candidates(&[candidate("rep+persistentgamedata1.dat", 1, SavePrefix::RepPlus, Some(1))]);
    let json = serde_json::to_string(&views[0]).unwrap();
    // `SavePrefix` comes from `discovery` and is already serialized in snake_case, like
    // `Edition`, `Dlc`, and `Kind`. Only the enums defined by `ipc` use camelCase.
    // The value is pinned here because the TypeScript type declares `'rep' | 'rep_plus'`.
    assert!(json.contains("\"rep_plus\""));
}

#[test]
fn source_is_mapped_to_a_provenance_without_data() {
    let views = candidates(&[candidate("rep+persistentgamedata1.dat", 1, SavePrefix::RepPlus, Some(1))]);
    assert_eq!(views[0].source, CandidateSource::SteamCloud);
}
```

- [ ] **Step 2: run it and confirm it fails**

`cargo test -p ipc --test profile`
Expected: FAIL, `candidates` and `CandidateSource` don't exist.

- [ ] **Step 3: implement**

Add to `crates/ipc/src/profile.rs`:

```rust
use discovery::{SaveCandidate, SavePrefix, SaveSource};
use std::time::UNIX_EPOCH;

/// The save's origin, without the data that `discovery`'s type carries along with it
/// (Steam account id, folder): those don't cross the IPC boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
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
    // Deterministic order even without dates: on ties, prefix, then slot.
    views.sort_by(|a, b| {
        b.modified_unix
            .cmp(&a.modified_unix)
            .then(a.prefix_rank().cmp(&b.prefix_rank()))
            .then(a.slot.cmp(&b.slot))
    });
    views
        .into_iter()
        .enumerate()
        .map(|(i, v)| CandidateView { suggested: i == 0, ..v })
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

/// With Steam Cloud the path contains the account id as a folder segment
/// (`…\userdata\<account_id>\250900\remote\…`), and that id must not cross the IPC
/// boundary. Only that segment is replaced: truncating the path would destroy the only
/// use of `path_hint`, which is to tell the user where the file was found.
fn redacted_path(path: &Path, source: &SaveSource) -> String {
    let full = path.display().to_string();
    match source {
        SaveSource::SteamCloud { account_id } => full.replace(account_id.as_str(), "<account>"),
        SaveSource::Documents { .. } | SaveSource::Override => full,
    }
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
```

Update `crates/ipc/src/lib.rs`:

```rust
pub use profile::{candidates, profile_id, CandidateSource, CandidateView, ProfileId};
```

- [ ] **Step 4: run it and confirm it passes**

`cargo test -p ipc --test profile`
Expected: 8 tests passed.

- [ ] **Step 5: gates and commit**

```bash
cargo fmt --check
cargo clippy -p ipc --all-targets -- -D warnings
git add crates/ipc
git commit -m "ipc: presentable candidates, with neither account id nor paths as the key"
```

---

### Task 3: `resolve_active` — the rule that never falls back

**Files:**
- Modify: `crates/ipc/src/profile.rs`, `crates/ipc/src/lib.rs`
- Tests: `crates/ipc/tests/profile.rs`

**Interfaces:**
- Consumes: `CandidateView`, `ProfileId` from tasks 1–2.
- Produces: `pub enum ActiveProfile { None { reason }, NeedsChoice { reason, suggested }, Active
  { profile, auto_selected } }`, `pub enum MissingReason`, `pub enum ChoiceReason`,
  `pub fn resolve_active(saved: Option<&ProfileId>, candidates: &[CandidateView], steam_found:
  bool, game_found: bool) -> ActiveProfile`.

- [ ] **Step 1: write the failing test**

Add to `crates/ipc/tests/profile.rs`:

```rust
use ipc::{resolve_active, ActiveProfile, ChoiceReason, MissingReason};

#[test]
fn no_candidates_reports_where_the_chain_broke() {
    match resolve_active(None, &[], false, false) {
        ActiveProfile::None { reason } => assert_eq!(reason, MissingReason::SteamNotFound),
        other => panic!("expected None, got {other:?}"),
    }
    match resolve_active(None, &[], true, false) {
        ActiveProfile::None { reason } => assert_eq!(reason, MissingReason::GameNotFound),
        other => panic!("expected None, got {other:?}"),
    }
    match resolve_active(None, &[], true, true) {
        ActiveProfile::None { reason } => assert_eq!(reason, MissingReason::NoSaves),
        other => panic!("expected None, got {other:?}"),
    }
}

#[test]
fn a_single_candidate_is_selected_but_declared() {
    let views = candidates(&[candidate("rep+persistentgamedata1.dat", 1, SavePrefix::RepPlus, Some(1))]);
    match resolve_active(None, &views, true, true) {
        ActiveProfile::Active { auto_selected, profile } => {
            assert!(auto_selected, "it must be declared that the app chose, not the user");
            assert_eq!(profile.slot, 1);
        }
        other => panic!("expected Active, got {other:?}"),
    }
}

#[test]
fn several_candidates_and_no_choice_asks_the_user() {
    let views = candidates(&[
        candidate("rep+persistentgamedata1.dat", 1, SavePrefix::RepPlus, Some(2_000)),
        candidate("rep_persistentgamedata1.dat", 1, SavePrefix::Rep, Some(1_000)),
    ]);
    match resolve_active(None, &views, true, true) {
        ActiveProfile::NeedsChoice { reason, suggested } => {
            assert_eq!(reason, ChoiceReason::NeverChosen);
            assert_eq!(suggested.as_ref(), Some(&views[0].id), "the most recent one is suggested");
        }
        other => panic!("expected NeedsChoice, got {other:?}"),
    }
}

#[test]
fn a_saved_choice_that_still_exists_wins() {
    let views = candidates(&[
        candidate("rep+persistentgamedata1.dat", 1, SavePrefix::RepPlus, Some(2_000)),
        candidate("rep_persistentgamedata1.dat", 1, SavePrefix::Rep, Some(1_000)),
    ]);
    let chosen = views[1].id.clone();
    match resolve_active(Some(&chosen), &views, true, true) {
        ActiveProfile::Active { auto_selected, profile } => {
            assert!(!auto_selected, "the user chose");
            assert_eq!(profile.id, chosen, "the saved choice wins, not the suggestion");
        }
        other => panic!("expected Active, got {other:?}"),
    }
}

#[test]
fn a_saved_choice_that_vanished_never_falls_back() {
    let views = candidates(&[
        candidate("rep+persistentgamedata1.dat", 1, SavePrefix::RepPlus, Some(2_000)),
        candidate("rep_persistentgamedata1.dat", 1, SavePrefix::Rep, Some(1_000)),
    ]);
    let gone = profile_id(Path::new("c:/steam/remote/rep+persistentgamedata3.dat"));
    match resolve_active(Some(&gone), &views, true, true) {
        ActiveProfile::NeedsChoice { reason, .. } => {
            assert!(matches!(reason, ChoiceReason::SavedProfileGone { .. }));
        }
        other => panic!("never fall back to another profile: {other:?}"),
    }
}

#[test]
fn a_single_candidate_that_is_not_the_saved_one_still_asks() {
    let views = candidates(&[candidate("rep+persistentgamedata1.dat", 1, SavePrefix::RepPlus, Some(1))]);
    let gone = profile_id(Path::new("c:/steam/remote/rep+persistentgamedata3.dat"));
    match resolve_active(Some(&gone), &views, true, true) {
        ActiveProfile::NeedsChoice { reason, .. } => {
            assert!(matches!(reason, ChoiceReason::SavedProfileGone { .. }));
        }
        other => panic!("a vanished choice must be declared even if only one file remains: {other:?}"),
    }
}
```

- [ ] **Step 2: run it and confirm it fails**

`cargo test -p ipc --test profile`
Expected: FAIL, `resolve_active` doesn't exist.

- [ ] **Step 3: implement**

Add to `crates/ipc/src/profile.rs`:

```rust
/// Every tagged enum on the IPC uses `rename_all = "camelCase"`: the tag that
/// reaches the frontend is `"steamNotFound"`, not `"SteamNotFound"`. If two enums
/// followed different conventions, a `switch` on the TypeScript side would fall
/// into no branch at all, and nothing would flag it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum MissingReason {
    SteamNotFound,
    GameNotFound,
    NoSaves,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ChoiceReason {
    NeverChosen,
    SavedProfileGone { was: String },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
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
        return ActiveProfile::None { reason: missing_reason(steam_found, game_found) };
    };

    let suggested = candidates.iter().find(|c| c.suggested).map(|c| c.id.clone());

    match saved {
        Some(id) => match candidates.iter().find(|c| &c.id == id) {
            Some(found) => ActiveProfile::Active {
                profile: found.clone(),
                auto_selected: false,
            },
            None => ActiveProfile::NeedsChoice {
                reason: ChoiceReason::SavedProfileGone { was: id.as_str().to_string() },
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
```

Update the export in `lib.rs`:

```rust
pub use profile::{
    candidates, profile_id, resolve_active, ActiveProfile, CandidateSource, CandidateView,
    ChoiceReason, MissingReason, ProfileId,
};
```

- [ ] **Step 4: run it and confirm it passes**

`cargo test -p ipc --test profile`
Expected: 14 tests passed.

- [ ] **Step 5: gates and commit**

```bash
cargo fmt --check
cargo clippy -p ipc --all-targets -- -D warnings
git add crates/ipc
git commit -m "ipc: active profile resolution, with no silent fallbacks"
```

---

### Task 4: mark tables and index → cell mapping

**Files:**
- Create: `crates/ipc/src/marks.rs`
- Modify: `crates/ipc/src/lib.rs`
- Tests: `crates/ipc/tests/marks.rs`

**Interfaces:**
- Consumes: nothing.
- Produces: `pub const BOSSES: [&str; 10]`, `pub const CHARACTERS: [(&str, CharacterGroup); 34]`,
  `pub enum CharacterGroup { Original, Forgotten, Later }`,
  `pub fn counter_index(character: usize, boss: usize) -> Option<usize>` — `None` when the
  cell is not located in the tables.

The tables are carried over from `reference/isaac_counters.py`. `BLOCKS_19` is **derived and
undocumented**: it stops at Hush, and this is the reason for the 19 unknown cells.

- [ ] **Step 1: write the failing test**

`crates/ipc/tests/marks.rs`:

```rust
use ipc::{counter_index, CharacterGroup, BOSSES, CHARACTERS};

#[test]
fn tables_have_the_expected_shape() {
    assert_eq!(BOSSES.len(), 10);
    assert_eq!(CHARACTERS.len(), 34);
    assert_eq!(BOSSES[0], "Mom's Heart");
    assert_eq!(BOSSES[9], "Delirium");
    assert_eq!(CHARACTERS[0], ("Isaac", CharacterGroup::Original));
    assert_eq!(CHARACTERS[14], ("The Forgotten", CharacterGroup::Forgotten));
    assert_eq!(CHARACTERS[15], ("Bethany", CharacterGroup::Later));
    assert_eq!(CHARACTERS[33], ("T. Jacob & Esau", CharacterGroup::Later));
}

#[test]
fn original_characters_use_the_verified_blocks() {
    // Mom's Heart starts at 27; Isaac is the first of the 14.
    assert_eq!(counter_index(0, 0), Some(27));
    // Apollyon is the fourteenth: 27 + 13.
    assert_eq!(counter_index(13, 0), Some(40));
    // Delirium for the 14 originals starts at 173.
    assert_eq!(counter_index(0, 9), Some(173));
}

#[test]
fn the_forgotten_uses_single_cells() {
    assert_eq!(counter_index(14, 0), Some(203)); // Mom's Heart
    assert_eq!(counter_index(14, 8), Some(211)); // Hush
    assert_eq!(counter_index(14, 9), Some(213)); // Delirium: 212 belongs to another family
}

#[test]
fn later_characters_stop_at_hush() {
    assert_eq!(counter_index(15, 0), Some(214)); // Bethany, Mom's Heart
    assert_eq!(counter_index(33, 8), Some(384)); // T. Jacob & Esau, Hush = 366 + 18
    assert_eq!(
        counter_index(15, 9),
        None,
        "the Delirium column isn't located for the 19: this is where the unknown is born"
    );
}

#[test]
fn exactly_nineteen_cells_are_unlocated() {
    let unlocated = (0..CHARACTERS.len())
        .flat_map(|c| (0..BOSSES.len()).map(move |b| (c, b)))
        .filter(|&(c, b)| counter_index(c, b).is_none())
        .count();
    assert_eq!(unlocated, 19);
}
```

- [ ] **Step 2: run it and confirm it fails**

`cargo test -p ipc --test marks`
Expected: FAIL, the module doesn't exist.

- [ ] **Step 3: implement**

`crates/ipc/src/marks.rs`:

```rust
use serde::Serialize;

/// The ten verified bosses. Mother and The Beast don't yet have a column
/// for any character, so they don't appear here.
pub const BOSSES: [&str; 10] = [
    "Mom's Heart",
    "Isaac",
    "Satan",
    "Boss Rush",
    "Blue Baby",
    "The Lamb",
    "Mega Satan",
    "Greed",
    "Hush",
    "Delirium",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CharacterGroup {
    /// The 14 originals: 14-cell blocks, verified.
    Original,
    /// The Forgotten, added later: single cells.
    Forgotten,
    /// Bethany, Jacob & Esau, and the 17 Tainted: 19-cell blocks, derived.
    Later,
}

pub const CHARACTERS: [(&str, CharacterGroup); 34] = [
    ("Isaac", CharacterGroup::Original),
    ("Magdalene", CharacterGroup::Original),
    ("Cain", CharacterGroup::Original),
    ("Judas", CharacterGroup::Original),
    ("Blue Baby", CharacterGroup::Original),
    ("Eve", CharacterGroup::Original),
    ("Samson", CharacterGroup::Original),
    ("Azazel", CharacterGroup::Original),
    ("Lazarus", CharacterGroup::Original),
    ("Eden", CharacterGroup::Original),
    ("The Lost", CharacterGroup::Original),
    ("Lilith", CharacterGroup::Original),
    ("Keeper", CharacterGroup::Original),
    ("Apollyon", CharacterGroup::Original),
    ("The Forgotten", CharacterGroup::Forgotten),
    ("Bethany", CharacterGroup::Later),
    ("Jacob & Esau", CharacterGroup::Later),
    ("T. Isaac", CharacterGroup::Later),
    ("T. Magdalene", CharacterGroup::Later),
    ("T. Cain", CharacterGroup::Later),
    ("T. Judas", CharacterGroup::Later),
    ("T. Blue Baby", CharacterGroup::Later),
    ("T. Eve", CharacterGroup::Later),
    ("T. Samson", CharacterGroup::Later),
    ("T. Azazel", CharacterGroup::Later),
    ("T. Lazarus", CharacterGroup::Later),
    ("T. Eden", CharacterGroup::Later),
    ("T. The Lost", CharacterGroup::Later),
    ("T. Lilith", CharacterGroup::Later),
    ("T. Keeper", CharacterGroup::Later),
    ("T. Apollyon", CharacterGroup::Later),
    ("T. Forgotten", CharacterGroup::Later),
    ("T. Bethany", CharacterGroup::Later),
    ("T. Jacob & Esau", CharacterGroup::Later),
];

/// Base of the 14-cell block, per boss. Verified (REPENTOGON + real saves).
const BLOCKS_14: [usize; 10] = [27, 41, 55, 69, 83, 97, 116, 130, 144, 173];

/// Single cells for The Forgotten, per boss. 212 belongs to another family.
const FORGOTTEN: [usize; 10] = [203, 204, 205, 206, 207, 208, 209, 210, 211, 213];

/// Base of the 19-cell block, per boss. DERIVED, not documented, and it stops
/// at Hush: from Delirium onward the regularity breaks down. `None` = not located.
const BLOCKS_19: [Option<usize>; 10] = [
    Some(214),
    Some(233),
    Some(252),
    Some(271),
    Some(290),
    Some(309),
    Some(328),
    Some(347),
    Some(366),
    None,
];

const FIRST_LATER: usize = 15;

/// Index into the counters section for the (character, boss) cell.
/// `None` when the cell isn't located in the tables.
pub fn counter_index(character: usize, boss: usize) -> Option<usize> {
    let (_, group) = *CHARACTERS.get(character)?;
    let _ = BOSSES.get(boss)?;
    match group {
        CharacterGroup::Original => Some(BLOCKS_14[boss] + character),
        CharacterGroup::Forgotten => Some(FORGOTTEN[boss]),
        CharacterGroup::Later => BLOCKS_19[boss].map(|base| base + character - FIRST_LATER),
    }
}
```

Update `lib.rs`:

```rust
mod marks;
pub use marks::{counter_index, CharacterGroup, BOSSES, CHARACTERS};
```

- [ ] **Step 4: run it and confirm it passes**

`cargo test -p ipc --test marks`
Expected: 5 tests passed.

- [ ] **Step 5: gates and commit**

```bash
cargo fmt --check
cargo clippy -p ipc --all-targets -- -D warnings
git add crates/ipc
git commit -m "ipc: mark tables carried over from the Python reference"
```

---

### Task 5: `MarksMatrix`, `Known`/`Unknown`/`Unexpected` cells, and totals

**Files:**
- Modify: `crates/ipc/src/marks.rs`, `crates/ipc/src/lib.rs`
- Tests: `crates/ipc/tests/marks.rs`

**Interfaces:**
- Consumes: `counter_index`, `BOSSES`, `CHARACTERS` from Task 4.
- Produces: `pub enum Cell { Known { bits: u8 }, Unknown, Unexpected { value: u32 } }`,
  `pub struct CharacterRow`, `pub struct MarksTotals`, `pub struct MarksMatrix`,
  `pub fn marks_matrix(counters: &[u32]) -> MarksMatrix`.

- [ ] **Step 1: write the failing test**

Add to `crates/ipc/tests/marks.rs`:

```rust
use ipc::{marks_matrix, Cell};

/// Fake counters section, as long as a real save, all zero
/// except for the given indices.
fn counters(len: usize, set: &[(usize, u32)]) -> Vec<u32> {
    (0..len)
        .map(|i| set.iter().find(|&&(j, _)| j == i).map_or(0, |&(_, v)| v))
        .collect()
}

#[test]
fn matrix_has_the_expected_shape_and_totals() {
    let m = marks_matrix(&counters(523, &[]));
    assert_eq!(m.characters.len(), 34);
    assert_eq!(m.bosses.len(), 10);
    assert_eq!(m.totals.cells, 340);
    assert_eq!(m.totals.unknown, 19);
    assert_eq!(m.totals.readable, 321);
    assert_eq!(m.totals.unexpected, 0);
    assert_eq!(m.totals.started, 0);
}

#[test]
fn the_delirium_column_is_unknown_for_later_characters() {
    let m = marks_matrix(&counters(523, &[]));
    // row 15 = Bethany, column 9 = Delirium
    assert_eq!(m.characters[15].cells[9], Cell::Unknown);
    // row 0 = Isaac, same column: located
    assert_eq!(m.characters[0].cells[9], Cell::Known { bits: 0 });
}

#[test]
fn a_read_value_becomes_a_bit_mask() {
    let m = marks_matrix(&counters(523, &[(27, 3), (41, 7)]));
    assert_eq!(m.characters[0].cells[0], Cell::Known { bits: 3 });
    assert_eq!(m.characters[0].cells[1], Cell::Known { bits: 7 });
    assert_eq!(m.totals.started, 2, "only nonzero readable cells count");
}

#[test]
fn a_value_outside_the_mask_range_is_flagged_not_truncated() {
    let m = marks_matrix(&counters(523, &[(27, 49)]));
    assert_eq!(
        m.characters[0].cells[0],
        Cell::Unexpected { value: 49 },
        "49 isn't a mask: the table would be wrong, it must not be truncated to 1"
    );
    assert_eq!(m.totals.unexpected, 1);
    assert_eq!(m.totals.started, 0, "a suspicious cell doesn't count as a started mark");
}

#[test]
fn a_shorter_section_yields_unknown_not_a_panic() {
    // 300 counters: the 19-cell blocks (214..384) mostly fall outside.
    let m = marks_matrix(&counters(300, &[]));
    assert_eq!(m.characters[33].cells[8], Cell::Unknown, "index 384 is outside the file");
    assert_eq!(m.characters[0].cells[0], Cell::Known { bits: 0 }, "index 27 is still there");
    assert!(m.totals.unknown > 19);
    assert_eq!(m.totals.readable + m.totals.unknown, 340);
}

#[test]
fn an_empty_section_is_all_unknown() {
    let m = marks_matrix(&[]);
    assert_eq!(m.totals.unknown, 340);
    assert_eq!(m.totals.readable, 0);
}
```

- [ ] **Step 2: run it and confirm it fails**

`cargo test -p ipc --test marks`
Expected: FAIL, `marks_matrix` doesn't exist.

- [ ] **Step 3: implement**

Add to `crates/ipc/src/marks.rs`:

```rust
/// A cell of the matrix. The three variants are the module's reason for existing:
/// "never done", "not readable", and "suspicious value" are three different things.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum Cell {
    /// Valid mask: bits 0 and 1 = mark levels, bit 2 = unexplained third level.
    Known { bits: u8 },
    /// Index not located in the tables, or past the end of the section read.
    Unknown,
    /// Outside 0..=7: not a mask, so the index points somewhere else.
    Unexpected { value: u32 },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterRow {
    pub character: String,
    pub group: CharacterGroup,
    pub cells: Vec<Cell>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarksTotals {
    pub cells: usize,
    pub readable: usize,
    pub unknown: usize,
    pub unexpected: usize,
    pub started: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarksMatrix {
    pub characters: Vec<CharacterRow>,
    pub bosses: Vec<String>,
    pub totals: MarksTotals,
}

/// Builds the matrix from the counters read out of the file. It assumes no fixed
/// length: an index past the section read produces `Unknown`.
pub fn marks_matrix(counters: &[u32]) -> MarksMatrix {
    let rows: Vec<CharacterRow> = CHARACTERS
        .iter()
        .enumerate()
        .map(|(c, &(name, group))| CharacterRow {
            character: name.to_string(),
            group,
            cells: (0..BOSSES.len()).map(|b| cell_at(counters, c, b)).collect(),
        })
        .collect();

    let totals = totals_of(&rows);
    MarksMatrix {
        characters: rows,
        bosses: BOSSES.iter().map(|b| b.to_string()).collect(),
        totals,
    }
}

fn cell_at(counters: &[u32], character: usize, boss: usize) -> Cell {
    match counter_index(character, boss).and_then(|i| counters.get(i)) {
        None => Cell::Unknown,
        Some(&value) if value <= 7 => Cell::Known { bits: value as u8 },
        Some(&value) => Cell::Unexpected { value },
    }
}

fn totals_of(rows: &[CharacterRow]) -> MarksTotals {
    let cells = || rows.iter().flat_map(|r| r.cells.iter());
    // `cells()` returns a fresh iterator on every call: counting one category
    // consumes the iterator, so a single one can't be reused.
    let count = |f: fn(&Cell) -> bool| cells().filter(|&c| f(c)).count();
    MarksTotals {
        cells: cells().count(),
        readable: count(|c| matches!(c, Cell::Known { .. })),
        unknown: count(|c| matches!(c, Cell::Unknown)),
        unexpected: count(|c| matches!(c, Cell::Unexpected { .. })),
        started: count(|c| matches!(c, Cell::Known { bits } if *bits != 0)),
    }
}
```

Update `lib.rs`:

```rust
pub use marks::{
    counter_index, marks_matrix, Cell, CharacterGroup, CharacterRow, MarksMatrix, MarksTotals,
    BOSSES, CHARACTERS,
};
```

- [ ] **Step 4: run it and confirm it passes**

`cargo test -p ipc --test marks`
Expected: 11 tests passed.

- [ ] **Step 5: gates and commit**

```bash
cargo fmt --check
cargo clippy -p ipc --all-targets -- -D warnings
git add crates/ipc
git commit -m "ipc: matrice dei marchi con celle note, ignote e sospette"
```

---

### Task 6: cross-check against the Python reference

**Files:**
- Create: `crates/ipc/tests/cross_check.rs`

**Interfaces:**
- Consumes: `marks_matrix`, `Cell`, `BOSSES`, `CHARACTERS` from tasks 4–5; `core_save::Save`.
- Produces: no API. This is the test that guards against mis-transcribed tables.

The reference is the only independent source: a transcription error would produce a
plausible but wrong matrix, which no test written from the same code would catch.
The comparison happens **only on the cells the reference actually knows** — its
`marks()` function omits combinations it can't locate, so the JSON itself says which those are.

The reference is invoked as `python`, not `python3` (absent on Windows).

- [ ] **Step 1: write the failing test**

`crates/ipc/tests/cross_check.rs`:

```rust
use core_save::{Kind, Save};
use ipc::{marks_matrix, Cell, BOSSES, CHARACTERS};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

const SAMPLE: &str = "20250112.rep+persistentgamedata1.dat";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Matrix from the Python reference: { boss: { character: value } }.
/// Combinations the reference doesn't locate simply aren't there.
fn reference_marks(sample: &Path) -> Option<HashMap<String, HashMap<String, u32>>> {
    let script = "import sys, json; sys.path.insert(0, 'reference'); \
                  from isaac_save import Save; from isaac_counters import marks; \
                  print(json.dumps(marks(Save(sys.argv[1]).u32s('counters'))))";
    let out = Command::new("python")
        .current_dir(repo_root())
        .args(["-c", script, &sample.display().to_string()])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    serde_json::from_slice(&out.stdout).ok()
}

#[test]
fn rust_matrix_agrees_with_the_python_reference() {
    let sample = repo_root().join("samples").join(SAMPLE);
    let Ok(bytes) = std::fs::read(&sample) else {
        eprintln!("skip: {SAMPLE} missing from samples/");
        return;
    };
    let Some(reference) = reference_marks(&sample) else {
        eprintln!("skip: python or the reference are not available");
        return;
    };

    let counters = Save::parse(&bytes)
        .expect("the sample must be readable")
        .u32s(Kind::Counters)
        .expect("the counters section must be there");
    let matrix = marks_matrix(&counters);

    let compared = CHARACTERS
        .iter()
        .enumerate()
        .flat_map(|(c, &(name, _))| BOSSES.iter().enumerate().map(move |(b, boss)| (c, name, b, *boss)))
        .filter(|&(c, name, b, boss)| {
            let expected = reference.get(boss).and_then(|row| row.get(name));
            let actual = matrix.characters[c].cells[b];
            match (expected, actual) {
                // The reference knows the cell: it must match.
                (Some(&v), Cell::Known { bits }) => {
                    assert_eq!(u32::from(bits), v, "{name} × {boss}");
                    true
                }
                (Some(&v), other) => panic!("{name} × {boss}: the reference says {v}, we say {other:?}"),
                // The reference doesn't know it: ours must be unknown.
                (None, Cell::Unknown) => false,
                (None, other) => panic!("{name} × {boss}: the reference doesn't locate it, we say {other:?}"),
            }
        })
        .count();

    assert_eq!(compared, matrix.totals.readable, "all readable cells compared");
    assert_eq!(matrix.totals.unexpected, 0, "no suspicious value on a real save");
}
```

- [ ] **Step 2: run it and check the outcome**

`cargo test -p ipc --test cross_check -- --nocapture`
Expected with the sample present: PASS. If the test fails, **the working hypothesis is that
a table was mis-transcribed**: fix `marks.rs`, not the test.
Expected without the sample or without Python: the test prints `skip:` and passes.

- [ ] **Step 3: verify the expected number matches the measured one**

On the January 2025 sample the reference reports **93 marks started out of 321 readable
cells**. Add the assertion:

```rust
    assert_eq!(matrix.totals.readable, 321);
    assert_eq!(matrix.totals.started, 93, "measured on the January 2025 sample");
```

Note: these two numbers depend on the sample, not on the format. If a different save ends
up in `samples/`, `started` changes — and in that case the expected value must be updated
**after** verifying against the reference, never by adapting it to the Rust code's output.

- [ ] **Step 4: run it again**

`cargo test -p ipc --test cross_check`
Expected: PASS.

- [ ] **Step 5: gates and commit**

```bash
cargo fmt --check
cargo clippy -p ipc --all-targets -- -D warnings
git add crates/ipc
git commit -m "ipc: cross-check of the matrix against the Python reference"
```

---

### Task 7: `SaveSummary`, `SetupState`, and `Settings`

**Files:**
- Create: `crates/ipc/src/summary.rs`, `crates/ipc/src/settings.rs`
- Modify: `crates/ipc/src/lib.rs`, `crates/ipc/src/profile.rs`
- Tests: `crates/ipc/tests/real_saves.rs`, additions to `crates/ipc/tests/profile.rs`

**Interfaces:**
- Consumes: everything above; `core_save::{Save, Kind}`; `discovery::Discovery`.
- Produces: `pub struct SectionCount { kind, count }`, `pub struct SaveSummary`,
  `pub fn save_summary(profile: &ProfileId, save: &Save) -> SaveSummary`;
  `pub struct SetupState`, `pub struct SteamView`, `pub struct GameView`,
  `pub fn setup_state(d: &Discovery, saved: Option<&ProfileId>) -> SetupState`;
  `pub struct Settings { active_profile_id: Option<ProfileId> }` with `Default`.

- [ ] **Step 1: write the failing test**

`crates/ipc/tests/real_saves.rs`:

```rust
use core_save::{Kind, Save};
use ipc::{profile_id, save_summary, Settings};
use std::path::{Path, PathBuf};

fn sample(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../samples").join(name)
}

#[test]
fn summary_reports_the_counts_declared_by_the_file() {
    let path = sample("20250112.rep+persistentgamedata1.dat");
    let Ok(bytes) = std::fs::read(&path) else {
        eprintln!("skip: sample missing");
        return;
    };
    let save = Save::parse(&bytes).unwrap();
    let summary = save_summary(&profile_id(&path), &save);

    assert_eq!(summary.sections.len(), 10, "ten sections");
    let counters = summary.sections.iter().find(|s| s.kind == Kind::Counters).unwrap();
    assert_eq!(
        counters.count,
        save.section(Kind::Counters).unwrap().count,
        "the count is read from the file, not hardcoded"
    );
    assert!(summary.diagnostics.is_empty(), "a real save produces no diagnostics");
}

#[test]
fn summary_carries_no_raw_bytes() {
    let path = sample("20250112.rep+persistentgamedata1.dat");
    let Ok(bytes) = std::fs::read(&path) else {
        eprintln!("skip: sample missing");
        return;
    };
    let save = Save::parse(&bytes).unwrap();
    let json = serde_json::to_string(&save_summary(&profile_id(&path), &save)).unwrap();
    assert!(!json.contains("bytes"), "raw bytes don't cross the IPC boundary");
    assert!(!json.contains("offset"), "offsets don't cross the IPC boundary");
    assert!(json.len() < 2_000, "the payload is small: {} bytes", json.len());
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
        serde_json::from_str::<Settings>("{}").unwrap().active_profile_id,
        None,
        "an empty file is equivalent to no choice"
    );
}
```

- [ ] **Step 2: run it and confirm it fails**

`cargo test -p ipc --test real_saves`
Expected: FAIL, `save_summary` and `Settings` don't exist.

- [ ] **Step 3: implement**

`crates/ipc/src/settings.rs`:

```rust
use serde::{Deserialize, Serialize};

use crate::ProfileId;

/// Persisted settings. The type and its serialization live here;
/// reading and writing the file live in the app crate.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub active_profile_id: Option<ProfileId>,
}
```

`crates/ipc/src/summary.rs`:

```rust
use core_save::{Kind, Save};
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSummary {
    pub profile: ProfileId,
    pub sections: Vec<SectionCount>,
    pub diagnostics: Vec<String>,
}

pub fn save_summary(profile: &ProfileId, save: &Save) -> SaveSummary {
    SaveSummary {
        profile: profile.clone(),
        sections: save
            .sections
            .iter()
            .map(|s| SectionCount { kind: s.kind, count: s.count })
            .collect(),
        diagnostics: save.diagnostics.iter().map(save_diagnostic).collect(),
    }
}
```

In `crates/ipc/src/profile.rs`, add `SetupState` and the two views:

```rust
use discovery::{Discovery, Dlc, Edition};

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
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum SetupDiagnostic {
    SteamNotFound,
    GameNotFound,
    NoSavesFound,
    UnreadablePath { name: String, reason: String },
    MalformedManifest { name: String },
}

pub fn setup_state(d: &Discovery, saved: Option<&ProfileId>) -> SetupState {
    let views = candidates(&d.saves);
    let active = resolve_active(saved, &views, d.steam.is_some(), d.game.is_some());
    SetupState {
        steam: d.steam.as_ref().map(|s| SteamView {
            root_hint: s.root.display().to_string(),
            libraries: s.libraries.len(),
        }),
        game: d.game.as_ref().map(|g| GameView {
            dir_hint: g.dir.display().to_string(),
            edition: g.edition,
            dlcs: g.dlcs.clone(),
        }),
        candidates: views,
        active,
        diagnostics: d.diagnostics.iter().map(setup_diagnostic).collect(),
    }
}
```

`crates/ipc/src/lib.rs` in its final form — the file just re-exports:

```rust
//! ipc — view-models for the UI. Pure logic: no I/O, no dependency on Tauri.

mod marks;
mod profile;
mod settings;
mod summary;

pub use marks::{
    counter_index, marks_matrix, Cell, CharacterGroup, CharacterRow, MarksMatrix, MarksTotals,
    BOSSES, CHARACTERS,
};
pub use profile::{
    candidates, profile_id, resolve_active, setup_state, ActiveProfile, CandidateSource,
    CandidateView, ChoiceReason, GameView, MissingReason, ProfileId, SetupState, SteamView,
};
pub use settings::Settings;
pub use summary::{save_summary, SaveSummary, SectionCount};
```

- [ ] **Step 4: run it and confirm it passes**

`cargo test -p ipc`
Expected: all green (tests on samples skip if `samples/` is empty).

- [ ] **Step 5: gates and commit**

```bash
cargo test --workspace
cargo fmt --check
cargo clippy --all-targets -- -D warnings
git add crates/ipc
git commit -m "ipc: save summary, installation state, settings"
```

---

### Task 8: `app` crate, Tauri shell, and `IpcError`

**Files:**
- Create: `crates/app/Cargo.toml`, `crates/app/build.rs`, `crates/app/tauri.conf.json`,
  `crates/app/src/main.rs`, `crates/app/src/lib.rs`, `crates/app/src/error.rs`

**Interfaces:**
- Consumes: `ipc` in its entirety.
- Produces: `pub enum IpcError { NoActiveProfile, UnknownProfile { id }, UnreadableSave
  { reason }, SettingsNotWritable { reason } }`, serializable and tagged.

**Windows prerequisite:** MSVC Build Tools (already present: the workspace compiles and
links) and WebView2 (bundled with Windows 11).

The Tauri crate is **a single crate**, with `lib.rs` and `[[bin]] main.rs`: splitting the
entrypoint into a separate crate is a documented cause of `a bin target must be available for
cargo run`.

- [ ] **Step 1: create the crate**

`crates/app/Cargo.toml` (check the current version with `cargo search tauri` before pinning
it):

```toml
[package]
name = "app"
version = "0.1.0"
edition = "2021"
description = "IsaacDome — desktop application"

[lib]
name = "app_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[[bin]]
name = "isaac-dome"
path = "src/main.rs"

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
ipc = { path = "../ipc" }
discovery = { path = "../discovery" }
core-save = { path = "../core-save" }
```

`crates/app/build.rs`:

```rust
fn main() {
    tauri_build::build()
}
```

`crates/app/tauri.conf.json`:

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "IsaacDome",
  "version": "0.1.0",
  "identifier": "dev.isaacdome.app",
  "build": {
    "devUrl": "http://localhost:1420",
    "frontendDist": "../../ui/dist",
    "beforeDevCommand": "pnpm --dir ../../ui dev",
    "beforeBuildCommand": "pnpm --dir ../../ui build"
  },
  "app": {
    "windows": [{ "title": "IsaacDome", "width": 1280, "height": 800 }],
    "security": { "csp": null }
  },
  "bundle": { "active": true, "targets": "all" }
}
```

- [ ] **Step 2: write `IpcError`**

`crates/app/src/error.rs`:

```rust
use serde::Serialize;

/// Error that crosses the IPC boundary. Tagged, not a string: the UI must be able to
/// tell "no active profile" apart from "unreadable file" without parsing text.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum IpcError {
    /// Not an error to display: the UI goes to the selection screen.
    NoActiveProfile,
    UnknownProfile { id: String },
    UnreadableSave { reason: String },
    SettingsNotWritable { reason: String },
}
```

- [ ] **Step 3: confirm it compiles**

Add `crates/app/src/lib.rs` with just `mod error;` and an empty `run()`,
`crates/app/src/main.rs` calling it, then:

`cargo check -p app`
Expected: it compiles. If `tauri-build` complains about the missing `../../ui/dist`,
create a temporary `ui/dist/.gitkeep`.

- [ ] **Step 4: gates and commit**

```bash
cargo fmt --check
cargo clippy -p app --all-targets -- -D warnings
git add crates/app Cargo.lock
git commit -m "app: Tauri crate scaffold and the IPC error type"
```

---

### Task 9: settings persistence and the four commands

**Files:**
- Create: `crates/app/src/store.rs`
- Modify: `crates/app/src/lib.rs`

**Interfaces:**
- Consumes: `ipc::{setup_state, save_summary, marks_matrix, resolve_active, Settings,
  ProfileId, ActiveProfile}`, `discovery::discover`, `core_save::{Save, Kind}`.
- Produces: the commands `setup_state`, `select_profile`, `save_summary`, `completion`.

The commands take no paths: the active profile is state held by the backend.

- [ ] **Step 1: settings I/O**

`crates/app/src/store.rs`:

```rust
use ipc::Settings;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use crate::error::IpcError;

fn settings_path(app: &AppHandle) -> Result<PathBuf, IpcError> {
    app.path()
        .app_config_dir()
        .map(|d| d.join("settings.json"))
        .map_err(|e| IpcError::SettingsNotWritable { reason: e.to_string() })
}

/// A missing, unreadable, or malformed file is treated as "no choice saved".
/// Never a fatal error, never a silent overwrite of the user's file.
pub fn load(app: &AppHandle) -> Settings {
    settings_path(app)
        .ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(app: &AppHandle, settings: &Settings) -> Result<(), IpcError> {
    let path = settings_path(app)?;
    let fail = |e: std::io::Error| IpcError::SettingsNotWritable { reason: e.to_string() };
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(fail)?;
    }
    let body = serde_json::to_string_pretty(settings)
        .map_err(|e| IpcError::SettingsNotWritable { reason: e.to_string() })?;
    std::fs::write(&path, body).map_err(fail)
}
```

- [ ] **Step 2: the commands**

`crates/app/src/lib.rs`:

```rust
mod error;
mod store;

use core_save::{Kind, Save};
use discovery::{discover, Options};
use ipc::{ActiveProfile, MarksMatrix, ProfileId, SaveSummary, SetupState, Settings};
use tauri::AppHandle;

use crate::error::IpcError;

#[tauri::command]
fn setup_state(app: AppHandle) -> Result<SetupState, IpcError> {
    let settings = store::load(&app);
    let d = discover(&Options::default());
    Ok(ipc::setup_state(&d, settings.active_profile_id.as_ref()))
}

#[tauri::command]
fn select_profile(app: AppHandle, id: ProfileId) -> Result<SetupState, IpcError> {
    let d = discover(&Options::default());
    let views = ipc::candidates(&d.saves);
    if !views.iter().any(|c| c.id == id) {
        return Err(IpcError::UnknownProfile { id: id.as_str().to_string() });
    }
    let settings = Settings { active_profile_id: Some(id) };
    store::save(&app, &settings)?;
    Ok(ipc::setup_state(&d, settings.active_profile_id.as_ref()))
}

#[tauri::command]
fn save_summary(app: AppHandle) -> Result<SaveSummary, IpcError> {
    let (id, save) = active_save(&app)?;
    Ok(ipc::save_summary(&id, &save))
}

#[tauri::command]
fn completion(app: AppHandle) -> Result<MarksMatrix, IpcError> {
    let (_, save) = active_save(&app)?;
    let counters = save.u32s(Kind::Counters).unwrap_or_default();
    Ok(ipc::marks_matrix(&counters))
}

/// Opens the active profile's save. `NoActiveProfile` when there isn't one:
/// for the UI that means "go to selection", not an error to display.
fn active_save(app: &AppHandle) -> Result<(ProfileId, Save), IpcError> {
    let settings = store::load(app);
    let d = discover(&Options::default());
    let views = ipc::candidates(&d.saves);
    let state = ipc::resolve_active(
        settings.active_profile_id.as_ref(),
        &views,
        d.steam.is_some(),
        d.game.is_some(),
    );
    let ActiveProfile::Active { profile, .. } = state else {
        return Err(IpcError::NoActiveProfile);
    };
    let candidate = d
        .saves
        .iter()
        .find(|s| ipc::profile_id(&s.path) == profile.id)
        .ok_or(IpcError::NoActiveProfile)?;
    let save = Save::open(&candidate.path)
        .map_err(|e| IpcError::UnreadableSave { reason: format!("{e:?}") })?;
    Ok((profile.id, save))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            setup_state,
            select_profile,
            save_summary,
            completion
        ])
        .run(tauri::generate_context!())
        .expect("failed to start the application");
}
```

`crates/app/src/main.rs`:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    app_lib::run()
}
```

Note: `ProfileId` must derive `Deserialize` (already done in Task 1) because it arrives as
a command argument.

- [ ] **Step 3: confirm it compiles**

`cargo check -p app`
Expected: compiles without warnings.

- [ ] **Step 4: gates and commit**

```bash
cargo fmt --check
cargo clippy -p app --all-targets -- -D warnings
git add crates/app
git commit -m "app: the four commands and active profile persistence"
```

---

### Task 10: `ui/`, tooling, and tokens

**Prerequisite:** Node ≥ 22.12 installed, `corepack enable pnpm` run.

**Files:**
- Create: `ui/` (Vite `vue-ts` template), `ui/.prettierrc.json`, `ui/eslint.config.js`,
  `ui/src/assets/main.css`
- Modify: `ui/vite.config.ts`, `ui/package.json`, `.gitignore`

- [ ] **Step 1: create the project**

```bash
pnpm create vite ui --template vue-ts
cd ui && pnpm install
pnpm add @tauri-apps/api tailwindcss @tailwindcss/vite
pnpm add -D @tauri-apps/cli prettier prettier-plugin-tailwindcss eslint eslint-plugin-vue typescript-eslint @vue/eslint-config-typescript vue-tsc
```

- [ ] **Step 2: configure Vite, Prettier, and ESLint**

`ui/vite.config.ts`:

```ts
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [vue(), tailwindcss()],
  server: { port: 1420, strictPort: true },
})
```

`ui/.prettierrc.json` — `tailwindStylesheet` is mandatory in Tailwind v4, where there is no
longer a JavaScript config file to read the tokens from:

```json
{
  "semi": false,
  "singleQuote": true,
  "plugins": ["prettier-plugin-tailwindcss"],
  "tailwindStylesheet": "./src/assets/main.css"
}
```

`ui/eslint.config.js`:

```js
import pluginVue from 'eslint-plugin-vue'
import { withVueTs, vueTsConfigs } from '@vue/eslint-config-typescript'

export default withVueTs(
  { ignores: ['dist', 'node_modules'] },
  pluginVue.configs['flat/essential'],
  vueTsConfigs.recommended,
)
```

Scripts in `ui/package.json`:

```json
{
  "scripts": {
    "dev": "vite",
    "build": "vue-tsc --noEmit && vite build",
    "typecheck": "vue-tsc --noEmit",
    "lint": "eslint .",
    "format": "prettier --write .",
    "format:check": "prettier --check .",
    "scan": "node scripts/scan-conventions.mjs"
  }
}
```

- [ ] **Step 3: CSS and tokens**

`ui/src/assets/main.css` — the token **values** will be set by the design system; here we
only establish where they live:

```css
@import 'tailwindcss';

@custom-variant dark (&:where(.dark, .dark *));

@theme {
  --color-mark-done: oklch(0.72 0.11 150);
  --color-mark-partial: oklch(0.78 0.13 85);
  --color-mark-none: oklch(0.45 0.02 260);
  --color-mark-unknown: oklch(0.55 0.05 320);
  --opacity-muted: 0.6;
  --opacity-disabled: 0.4;
}
```

Import it in `ui/src/main.ts` and remove the template's `style.css`.

- [ ] **Step 4: verify**

```bash
cd ui && pnpm typecheck && pnpm lint && pnpm format:check && pnpm build
```
Expected: all green.

- [ ] **Step 5: commit**

Add `ui/node_modules/` and `ui/dist/` to `.gitignore` if not already covered.

```bash
git add ui .gitignore
git commit -m "ui: scaffold Vue 3 + Tailwind v4, con lint, format e type-check"
```

---

### Task 11: IPC layer and verification screen

**Files:**
- Create: `ui/src/lib/constants/commands.ts`, `ui/src/lib/ipc/types.ts`,
  `ui/src/lib/ipc/setup.ts`, `ui/src/lib/ipc/save.ts`
- Modify: `ui/src/App.vue`

**Interfaces:**
- Consumes: the four commands from Task 9.
- Produces: typed `setupState()`, `selectProfile(id)`, `saveSummary()`, `completion()`.

- [ ] **Step 1: constants and types**

`ui/src/lib/constants/commands.ts` — command names are strings and live in one place:

```ts
export const Command = {
  SetupState: 'setup_state',
  SelectProfile: 'select_profile',
  SaveSummary: 'save_summary',
  Completion: 'completion',
} as const
```

`ui/src/lib/ipc/types.ts` — mirrors the Rust types:

```ts
export type CandidateSource = { kind: 'steamCloud' } | { kind: 'documents' } | { kind: 'manual' }

export interface CandidateView {
  id: string
  prefix: 'rep' | 'rep_plus'
  slot: number
  source: CandidateSource
  modifiedUnix: number | null
  sizeBytes: number
  suggested: boolean
  pathHint: string
}

export type ActiveProfile =
  | { kind: 'none'; reason: { kind: 'steamNotFound' | 'gameNotFound' | 'noSaves' } }
  | {
      kind: 'needsChoice'
      reason: { kind: 'neverChosen' } | { kind: 'savedProfileGone'; was: string }
      suggested: string | null
    }
  | { kind: 'active'; profile: CandidateView; autoSelected: boolean }

export interface SetupState {
  steam: { rootHint: string; libraries: number } | null
  game: { dirHint: string; edition: string; dlcs: string[] } | null
  candidates: CandidateView[]
  active: ActiveProfile
  diagnostics: SetupDiagnostic[]
}

export type SetupDiagnostic =
  | { kind: 'steamNotFound' }
  | { kind: 'gameNotFound' }
  | { kind: 'noSavesFound' }
  | { kind: 'unreadablePath'; name: string; reason: string }
  | { kind: 'malformedManifest'; name: string }

export type Cell =
  | { kind: 'known'; bits: number }
  | { kind: 'unknown' }
  | { kind: 'unexpected'; value: number }

export interface MarksMatrix {
  characters: { character: string; group: string; cells: Cell[] }[]
  bosses: string[]
  totals: { cells: number; readable: number; unknown: number; unexpected: number; started: number }
}

export interface SaveSummary {
  profile: string
  sections: { kind: string; count: number }[]
  diagnostics: SaveDiagnostic[]
}

export type SaveDiagnostic =
  | { kind: 'unexpectedKind'; expected: number; found: number }
  | { kind: 'sectionOverrun'; section: number }
  | { kind: 'trailingBytes' }
```

- [ ] **Step 2: the wrappers — the only place with `invoke`**

`ui/src/lib/ipc/setup.ts`:

```ts
import { invoke } from '@tauri-apps/api/core'
import { Command } from '../constants/commands'
import type { SetupState } from './types'

export const setupState = (): Promise<SetupState> => invoke(Command.SetupState)

export const selectProfile = (id: string): Promise<SetupState> =>
  invoke(Command.SelectProfile, { id })
```

`ui/src/lib/ipc/save.ts`:

```ts
import { invoke } from '@tauri-apps/api/core'
import { Command } from '../constants/commands'
import type { MarksMatrix, SaveSummary } from './types'

export const saveSummary = (): Promise<SaveSummary> => invoke(Command.SaveSummary)

export const completion = (): Promise<MarksMatrix> => invoke(Command.Completion)
```

- [ ] **Step 3: the verification screen**

`ui/src/App.vue` — deliberately rough, but compliant with the conventions: no `<style>`, no
hardcoded values, no direct `invoke`. The `unknown` cells must be **visibly different**
from zeros: that's the point this screen has to demonstrate.

```vue
<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { selectProfile, setupState } from './lib/ipc/setup'
import { completion, saveSummary } from './lib/ipc/save'
import type { Cell, MarksMatrix, SaveSummary, SetupState } from './lib/ipc/types'
import { assertNever } from './lib/assertNever'

const state = ref<SetupState | null>(null)
const summary = ref<SaveSummary | null>(null)
const matrix = ref<MarksMatrix | null>(null)
const error = ref<string | null>(null)

const load = async () => {
  state.value = await setupState()
  if (state.value.active.kind !== 'active') return
  summary.value = await saveSummary()
  matrix.value = await completion()
}

const choose = async (id: string) => {
  state.value = await selectProfile(id)
  await load()
}

// Exhaustiveness is mandatory: `docs/frontend-conventions.md` names `Cell` as one of
// the two types where a forgotten branch produces a false value instead of an
// error. A nested ternary would still compile even with a new variant.
const cellText = (c: Cell) => {
  switch (c.kind) {
    case 'known':
      return String(c.bits)
    case 'unknown':
      return '?'
    case 'unexpected':
      return `!${c.value}`
    default:
      return assertNever(c)
  }
}

const cellClass = (c: Cell) => {
  switch (c.kind) {
    case 'known':
      return 'text-mark-done'
    case 'unknown':
      return 'text-mark-unknown italic'
    case 'unexpected':
      return 'text-mark-partial font-bold'
    default:
      return assertNever(c)
  }
}

onMounted(() => load().catch((e) => (error.value = JSON.stringify(e))))
</script>

<template>
  <main class="flex flex-col gap-6 p-6 font-mono text-sm">
    <p v-if="error">{{ error }}</p>

    <section v-if="state" class="flex flex-col gap-2">
      <h1 class="text-lg font-bold">Stato</h1>
      <p>Steam: {{ state.steam?.rootHint ?? 'not found' }}</p>
      <p>Game: {{ state.game?.dirHint ?? 'not found' }} ({{ state.game?.edition ?? '—' }})</p>
      <p>Profilo attivo: {{ state.active.kind }}</p>
      <ul class="flex flex-col gap-1">
        <li v-for="c in state.candidates" :key="c.id">
          <button class="underline" @click="choose(c.id)">
            {{ c.prefix }} slot {{ c.slot }} — {{ c.sizeBytes }} byte
            <span v-if="c.suggested">(suggerito)</span>
          </button>
        </li>
      </ul>
    </section>

    <section v-if="summary" class="flex flex-col gap-1">
      <h2 class="font-bold">Sezioni</h2>
      <p v-for="s in summary.sections" :key="s.kind">{{ s.kind }}: {{ s.count }}</p>
    </section>

    <section v-if="matrix" class="flex flex-col gap-2 overflow-x-auto">
      <h2 class="font-bold">
        Marchi: {{ matrix.totals.started }} iniziati su {{ matrix.totals.readable }} leggibili
        ({{ matrix.totals.unknown }} ignoti, {{ matrix.totals.unexpected }} sospetti)
      </h2>
      <table>
        <thead>
          <tr>
            <th class="text-left"></th>
            <th v-for="b in matrix.bosses" :key="b" class="px-2 text-left">{{ b }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="row in matrix.characters" :key="row.character">
            <td class="pr-2">{{ row.character }}</td>
            <td v-for="(c, i) in row.cells" :key="i" :class="cellClass(c)" class="px-2">
              {{ cellText(c) }}
            </td>
          </tr>
        </tbody>
      </table>
    </section>
  </main>
</template>
```

- [ ] **Step 4: run the app**

```bash
cargo tauri dev --config crates/app/tauri.conf.json
```
(or `pnpm --dir ui tauri dev` if the CLI is configured there)

Expected on this machine: Steam found, game **not** found, six candidates, state
`NeedsChoice`. After clicking a candidate: section counts and a 34×10 matrix, with the
Delirium column showing `?` for the last 19 rows.

- [ ] **Step 5: verify and commit**

```bash
cd ui && pnpm typecheck && pnpm lint && pnpm format:check
git add ui
git commit -m "ui: layer IPC tipizzato e schermata di verifica sui dati reali"
```

---

### Task 12: `scan-conventions.mjs` and the final gate

**Files:**
- Create: `ui/scripts/scan-conventions.mjs`

Enforces the three rules no linter covers. Without this script they remain good intentions:
confirmed that `eslint-plugin-vue` offers no official rule for `<style>` blocks nor for
enforcing `<script setup>`.

- [ ] **Step 1: write the script**

`ui/scripts/scan-conventions.mjs`:

```js
import { readdirSync, readFileSync, statSync } from 'node:fs'
import { join, relative } from 'node:path'

const ROOT = new URL('..', import.meta.url).pathname
const SRC = join(ROOT, 'src')
const IPC_DIR = join('src', 'lib', 'ipc')

const STYLE_ALLOWED = /-webkit-app-region|@keyframes|::-webkit-scrollbar/

const walk = (dir) =>
  readdirSync(dir).flatMap((name) => {
    const full = join(dir, name)
    return statSync(full).isDirectory() ? walk(full) : [full]
  })

const checks = [
  {
    name: 'style block not allowed',
    test: (file, body) =>
      file.endsWith('.vue') &&
      /<style[^>]*>([\s\S]*?)<\/style>/.exec(body) !== null &&
      !STYLE_ALLOWED.test(/<style[^>]*>([\s\S]*?)<\/style>/.exec(body)[1]),
  },
  {
    name: 'invoke() outside src/lib/ipc/',
    test: (file, body) => /\binvoke\s*\(/.test(body) && !relative(ROOT, file).startsWith(IPC_DIR),
  },
  { name: 'arbitrary pixel value in a class', test: (_f, body) => /\[\d+px\]/.test(body) },
  { name: 'hardcoded opacity', test: (_f, body) => /\bopacity-(?!0\b|100\b)\d+/.test(body) },
  { name: 'hardcoded duration', test: (_f, body) => /\bduration-\d+/.test(body) },
  { name: 'size prop on an icon: use size-*', test: (_f, body) => /:size="\d+"/.test(body) },
]

const violations = walk(SRC)
  .filter((f) => /\.(vue|ts)$/.test(f))
  .flatMap((file) => {
    const body = readFileSync(file, 'utf8')
    return checks.filter((c) => c.test(file, body)).map((c) => `${relative(ROOT, file)}: ${c.name}`)
  })

violations.forEach((v) => console.error(v))
console.log(`${violations.length} violations`)
process.exit(violations.length === 0 ? 0 : 1)
```

- [ ] **Step 2: run it**

```bash
cd ui && pnpm scan
```
Expected: `0 violations`, exit code 0. If one is flagged on the verification screen, fix the
code, not the script.

- [ ] **Step 3: verify the script actually catches violations**

Temporarily introduce `<style scoped>.x{color:red}</style>` in `App.vue`, rerun it, and
verify it gets flagged with exit code 1. Then remove it.

- [ ] **Step 4: full gate**

```bash
cargo test --workspace
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cd ui && pnpm typecheck && pnpm lint && pnpm format:check && pnpm scan
```

- [ ] **Step 5: commit and status update**

Check off the `ipc` module and the Tauri app in `docs/STATUS.md`, and write the report in
`docs/superpowers/plans/2026-09-02-app-shell-ipc-report.md` as done for the other three
modules.

```bash
git add ui/scripts docs/STATUS.md docs/superpowers/plans
git commit -m "ui: convention scanning and closing out the skeleton"
```

---

## Checking the plan against the spec

- **IPC boundary without paths** → task 2 (`CandidateView`, test on the account id), 9
  (commands with no path arguments), 7 (test that the JSON contains neither `bytes` nor
  `offset`).
- **Resolution rule, five lines** → task 3, one test per line plus the case of "a single
  candidate that differs from the saved one".
- **Matrix with unknown cells** → task 4 (19 unlocated cells), 5 (`Unknown`,
  `Unexpected`, truncated section), 6 (cross-check).
- **Counts read from the file** → task 7, test that compares against
  `save.section().count`.
- **Persistence of the choice** → task 9 (`store.rs`), spec criterion 5 verified by hand at
  task 11.
- **Frontend conventions applied from the first commit** → task 10 (tooling), 11 (screen),
  12 (scan).
- **Degradation** → task 5 (short section), 9 (malformed settings), 3 (no fallback).
