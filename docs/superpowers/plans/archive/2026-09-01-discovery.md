# discovery Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A Rust crate `discovery` that, on any PC, locates Steam, the Isaac game folder (appid 250900), and the save files, deducing edition and DLCs, without ever choosing on the user's behalf and without failing.

**Architecture:** Pure resolver `discover(&Options) -> Discovery`. The sensitive logic (edition map, filename parsing, filesystem scanning) lives in pure functions that receive the roots as arguments, tested with real fixture strings and fake trees in `tempdir`. Integration with the external crates (`steamlocate`, `keyvalues-parser`, `winreg`) is isolated in a few functions, validated by the integration test on the real machine.

**Tech Stack:** Rust (edition 2021), `steamlocate`, `keyvalues-parser`, `winreg` (Windows only), `dirs`, `serde`; dev-dep `tempfile`.

## Global Constraints

- **Must work on an unknown PC:** no hard-coded paths, account ids, or drives in the production logic. The real data from the development machine is only test fixture.
- **Pure resolver:** `discover` enumerates ALL candidates, never picks one. It does not read the contents of the saves.
- **Degrade, never fail:** `discover` never returns `Err`; every missing state or I/O error becomes a `Diagnostic` and everything else proceeds. No `panic!`/`unwrap`/unchecked indexing on untrusted input or filesystem (tests may use `unwrap`).
- **Manual fallback at every step:** `Options` provides overrides for Steam, the game, and the saves.
- **Read-only:** no writes to the user's disk.
- **DLC→edition map (verified):** Afterbirth `401920`, Afterbirth+ `570660`, Repentance `1426300`, Repentance+ `3353470`; edition = the highest owned DLC, base = `Rebirth`.
- **Gate before considering a task done:** `cargo test -p discovery`, `cargo fmt --check`, `cargo clippy -p discovery --all-targets -- -D warnings`, all with no warnings.
- Windows environment, PowerShell shell; `cargo`/`git` cross-shell.

---

## File Structure

```
crates/discovery/
  Cargo.toml
  src/
    lib.rs        # tutti i tipi pubblici + enum (serde), discover() orchestratore
    edition.rs    # costanti appid DLC, edition_from_appids, dlcs_from_appids (puri)
    saves.rs      # parse_save_filename (puro) + scan_dir/scan_userdata/scan_documents (fs)
    game.rs       # parse_manifest (keyvalues-parser) + find_game (steamlocate)
    steam.rs      # find_steam (steamlocate -> winreg -> override)
  tests/
    pure.rs       # edition, filename, manifest: fixture reali, girano sempre
    scan.rs       # scansione su alberi finti in tempdir, gira sempre
    real_machine.rs  # discover(default) end-to-end, skip se Steam/Isaac assenti
```

Decomposition note: the model enums (`Edition`, `Dlc`, `SavePrefix`, `SaveSource`, `SteamSource`, `Diagnostic`) and structs (`Discovery`, `Options`, `SteamInstall`, `GameInstall`, `SaveCandidate`) all live in `lib.rs` to avoid cross-references between modules; the modules only contain functions that operate on those types.

---

### Task 1: Crate scaffold, dependencies and public types

**Files:**
- Create: `crates/discovery/Cargo.toml`
- Create: `crates/discovery/src/lib.rs`

**Interfaces:**
- Consumes: nothing.
- Produces: all the public model types (see below), `discover(&Options) -> Discovery` as a stub returning an empty `Discovery`, module declarations.

- [ ] **Step 1: Create the crate and add the dependencies**

The workspace root `Cargo.toml` already has `members = ["crates/*"]`, so the new crate is picked up automatically. Create `crates/discovery/Cargo.toml`:

```toml
[package]
name = "discovery"
version = "0.1.0"
edition = "2021"
description = "Locates Steam, the Isaac game and the saves on any PC"

[dependencies]
serde = { version = "1", features = ["derive"] }

[dev-dependencies]
```

Then, from the repo root, let cargo resolve the correct versions (do NOT write versions by hand — avoid guessing):

```bash
cargo add -p discovery steamlocate keyvalues-parser dirs
cargo add -p discovery --target 'cfg(windows)' winreg
cargo add -p discovery --dev tempfile
```

- [ ] **Step 2: Write `lib.rs` with the public types and the `discover` stub**

```rust
//! discovery — locates Steam, the Isaac game (250900) and the saves, on any PC.

use std::path::PathBuf;
use std::time::SystemTime;

use serde::Serialize;

mod edition;
mod game;
mod saves;
mod steam;

/// Overrides supplied by the caller/UI for manual fallback. All optional.
#[derive(Debug, Default, Clone)]
pub struct Options {
    pub steam_root: Option<PathBuf>,
    pub game_dir: Option<PathBuf>,
    pub save_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Discovery {
    pub steam: Option<SteamInstall>,
    pub game: Option<GameInstall>,
    pub saves: Vec<SaveCandidate>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SteamInstall {
    pub root: PathBuf,
    pub source: SteamSource,
    pub libraries: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SteamSource {
    SteamLocate,
    Registry,
    Override,
}

#[derive(Debug, Clone, Serialize)]
pub struct GameInstall {
    pub dir: PathBuf,
    pub library: PathBuf,
    pub manifest: PathBuf,
    pub edition: Edition,
    pub dlcs: Vec<Dlc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Edition {
    Rebirth,
    Afterbirth,
    AfterbirthPlus,
    Repentance,
    RepentancePlus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Dlc {
    Afterbirth,
    AfterbirthPlus,
    Repentance,
    RepentancePlus,
}

#[derive(Debug, Clone, Serialize)]
pub struct SaveCandidate {
    pub path: PathBuf,
    pub slot: u8,
    pub source: SaveSource,
    pub prefix: SavePrefix,
    pub modified: Option<SystemTime>,
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum SaveSource {
    SteamCloud { account_id: String },
    Documents { folder: PathBuf },
    Override,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SavePrefix {
    Rep,
    RepPlus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Diagnostic {
    SteamNotFound,
    GameNotFound,
    NoSavesFound,
    UnreadablePath { path: PathBuf, reason: String },
    MalformedManifest { path: PathBuf },
}

/// Entry point. Enumerates everything it finds; never chooses; never returns `Err`.
pub fn discover(_opts: &Options) -> Discovery {
    Discovery {
        steam: None,
        game: None,
        saves: Vec::new(),
        diagnostics: Vec::new(),
    }
}
```

- [ ] **Step 3: Create the four empty modules**

So that `lib.rs` compiles with `mod edition; mod game; mod saves; mod steam;`, create four files each with just a comment:

`crates/discovery/src/edition.rs`:
```rust
//! DLC→edition map and pure helpers.
```
`crates/discovery/src/saves.rs`:
```rust
//! Filename parsing and scanning of the save folders.
```
`crates/discovery/src/game.rs`:
```rust
//! Game folder and edition from the appmanifest.
```
`crates/discovery/src/steam.rs`:
```rust
//! Locating the Steam root.
```

- [ ] **Step 4: Build**

Run: `cargo build -p discovery`
Expected: PASS (the "field is never read" warning is allowed for now). The dependencies resolve.

- [ ] **Step 5: Commit**

```bash
git add crates/discovery/Cargo.toml crates/discovery/src/ Cargo.lock
git commit -m "discovery: scaffold del crate, dipendenze e tipi pubblici"
```

---

### Task 2: DLC→edition map (pure)

**Files:**
- Modify: `crates/discovery/src/edition.rs`
- Create: `crates/discovery/tests/pure.rs`

**Interfaces:**
- Consumes: `Edition`, `Dlc` from `lib.rs`.
- Produces (crate-visible):
  - `pub(crate) const DLC_AFTERBIRTH/DLC_AFTERBIRTH_PLUS/DLC_REPENTANCE/DLC_REPENTANCE_PLUS: u32`
  - `pub(crate) fn edition_from_appids(appids: &BTreeSet<u32>) -> Edition`
  - `pub(crate) fn dlcs_from_appids(appids: &BTreeSet<u32>) -> Vec<Dlc>`

- [ ] **Step 1: Write the test in `tests/pure.rs`**

Integration tests can't see `pub(crate)` items. To test the pure functions, expose a small test API in `lib.rs` behind `#[doc(hidden)]`. Add at the bottom of `lib.rs`:

```rust
/// Internal API exposed only for integration tests. Not part of the public contract.
#[doc(hidden)]
pub mod testing {
    use std::collections::BTreeSet;

    use crate::{Dlc, Edition};

    pub fn edition_from_appids(appids: &BTreeSet<u32>) -> Edition {
        crate::edition::edition_from_appids(appids)
    }
    pub fn dlcs_from_appids(appids: &BTreeSet<u32>) -> Vec<Dlc> {
        crate::edition::dlcs_from_appids(appids)
    }
}
```

Create `crates/discovery/tests/pure.rs`:

```rust
use std::collections::BTreeSet;

use discovery::testing::{dlcs_from_appids, edition_from_appids};
use discovery::{Dlc, Edition};

fn set(ids: &[u32]) -> BTreeSet<u32> {
    ids.iter().copied().collect()
}

#[test]
fn edition_is_highest_owned_dlc() {
    assert_eq!(edition_from_appids(&set(&[])), Edition::Rebirth);
    assert_eq!(edition_from_appids(&set(&[401920])), Edition::Afterbirth);
    assert_eq!(edition_from_appids(&set(&[401920, 570660])), Edition::AfterbirthPlus);
    assert_eq!(edition_from_appids(&set(&[401920, 570660, 1426300])), Edition::Repentance);
    assert_eq!(
        edition_from_appids(&set(&[401920, 570660, 1426300, 3353470])),
        Edition::RepentancePlus
    );
    // Gaps in the sequence: it still counts the highest one present.
    assert_eq!(edition_from_appids(&set(&[3353470])), Edition::RepentancePlus);
}

#[test]
fn dlcs_lists_all_owned_in_order() {
    assert_eq!(dlcs_from_appids(&set(&[])), Vec::<Dlc>::new());
    assert_eq!(
        dlcs_from_appids(&set(&[401920, 570660, 1426300, 3353470])),
        vec![Dlc::Afterbirth, Dlc::AfterbirthPlus, Dlc::Repentance, Dlc::RepentancePlus]
    );
}
```

- [ ] **Step 2: Verify the failure**

Run: `cargo test -p discovery --test pure`
Expected: FAILS to compile (`edition::edition_from_appids` not defined).

- [ ] **Step 3: Implement `edition.rs`**

```rust
//! DLC→edition map and pure helpers.

use std::collections::BTreeSet;

use crate::{Dlc, Edition};

pub(crate) const DLC_AFTERBIRTH: u32 = 401920;
pub(crate) const DLC_AFTERBIRTH_PLUS: u32 = 570660;
pub(crate) const DLC_REPENTANCE: u32 = 1426300;
pub(crate) const DLC_REPENTANCE_PLUS: u32 = 3353470;

/// Edition = the highest DLC owned; base = Rebirth.
pub(crate) fn edition_from_appids(appids: &BTreeSet<u32>) -> Edition {
    if appids.contains(&DLC_REPENTANCE_PLUS) {
        Edition::RepentancePlus
    } else if appids.contains(&DLC_REPENTANCE) {
        Edition::Repentance
    } else if appids.contains(&DLC_AFTERBIRTH_PLUS) {
        Edition::AfterbirthPlus
    } else if appids.contains(&DLC_AFTERBIRTH) {
        Edition::Afterbirth
    } else {
        Edition::Rebirth
    }
}

/// All owned DLCs, in ascending order.
pub(crate) fn dlcs_from_appids(appids: &BTreeSet<u32>) -> Vec<Dlc> {
    let mut v = Vec::new();
    if appids.contains(&DLC_AFTERBIRTH) {
        v.push(Dlc::Afterbirth);
    }
    if appids.contains(&DLC_AFTERBIRTH_PLUS) {
        v.push(Dlc::AfterbirthPlus);
    }
    if appids.contains(&DLC_REPENTANCE) {
        v.push(Dlc::Repentance);
    }
    if appids.contains(&DLC_REPENTANCE_PLUS) {
        v.push(Dlc::RepentancePlus);
    }
    v
}
```

- [ ] **Step 4: Run the tests**

Run: `cargo test -p discovery --test pure`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/discovery/src/edition.rs crates/discovery/src/lib.rs crates/discovery/tests/pure.rs
git commit -m "discovery: mappa DLC->edizione"
```

---

### Task 3: Save filename parsing (pure)

**Files:**
- Modify: `crates/discovery/src/saves.rs`
- Modify: `crates/discovery/src/lib.rs` (exposure in `testing`)
- Modify: `crates/discovery/tests/pure.rs`

**Interfaces:**
- Consumes: `SavePrefix` from `lib.rs`.
- Produces: `pub(crate) fn parse_save_filename(name: &str) -> Option<(SavePrefix, u8)>`.

- [ ] **Step 1: Add the test in `tests/pure.rs`**

First, expose the function to the tests: add inside `pub mod testing` in `lib.rs`:

```rust
    use crate::SavePrefix;

    pub fn parse_save_filename(name: &str) -> Option<(SavePrefix, u8)> {
        crate::saves::parse_save_filename(name)
    }
```

Then in `tests/pure.rs` add at the top `use discovery::SavePrefix;` and `use discovery::testing::parse_save_filename;`, and at the bottom:

```rust
#[test]
fn parses_valid_save_filenames() {
    assert_eq!(parse_save_filename("rep+persistentgamedata1.dat"), Some((SavePrefix::RepPlus, 1)));
    assert_eq!(parse_save_filename("rep_persistentgamedata2.dat"), Some((SavePrefix::Rep, 2)));
    assert_eq!(parse_save_filename("rep+persistentgamedata3.dat"), Some((SavePrefix::RepPlus, 3)));
}

#[test]
fn rejects_non_save_filenames() {
    // Dated backups: they are NOT profile candidates (they have a date prefix).
    assert_eq!(parse_save_filename("20250626.rep+persistentgamedata1.dat"), None);
    assert_eq!(parse_save_filename("options.ini"), None);
    assert_eq!(parse_save_filename("rep+persistentgamedata.dat"), None); // missing the slot
    assert_eq!(parse_save_filename("rep+persistentgamedata12.dat"), None); // slot not a single digit
    assert_eq!(parse_save_filename("rep+persistentgamedata0.dat"), None); // invalid slot 0
    assert_eq!(parse_save_filename("persistentgamedata1.dat"), None); // missing the prefix
}
```

- [ ] **Step 2: Verify the failure**

Run: `cargo test -p discovery --test pure`
Expected: FAILS to compile (`saves::parse_save_filename` not defined).

- [ ] **Step 3: Implement `parse_save_filename` in `saves.rs`**

```rust
//! Filename parsing and scanning of the save folders.

use crate::SavePrefix;

/// `rep+persistentgamedata1.dat` → `(RepPlus, 1)`; `rep_persistentgamedata2.dat` → `(Rep, 2)`.
/// Requires the exact prefix, `persistentgamedata`, a single slot digit 1..=9, then `.dat`.
/// Dated backups (`2025….dat`) and any other name don't match.
pub(crate) fn parse_save_filename(name: &str) -> Option<(SavePrefix, u8)> {
    let stem = name.strip_suffix(".dat")?;
    let (prefix, rest) = if let Some(rest) = stem.strip_prefix("rep+") {
        (SavePrefix::RepPlus, rest)
    } else if let Some(rest) = stem.strip_prefix("rep_") {
        (SavePrefix::Rep, rest)
    } else {
        return None;
    };
    let digits = rest.strip_prefix("persistentgamedata")?;
    if digits.len() != 1 {
        return None;
    }
    let slot: u8 = digits.parse().ok()?;
    if slot == 0 {
        return None;
    }
    Some((prefix, slot))
}
```

- [ ] **Step 4: Run the tests**

Run: `cargo test -p discovery --test pure`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/discovery/src/saves.rs crates/discovery/src/lib.rs crates/discovery/tests/pure.rs
git commit -m "discovery: save file name parsing"
```

---

### Task 4: Appmanifest `.acf` parsing (keyvalues-parser integration)

**⚠️ Crate integration task:** the code below is a REFERENCE. The exact `keyvalues-parser` API depends on the version cargo resolves. Verify the real API with `cargo doc -p keyvalues-parser --open` (or on docs.rs for the version in `Cargo.lock`) and ADAPT the implementation. The test fixture with the real `.acf` string is the invariant contract: when it passes, the adaptation is correct. Do not change the function signature or the test.

**Files:**
- Modify: `crates/discovery/src/game.rs`
- Modify: `crates/discovery/src/lib.rs` (exposure in `testing`)
- Modify: `crates/discovery/tests/pure.rs`

**Interfaces:**
- Consumes: nothing from the public types (uses only `std` + `keyvalues-parser`).
- Produces: `pub(crate) struct Manifest { pub installdir: String, pub dlc_appids: std::collections::BTreeSet<u32> }` and `pub(crate) fn parse_manifest(text: &str) -> Option<Manifest>`.

- [ ] **Step 1: Add the test in `tests/pure.rs`**

Expose to the tests: inside `pub mod testing` in `lib.rs` add:

```rust
    use std::collections::BTreeSet;

    pub fn parse_manifest_fields(text: &str) -> Option<(String, BTreeSet<u32>)> {
        crate::game::parse_manifest(text).map(|m| (m.installdir, m.dlc_appids))
    }
```

In `tests/pure.rs` add:

```rust
const REAL_ACF: &str = r#"
"AppState"
{
	"appid"		"250900"
	"name"		"The Binding of Isaac: Rebirth"
	"installdir"		"The Binding of Isaac Rebirth"
	"InstalledDepots"
	{
		"250902"
		{
			"manifest"		"4994611894646808503"
			"size"		"326498528"
		}
		"250905"
		{
			"manifest"		"1709017229885880564"
			"size"		"165283102"
			"dlcappid"		"401920"
		}
		"250908"
		{
			"manifest"		"7333987924869605149"
			"size"		"147989670"
			"dlcappid"		"570660"
		}
		"250911"
		{
			"manifest"		"7652847940910762229"
			"size"		"655313480"
			"dlcappid"		"1426300"
		}
		"3353471"
		{
			"manifest"		"229910742625134068"
			"size"		"763003388"
			"dlcappid"		"3353470"
		}
	}
}
"#;

#[test]
fn parses_installdir_and_dlc_appids_from_real_acf() {
    let (installdir, dlcs) = discovery::testing::parse_manifest_fields(REAL_ACF).expect("valid manifest");
    assert_eq!(installdir, "The Binding of Isaac Rebirth");
    assert_eq!(dlcs, [401920u32, 570660, 1426300, 3353470].into_iter().collect());
}

#[test]
fn malformed_acf_returns_none() {
    assert_eq!(discovery::testing::parse_manifest_fields("questo non è vdf {{{"), None);
}
```

- [ ] **Step 2: Verify the failure**

Run: `cargo test -p discovery --test pure`
Expected: FAILS to compile (`game::parse_manifest` not defined).

- [ ] **Step 3: Implement `parse_manifest` in `game.rs` (reference — adapt to the real API)**

```rust
//! Game folder and edition from the appmanifest.

use std::collections::BTreeSet;

use keyvalues_parser::Vdf;

pub(crate) struct Manifest {
    pub installdir: String,
    pub dlc_appids: BTreeSet<u32>,
}

/// Extracts `installdir` and the depots' `dlcappid` from an appmanifest `.acf`.
/// Tolerant: a malformed file → `None`, never panics.
pub(crate) fn parse_manifest(text: &str) -> Option<Manifest> {
    let vdf = Vdf::parse(text).ok()?;
    let app_state = vdf.value.get_obj()?;

    let installdir = app_state
        .get("installdir")
        .and_then(|vals| vals.first())
        .and_then(|v| v.get_str())
        .map(str::to_owned)?;

    let mut dlc_appids = BTreeSet::new();
    if let Some(depots) = app_state
        .get("InstalledDepots")
        .and_then(|vals| vals.first())
        .and_then(|v| v.get_obj())
    {
        for depot_vals in depots.values() {
            let Some(depot) = depot_vals.first().and_then(|v| v.get_obj()) else {
                continue;
            };
            if let Some(id) = depot
                .get("dlcappid")
                .and_then(|vals| vals.first())
                .and_then(|v| v.get_str())
                .and_then(|s| s.parse::<u32>().ok())
            {
                dlc_appids.insert(id);
            }
        }
    }

    Some(Manifest {
        installdir,
        dlc_appids,
    })
}
```

If `get_obj`/`get_str` don't exist under these names in the resolved version, look for the equivalents (often `Value::Obj(_)`/`Value::Str(_)` via match, or `expect_obj`/`unwrap_obj` methods). Keep the "malformed → None" behavior.

- [ ] **Step 4: Run the tests**

Run: `cargo test -p discovery --test pure`
Expected: PASS (`parses_installdir_and_dlc_appids_from_real_acf`, `malformed_acf_returns_none`).

- [ ] **Step 5: Commit**

```bash
git add crates/discovery/src/game.rs crates/discovery/src/lib.rs crates/discovery/tests/pure.rs
git commit -m "discovery: parsing dell'appmanifest .acf (installdir + DLC)"
```

---

### Task 5: Filesystem save scanning

**Files:**
- Modify: `crates/discovery/src/saves.rs`
- Modify: `crates/discovery/src/lib.rs` (exposure in `testing`)
- Create: `crates/discovery/tests/scan.rs`

**Interfaces:**
- Consumes: `SaveCandidate`, `SaveSource`, `SavePrefix`, `Diagnostic` from `lib.rs`; `parse_save_filename`.
- Produces:
  - `pub(crate) fn scan_dir(dir: &Path, source: &dyn Fn() -> SaveSource) -> (Vec<SaveCandidate>, Vec<Diagnostic>)`
  - `pub(crate) fn scan_userdata(steam_root: &Path) -> (Vec<SaveCandidate>, Vec<Diagnostic>)`
  - `pub(crate) fn scan_documents(documents: &Path) -> (Vec<SaveCandidate>, Vec<Diagnostic>)`

- [ ] **Step 1: Write the tests in `tests/scan.rs`**

Expose to the tests: inside `pub mod testing` in `lib.rs` add:

```rust
    use std::path::Path;

    use crate::{Diagnostic, SaveCandidate, SaveSource};

    pub fn scan_userdata(steam_root: &Path) -> (Vec<SaveCandidate>, Vec<Diagnostic>) {
        crate::saves::scan_userdata(steam_root)
    }
    pub fn scan_documents(documents: &Path) -> (Vec<SaveCandidate>, Vec<Diagnostic>) {
        crate::saves::scan_documents(documents)
    }
    pub fn scan_override(dir: &Path) -> (Vec<SaveCandidate>, Vec<Diagnostic>) {
        crate::saves::scan_dir(dir, &|| SaveSource::Override)
    }
```

Create `crates/discovery/tests/scan.rs`:

```rust
use std::fs;
use std::path::Path;

use discovery::testing::{scan_documents, scan_override, scan_userdata};
use discovery::{Diagnostic, SavePrefix, SaveSource};

fn touch(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, b"x").unwrap();
}

#[test]
fn scans_all_accounts_in_userdata() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    touch(&root.join("userdata/111/250900/remote/rep+persistentgamedata1.dat"));
    touch(&root.join("userdata/111/250900/remote/rep+persistentgamedata2.dat"));
    touch(&root.join("userdata/222/250900/remote/rep_persistentgamedata1.dat"));
    // Noise that must not get in:
    touch(&root.join("userdata/111/250900/remote/options.ini"));
    touch(&root.join("userdata/333/999999/remote/rep+persistentgamedata1.dat"));

    let (mut saves, diags) = scan_userdata(root);
    saves.sort_by_key(|c| c.path.clone());
    assert_eq!(saves.len(), 3, "two accounts with Isaac, three valid files");
    assert!(diags.is_empty(), "no errors: {diags:?}");

    let accounts: Vec<_> = saves
        .iter()
        .filter_map(|c| match &c.source {
            SaveSource::SteamCloud { account_id } => Some(account_id.clone()),
            _ => None,
        })
        .collect();
    assert!(accounts.contains(&"111".to_string()));
    assert!(accounts.contains(&"222".to_string()));
    assert!(!accounts.contains(&"333".to_string()), "999999 is not Isaac");

    let slot2 = saves.iter().find(|c| c.slot == 2).unwrap();
    assert_eq!(slot2.prefix, SavePrefix::RepPlus);
    assert!(slot2.size > 0);
}

#[test]
fn missing_userdata_is_not_an_error() {
    let tmp = tempfile::tempdir().unwrap();
    let (saves, diags) = scan_userdata(tmp.path()); // no userdata folder
    assert!(saves.is_empty());
    assert!(diags.is_empty(), "the absence of userdata is normal, not an error");
}

#[test]
fn scans_documents_both_folder_names() {
    let tmp = tempfile::tempdir().unwrap();
    let docs = tmp.path();
    touch(&docs.join("My Games/Binding of Isaac Repentance+/rep+persistentgamedata1.dat"));
    touch(&docs.join("My Games/Binding of Isaac Repentance/rep_persistentgamedata1.dat"));
    // Dated backups must NOT get in:
    touch(&docs.join("My Games/Binding of Isaac Repentance+/save_backups/20250626.rep+persistentgamedata1.dat"));

    let (saves, diags) = scan_documents(docs);
    assert_eq!(saves.len(), 2, "one file per folder, backups excluded");
    assert!(saves.iter().all(|c| matches!(c.source, SaveSource::Documents { .. })));
    assert!(diags.is_empty());
}

#[test]
fn override_dir_yields_override_source() {
    let tmp = tempfile::tempdir().unwrap();
    touch(&tmp.path().join("rep+persistentgamedata1.dat"));
    let (saves, diags) = scan_override(tmp.path());
    assert_eq!(saves.len(), 1);
    assert!(matches!(saves[0].source, SaveSource::Override));
    assert!(diags.is_empty());
    let _ = Diagnostic::NoSavesFound; // type in use
}
```

- [ ] **Step 2: Verify the failure**

Run: `cargo test -p discovery --test scan`
Expected: FAILS to compile (scan functions not defined).

- [ ] **Step 3: Implement the scanning in `saves.rs`**

Add at the bottom of `saves.rs` (above or below `parse_save_filename`):

```rust
use std::path::Path;

use crate::{Diagnostic, SaveCandidate, SaveSource};

/// Scans a single folder for valid save files. `source` builds the source
/// for each candidate found. Missing folder = no candidates, no error;
/// other I/O errors → `Diagnostic::UnreadablePath`.
pub(crate) fn scan_dir(
    dir: &Path,
    source: &dyn Fn() -> SaveSource,
) -> (Vec<SaveCandidate>, Vec<Diagnostic>) {
    let mut saves = Vec::new();
    let mut diags = Vec::new();

    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) => {
            if err.kind() != std::io::ErrorKind::NotFound {
                diags.push(Diagnostic::UnreadablePath {
                    path: dir.to_path_buf(),
                    reason: err.to_string(),
                });
            }
            return (saves, diags);
        }
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        let Some((prefix, slot)) = parse_save_filename(name) else {
            continue;
        };
        let meta = entry.metadata().ok();
        saves.push(SaveCandidate {
            path: entry.path(),
            slot,
            prefix,
            source: source(),
            modified: meta.as_ref().and_then(|m| m.modified().ok()),
            size: meta.as_ref().map(|m| m.len()).unwrap_or(0),
        });
    }

    (saves, diags)
}

/// For each account in `<steam_root>/userdata/*`, scans `250900/remote/`.
pub(crate) fn scan_userdata(steam_root: &Path) -> (Vec<SaveCandidate>, Vec<Diagnostic>) {
    let userdata = steam_root.join("userdata");
    let mut saves = Vec::new();
    let mut diags = Vec::new();

    let entries = match std::fs::read_dir(&userdata) {
        Ok(entries) => entries,
        Err(err) => {
            if err.kind() != std::io::ErrorKind::NotFound {
                diags.push(Diagnostic::UnreadablePath {
                    path: userdata,
                    reason: err.to_string(),
                });
            }
            return (saves, diags);
        }
    };

    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let account_id = entry.file_name().to_string_lossy().into_owned();
        let remote = entry.path().join("250900").join("remote");
        let (mut c, mut d) = scan_dir(&remote, &|| SaveSource::SteamCloud {
            account_id: account_id.clone(),
        });
        saves.append(&mut c);
        diags.append(&mut d);
    }

    (saves, diags)
}

/// Scans the two possible folders in Documents (with and without the `+`).
pub(crate) fn scan_documents(documents: &Path) -> (Vec<SaveCandidate>, Vec<Diagnostic>) {
    let mut saves = Vec::new();
    let mut diags = Vec::new();

    for folder in ["Binding of Isaac Repentance+", "Binding of Isaac Repentance"] {
        let dir = documents.join("My Games").join(folder);
        let dir_for_source = dir.clone();
        let (mut c, mut d) = scan_dir(&dir, &|| SaveSource::Documents {
            folder: dir_for_source.clone(),
        });
        saves.append(&mut c);
        diags.append(&mut d);
    }

    (saves, diags)
}
```

- [ ] **Step 4: Run the tests**

Run: `cargo test -p discovery --test scan`
Expected: PASS (4 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/discovery/src/saves.rs crates/discovery/src/lib.rs crates/discovery/tests/scan.rs
git commit -m "discovery: scansione dei salvataggi (userdata multi-account, Documents, override)"
```

---

### Task 6: Steam and game location (steamlocate/winreg integration)

**⚠️ Crate integration task:** the code for `steamlocate` and `winreg` is a REFERENCE; verify the real APIs of the resolved versions (`cargo doc -p steamlocate --open`) and ADAPT. These functions have no dedicated unit tests (they depend on the environment): the contract is Task 7's integration test on the real machine. Keep the indicated signatures.

**Files:**
- Modify: `crates/discovery/src/steam.rs`
- Modify: `crates/discovery/src/game.rs`

**Interfaces:**
- Consumes: `Options`, `SteamInstall`, `SteamSource`, `GameInstall`, `Diagnostic` from `lib.rs`; `edition::{edition_from_appids, dlcs_from_appids}`; `game::parse_manifest`.
- Produces:
  - `pub(crate) fn find_steam(opts: &crate::Options) -> (Option<crate::SteamInstall>, Vec<crate::Diagnostic>)`
  - `pub(crate) fn find_game(opts: &crate::Options, steam: Option<&crate::SteamInstall>) -> (Option<crate::GameInstall>, Vec<crate::Diagnostic>)`

- [ ] **Step 1: Implement `find_steam` in `steam.rs` (reference — adapt)**

```rust
//! Locating the Steam root.

use std::path::PathBuf;

use crate::{Diagnostic, Options, SteamInstall, SteamSource};

/// Finds the Steam root and its libraries. Order: override → steamlocate → registry.
/// No source found → `Diagnostic::SteamNotFound`, `None`.
pub(crate) fn find_steam(opts: &Options) -> (Option<SteamInstall>, Vec<Diagnostic>) {
    if let Some(root) = &opts.steam_root {
        let libraries = libraries_under(root);
        return (
            Some(SteamInstall {
                root: root.clone(),
                source: SteamSource::Override,
                libraries,
            }),
            Vec::new(),
        );
    }

    // steamlocate: check the real API of the resolved version and adapt.
    if let Some(install) = locate_via_steamlocate() {
        return (Some(install), Vec::new());
    }

    if let Some(root) = steam_root_from_registry() {
        let libraries = libraries_under(&root);
        return (
            Some(SteamInstall {
                root,
                source: SteamSource::Registry,
                libraries,
            }),
            Vec::new(),
        );
    }

    (None, vec![Diagnostic::SteamNotFound])
}

/// Known libraries under a root, including ones on other drives. Reference: adapt to
/// however `steamlocate` exposes libraries, or read `steamapps/libraryfolders.vdf`.
fn libraries_under(root: &std::path::Path) -> Vec<PathBuf> {
    // At least the main library under the root; steamlocate adds the others.
    vec![root.to_path_buf()]
}

/// Reference: replace with the real call to steamlocate for the resolved version.
fn locate_via_steamlocate() -> Option<SteamInstall> {
    // Indicative example (adapt):
    //   let dir = steamlocate::SteamDir::locate().ok()?;
    //   let libraries = dir.libraries()?... .map(|l| l.path().to_path_buf()).collect();
    //   Some(SteamInstall { root: dir.path().to_path_buf(), source: SteamSource::SteamLocate, libraries })
    None
}

#[cfg(windows)]
fn steam_root_from_registry() -> Option<PathBuf> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey(r"Software\Valve\Steam").ok()?;
    let path: String = key.get_value("SteamPath").ok()?;
    Some(PathBuf::from(path))
}

#[cfg(not(windows))]
fn steam_root_from_registry() -> Option<PathBuf> {
    None
}
```

> Note: `locate_via_steamlocate` returns `None` until you wire up the real API. In that case, on a real machine the registry fallback kicks in, which still finds the Steam root but with `libraries` = only the main root. Wire up steamlocate to cover libraries on other drives (the case of the game being on `D:`).

- [ ] **Step 2: Implement `find_game` in `game.rs` (reference — adapt)**

Add to `game.rs`, below `parse_manifest`:

```rust
use std::path::{Path, PathBuf};

use crate::edition::{dlcs_from_appids, edition_from_appids};
use crate::{Diagnostic, GameInstall, Options, SteamInstall};

const APPID: u32 = 250900;

/// Finds the game folder. Override → otherwise looks for 250900 in all libraries.
pub(crate) fn find_game(
    opts: &Options,
    steam: Option<&SteamInstall>,
) -> (Option<GameInstall>, Vec<Diagnostic>) {
    if let Some(dir) = &opts.game_dir {
        // With an override, try reading a manifest alongside it if there is one; otherwise Rebirth.
        return (Some(game_from_dir(dir.clone(), dir.clone(), None)), Vec::new());
    }

    let Some(steam) = steam else {
        return (None, vec![Diagnostic::GameNotFound]);
    };

    let mut diags = Vec::new();
    for library in &steam.libraries {
        let manifest = library
            .join("steamapps")
            .join(format!("appmanifest_{APPID}.acf"));
        let text = match std::fs::read_to_string(&manifest) {
            Ok(text) => text,
            Err(_) => continue, // 250900 isn't in this library
        };
        let Some(parsed) = parse_manifest(&text) else {
            diags.push(Diagnostic::MalformedManifest {
                path: manifest.clone(),
            });
            continue;
        };
        let dir = library
            .join("steamapps")
            .join("common")
            .join(&parsed.installdir);
        return (
            Some(game_from_dir(dir, library.clone(), Some((manifest, parsed)))),
            diags,
        );
    }

    diags.push(Diagnostic::GameNotFound);
    (None, diags)
}

fn game_from_dir(
    dir: PathBuf,
    library: PathBuf,
    parsed: Option<(PathBuf, Manifest)>,
) -> GameInstall {
    match parsed {
        Some((manifest, m)) => GameInstall {
            dir,
            library,
            manifest,
            edition: edition_from_appids(&m.dlc_appids),
            dlcs: dlcs_from_appids(&m.dlc_appids),
        },
        None => GameInstall {
            manifest: PathBuf::new(),
            edition: crate::Edition::Rebirth,
            dlcs: Vec::new(),
            dir,
            library,
        },
    }
}

// Silences the warning if `Path` isn't used elsewhere.
#[allow(unused_imports)]
use Path as _Path;
```

> If `use Path as _Path;` generates a useless warning, remove it along with the unused `Path` import. Don't leave dead imports.

- [ ] **Step 3: Build and check the gates (no unit tests here)**

Run: `cargo build -p discovery` then `cargo clippy -p discovery --all-targets -- -D warnings`
Expected: PASS with no warnings. If clippy flags dead imports/variables, clean them up.

- [ ] **Step 4: Commit**

```bash
git add crates/discovery/src/steam.rs crates/discovery/src/game.rs
git commit -m "discovery: locating Steam (steamlocate/winreg) and the game"
```

---

### Task 7: `discover` orchestration and real integration test

**Files:**
- Modify: `crates/discovery/src/lib.rs` (body of `discover`)
- Create: `crates/discovery/tests/real_machine.rs`

**Interfaces:**
- Consumes: `steam::find_steam`, `game::find_game`, `saves::{scan_dir, scan_userdata, scan_documents}`.
- Produces: the complete `discover(&Options) -> Discovery`.

- [ ] **Step 1: Write the integration test in `tests/real_machine.rs`**

```rust
use discovery::{discover, Options};

/// End-to-end on the real machine. Doesn't hardcode paths: asserts THAT it finds, not WHERE.
/// Skips with a note if Steam or Isaac aren't present (CI, machines without the game).
#[test]
fn discovers_game_and_at_least_one_save_if_present() {
    let d = discover(&Options::default());

    let Some(steam) = &d.steam else {
        eprintln!("skip: Steam not found on this machine");
        return;
    };
    eprintln!("steam root: {:?}, libraries: {}", steam.root, steam.libraries.len());

    let Some(game) = &d.game else {
        eprintln!("skip: Isaac (250900) not installed");
        return;
    };
    assert!(game.dir.exists(), "the reported game folder must exist");
    eprintln!("game: {:?} edition {:?}", game.dir, game.edition);

    // If the game is there, there's normally at least one save (cloud or Documents).
    // Not guaranteed (profile never launched), so we log instead of failing hard.
    if d.saves.is_empty() {
        eprintln!("note: no saves found (profile never launched?)");
    } else {
        eprintln!("saves found: {}", d.saves.len());
        for s in &d.saves {
            assert!(s.path.exists(), "every candidate must point to a file that exists");
        }
    }
}
```

- [ ] **Step 2: Implement the body of `discover` in `lib.rs`**

Replace the `discover` stub with:

```rust
pub fn discover(opts: &Options) -> Discovery {
    let mut diagnostics = Vec::new();

    let (steam, mut steam_diags) = steam::find_steam(opts);
    diagnostics.append(&mut steam_diags);

    let (game, mut game_diags) = game::find_game(opts, steam.as_ref());
    diagnostics.append(&mut game_diags);

    let mut saves = Vec::new();

    if let Some(dir) = &opts.save_dir {
        let (mut c, mut d) = saves::scan_dir(dir, &|| SaveSource::Override);
        saves.append(&mut c);
        diagnostics.append(&mut d);
    }
    if let Some(steam) = &steam {
        let (mut c, mut d) = saves::scan_userdata(&steam.root);
        saves.append(&mut c);
        diagnostics.append(&mut d);
    }
    if let Some(documents) = dirs::document_dir() {
        let (mut c, mut d) = saves::scan_documents(&documents);
        saves.append(&mut c);
        diagnostics.append(&mut d);
    }

    if saves.is_empty() {
        diagnostics.push(Diagnostic::NoSavesFound);
    }

    Discovery {
        steam,
        game,
        saves,
        diagnostics,
    }
}
```

Make sure `scan_dir` is `pub(crate)` and visible (already from Task 5). Add internal `use` statements if the compiler requires them (e.g. `SaveSource` is already in the module).

- [ ] **Step 3: Run the whole suite**

Run: `cargo test -p discovery -- --nocapture`
Expected: PASS. `pure` and `scan` always green; `real_machine` prints the paths found (game on `D:`, saves on `C:`) and passes. If on this machine the game is NOT found, `locate_via_steamlocate` needs to be wired up to the real API (see Task 6): without multiple libraries, the game on `D:` cannot be found with only the registry fallback.

- [ ] **Step 4: Final gates**

Run: `cargo fmt --check` (run `cargo fmt` if needed), then `cargo clippy -p discovery --all-targets -- -D warnings`
Expected: exit 0, no warnings.

- [ ] **Step 5: Commit**

```bash
git add crates/discovery/src/lib.rs crates/discovery/tests/real_machine.rs
git commit -m "discovery: orchestrazione di discover e integration test reale"
```

---

## Self-Review

**Spec coverage:**
- Pure resolver, enumerates everything, never chooses → Task 7 (`discover`), saves in a `Vec`. ✓
- Must work on an unknown PC, nothing hard-coded → logic operates on roots passed as arguments (Task 5), the integration test asserts *that* it finds things, not *where* (Task 7). ✓
- steamlocate/keyvalues-parser/winreg/dirs → Task 1 (deps), 4 (.acf), 6 (steamlocate/winreg). ✓
- Model and API (Options, Discovery, SteamInstall, GameInstall, SaveCandidate, enums, Diagnostic) → Task 1. ✓
- 5-step orchestration (steam→libraries→game→edition→saves) → Task 6 + 7. ✓
- Edition detection from the depots, verified map → Task 2 + 4. ✓
- Save enumeration: override + multi-account userdata + Documents (rep_/rep+, slot) → Task 5. ✓
- Filename parsing, dated backups excluded → Task 3. ✓
- Tests on three levels (pure, tempdir, real skippable) → Task 2/3/4 (pure), 5 (scan), 7 (real_machine). ✓
- Degrade, never fail, no Err, no panic → `discover` always returns a `Discovery`; tolerant scans (Task 5); malformed manifest → diagnostic (Task 4/6). ✓
- Read-only → no writes in any task. ✓

**Placeholder scan:** the two integration tasks (4, 6) contain reference code explicitly marked "adapt to the real API", with the test fixtures as the contract — these are not vague placeholders but complete implementations to verify against the resolved crate version. No "TODO"/"TBD". ✓

**Type consistency:** `Options`, `Discovery`, `SteamInstall`, `SteamSource`, `GameInstall`, `Edition`, `Dlc`, `SaveCandidate`, `SaveSource`, `SavePrefix`, `Diagnostic` used with the same signatures across all tasks; `scan_dir` takes `&dyn Fn() -> SaveSource` everywhere (Task 5 and 7); `find_steam`/`find_game` with the declared signatures (Task 6) used in `discover` (Task 7); `parse_manifest -> Option<Manifest>` (Task 4) used in `find_game` (Task 6). ✓
