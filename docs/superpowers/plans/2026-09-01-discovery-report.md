# Discovery Crate Implementation Report

**Status: DONE**

**Date:** 2026-09-01

---

## Commits

| Hash | Message |
|------|---------|
| `71c7e87` | discovery: crate scaffold, dependencies and public types |
| `d631940` | discovery: DLC->edition map |
| `91e09bb` | discovery: save filename parsing |
| `95705aa` | discovery: appmanifest .acf parsing (installdir + DLC) |
| `df6c76a` | discovery: save scanning (multi-account userdata, Documents, override) |
| `cf22497` | discovery: Steam location (steamlocate/winreg) and game location |
| `a0a067c` | discovery: discover orchestration and real integration test |

---

## Resolved Dependency Versions

- **steamlocate**: `2.1.1`
- **keyvalues-parser**: `0.2.4`
- **dirs**: `6.0.0`
- **winreg**: `0.56.0`
- **tempfile**: `3.27.0` (dev)

---

## Gate Results

### cargo test -p discovery -- --nocapture

```
running 0 tests  (lib)

running 6 tests  (pure.rs)
test parses_valid_save_filenames ... ok
test dlcs_lists_all_owned_in_order ... ok
test edition_is_highest_owned_dlc ... ok
test rejects_non_save_filenames ... ok
test malformed_acf_returns_none ... ok
test parses_installdir_and_dlc_appids_from_real_acf ... ok
test result: ok. 6 passed; 0 failed

running 1 test  (real_machine.rs)
test discovers_game_and_at_least_one_save_if_present ... ok
test result: ok. 1 passed; 0 failed

running 4 tests  (scan.rs)
test missing_userdata_is_not_an_error ... ok
test override_dir_yields_override_source ... ok
test scans_documents_both_folder_names ... ok
test scans_all_accounts_in_userdata ... ok
test result: ok. 4 passed; 0 failed

Total: 11 tests, 0 failed
```

### cargo fmt --check

Clean. No diffs.

### cargo clippy -p discovery --all-targets -- -D warnings

Clean. No warnings or errors.

---

## API Adaptations

### keyvalues-parser 0.2.4

The recommended API is `keyvalues_parser::parse(text)` (free function) returning `Result<PartialVdf>`. `Vdf::parse()` is deprecated since 0.2.4. Used `Parser::new().literal_special_chars(true).parse(text)` for the ACF manifest since Steam paths on Windows contain backslashes, then called `.into()` to convert `PartialVdf` to `Vdf`.

Navigation through parsed VDF:
- Root value accessed as `vdf.value.get_obj()` → `&Obj`
- `Obj` derefs to `BTreeMap<Cow<str>, Vec<Value>>`
- Values accessed with `.get("key").and_then(|vals| vals.first()).and_then(|v| v.get_str())`
- Nested objects with `v.get_obj()`

### steamlocate 2.1.1

Main API: `SteamDir::locate()` → `Result<SteamDir>`, then `steam_dir.library_paths()` → `Result<Vec<PathBuf>>`. The library paths already include ALL Steam library folders (not just the default one), so the game on `D:\SteamLibrary` is correctly found by iterating `steam.libraries`.

`SteamDir::find_app()` was not used — we parse appmanifest ACF files ourselves to extract the DLC appids for edition detection, which `find_app()` does not expose.

### winreg 0.56.0

Used as fallback after steamlocate. Registry key: `HKEY_LOCAL_MACHINE\SOFTWARE\Wow6432Node\Valve\Steam` (32-bit Steam on 64-bit Windows), with fallback to `SOFTWARE\Valve\Steam`. Value: `InstallPath`.

---

## Integration Test Result

On this development machine:

- **Steam root:** `C:\Program Files (x86)\Steam`
- **Libraries found:** 2 (including `D:\SteamLibrary`)
- **Game directory:** `D:\SteamLibrary\steamapps\common\The Binding of Isaac Rebirth`
- **Edition detected:** `RepentancePlus` (all 4 DLCs present)
- **Save files found:** 4

The game is indeed on `D:` while Steam is on `C:` — the multi-library enumeration via `steamlocate.library_paths()` correctly handles this case.

---

## Deviations from Plan

### Task ordering for clippy gate

The plan prescribed: implement Tasks 6 then 7, running clippy before committing Task 6. However, Task 6 introduces `find_steam()`, `find_game()`, and helper functions that are all dead code until `discover()` is wired up in Task 7. Running clippy with `-D warnings` before Task 7 fails with dead-code errors.

Resolution: implemented Task 7's `discover()` immediately before running the Task 6 clippy gate. The actual commits were staged correctly (Task 6 commit covers only steam.rs and game.rs; Task 7 commit covers lib.rs and real_machine.rs), but the clippy and fmt checks ran after the full implementation was in place.

### Task 1 initial build failure

The Cargo manifest requires at least one target (lib or bin) to be recognized as valid workspace member. Created the four stub module files (`edition.rs`, `saves.rs`, `game.rs`, `steam.rs`) before running `cargo add` — then found that `edition_from_appids` and `dlcs_from_appids` were missing from the stub `edition.rs`, causing a build failure. Resolved by implementing `edition.rs` fully as part of the scaffold step (which is consistent with Task 2's requirements anyway).

---

## Fix post-review

**Date:** 2026-09-01

### FIX 1 — HKCU registry fallback before HKLM (`src/steam.rs`)

`try_registry` now first tries `HKCU\Software\Valve\Steam`, value `SteamPath`. If that
key exists and returns a path, it is used immediately (it is the value written by Steam
itself and always consistent with the active installation). Only if that fails does it fall
back to `HKLM\SOFTWARE\Wow6432Node\Valve\Steam\InstallPath` as before. The
`#[cfg(windows)]` guard / `#[cfg(not(windows))]` stub is unchanged.

### FIX 2 — Fall back to canonical folder on malformed `.acf` (`src/game.rs`)

Added constant `CANONICAL_INSTALLDIR = "The Binding of Isaac Rebirth"`. When
`parse_manifest` returns `None`, the code now emits `MalformedManifest` (as before) and
then checks whether `<library>/steamapps/common/The Binding of Isaac Rebirth` exists: if so,
it returns the game with `edition = Rebirth` and the accumulated diagnostics, without emitting
`GameNotFound`. If the canonical folder does not exist, it moves on to the next libraries.
`GameNotFound` is now emitted only at the very end of the loop, only if no library produced
a `GameInstall`.

### FIX 3 — Expose orchestration to tests (`src/lib.rs`, `mod testing`)

Added to `pub mod testing` the two functions:
- `find_steam(opts) -> (Option<SteamInstall>, Vec<Diagnostic>)` — delegates to `crate::steam::find_steam`
- `find_game(opts, steam) -> (Option<GameInstall>, Vec<Diagnostic>)` — delegates to `crate::game::find_game`

Added `GameInstall` to the `testing` module's imports so it compiles.

### FIX 4 — Degradation tests (`tests/discover.rs`, new file)

Added 4 new tests:

| Test | What it checks |
|---|---|
| `malformed_manifest_falls_back_to_canonical_dir` | garbage `.acf` + canonical folder present → `game = Some`, `edition = Rebirth`, `MalformedManifest` present, `GameNotFound` absent |
| `malformed_manifest_without_canonical_dir_continues_to_next_library` | malformed `.acf` in lib1 with no canonical folder, valid manifest in lib2 → finds the game in lib2 with the correct edition, `MalformedManifest` present, `GameNotFound` absent |
| `finds_game_in_second_library_when_absent_from_first` | lib1 with no manifest, lib2 with a valid manifest → `game.dir` under lib2, `GameNotFound` absent |
| `scan_override_on_file_path_yields_unreadable_path` | `scan_override` pointed at a FILE (not a directory) → `UnreadablePath` present |

### FIX 5 — Spec aligned (`docs/superpowers/specs/2026-09-01-discovery-design.md`)

In point 1 of the orchestration: made explicit that the registry is read first from
`HKCU\Software\Valve\Steam\SteamPath` and then, as a further fallback, from
`HKLM\SOFTWARE\Wow6432Node\Valve\Steam\InstallPath`.

In point 3: rewrote the semantics of the malformed `.acf` — fall back to the canonical folder
`The Binding of Isaac Rebirth` with `edition = Rebirth` if it exists; otherwise move to the
next library. `GameNotFound` only at the end if no library produced a game.

---

### Post-review gate

```
cargo test -p discovery
  running 4 tests  (discover.rs)  → 4 passed
  running 6 tests  (pure.rs)      → 6 passed
  running 1 test   (real_machine) → 1 passed
  running 4 tests  (scan.rs)      → 4 passed
  Total: 15 tests, 0 failed

cargo fmt --check -p discovery    → Clean
cargo clippy -p discovery --all-targets -- -D warnings → Clean
```
