# discovery — locating Steam, the game, and the saves (design)

**Date:** 2026-09-01
**Sub-project:** M1 · second piece
**Status:** approved in brainstorming, pending spec review


> **Executed and merged.** The "pending spec review" above was never cleared and stayed on
> this document after its work shipped; it is a note about how the design was agreed, not
> about where the spec stands. The plan is archived as `docs/superpowers/plans/archive/2026-09-01-discovery.md`
> — which this project only does when a sub-project merges into `develop` — and the report is
> `docs/superpowers/reports/2026-09-01-discovery-report.md`. Read the report first: where the two disagree, it is
> the one that measured. (Noted 2026-09-15, on a pass over all 32 specs.)

## Context

Second sub-project of M1. `discovery` is the Rust crate that, on any PC, finds the Steam
installation, the folder of *The Binding of Isaac: Rebirth* (appid 250900), and the save
files, also deducing edition and owned DLCs. It is a **pure resolver**: it enumerates
what it finds along with metadata and never chooses on the user's behalf. It doesn't read
the saves (that's `core-save`'s responsibility); it returns their paths.

It feeds two consumers: `unpack` (which needs the installation folder to extract the
`.a` archives) and profile reading (which needs the save's path). On the development
machine these two paths live on **different disks** — the game on `D:\SteamLibrary`,
the saves under the main Steam on `C:` — so the two searches are treated as independent
by construction.

## Load-bearing constraint: it must work on an unknown PC

`discovery` ships inside an installer and must run without configuration on any Steam
installation of the game. This governs every decision below.

- **Nothing hardcoded to the dev machine.** No absolute path (`D:\SteamLibrary`), no
  account id, no assumption about the disk. Everything is derived at runtime. The dev
  machine's real data is *one* test fixture, never production values.
- **Every real variant covered:**
  - Steam from the registry; if the registry doesn't answer → fallback, then manual
    override.
  - Game in any library → *all* libraries from `libraryfolders.vdf` are iterated.
  - Multiple Steam accounts under `userdata` → all enumerated.
  - Steam Cloud on (save in `userdata`), off (save in Documents), or both → both sources
    scanned.
  - Folder `Binding of Isaac Repentance` and `…Repentance+`; prefix `rep_` and `rep+`;
    slot 1/2/3 → all of them.
  - Non-Steam / GOG / pirated copy → not auto-discoverable, but reachable via override.
  - Partial DLCs → edition = the highest DLC owned; never assumed to be Rep+.
  - Inconsistent states (game without saves, saves without a game, Isaac never launched
    → no `remote/`, directory unreadable due to permissions) → every part is independent
    and degrades with a `Diagnostic`, never a panic/crash.
- **Manual fallback at every step:** override for Steam, game and saves, so even the
  most atypical installation is recoverable by pointing to a path.
- **Degrade, never fail:** no read error, malformed `.vdf`/`.acf`, or inaccessible
  directory must interrupt `discover`. A `Diagnostic` is recorded and it proceeds with
  what was found.
- **Read-only:** `discovery` writes nothing to the user's disk.

## Crates and platform seam

Stack fixed by the project document:
- `steamlocate` — Steam root, library enumeration, an appid's install folder. Saves us
  hand-parsing `libraryfolders.vdf`.
- `keyvalues-parser` — reading `appmanifest_250900.acf` for the DLC depots (which
  `steamlocate` doesn't reliably expose) and, if needed, as a `.vdf` parsing fallback.
- `winreg` — fallback for the Steam root on Windows when `steamlocate` doesn't find it.

Windows-specific code (registry reads, the Documents path) sits behind a small isolated
function. The scanning functions receive the *roots* as arguments, so they are
OS-agnostic and testable with fake directories. Windows implementation now; nothing
precludes Linux/macOS later (where only the root lookup and the default paths would
change).

## Modules

```
crates/discovery/
  Cargo.toml
  src/
    lib.rs        # public types (Discovery, Options, ...), discover() that orchestrates
    steam.rs      # Steam root (steamlocate -> winreg -> override) + libraries
    game.rs       # install folder for 250900 + edition/DLCs from the .acf
    saves.rs      # enumeration of save candidates + filename parsing
    edition.rs    # Edition enum, depot->DLC map, pure helpers
  tests/
    fixtures.rs   # parsing against real strings (.vdf, .acf) and fake trees in a tempdir
    real_machine.rs  # end-to-end integration, skipped if Steam/Isaac absent
```

## Model and public API

```rust
use std::path::PathBuf;
use std::time::SystemTime;

/// Overrides supplied by the caller/UI for manual fallback. All optional.
#[derive(Debug, Default, Clone)]
pub struct Options {
    /// Manually set Steam root (skips registry and auto-search).
    pub steam_root: Option<PathBuf>,
    /// Manually set game folder (non-Steam copy, external disk).
    pub game_dir: Option<PathBuf>,
    /// Manually set folder to scan for saves.
    pub save_dir: Option<PathBuf>,
}

/// Entry point: enumerates everything it finds, without choosing.
pub fn discover(opts: &Options) -> Discovery;

#[derive(Debug, Clone, Serialize)]
pub struct Discovery {
    pub steam: Option<SteamInstall>,
    pub game: Option<GameInstall>,
    pub saves: Vec<SaveCandidate>,      // ALL candidates found
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SteamInstall {
    pub root: PathBuf,
    pub source: SteamSource,            // how the root was found
    pub libraries: Vec<PathBuf>,        // all known libraries
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SteamSource { Registry, SteamLocate, Override }

#[derive(Debug, Clone, Serialize)]
pub struct GameInstall {
    pub dir: PathBuf,                   // .../common/The Binding of Isaac Rebirth
    pub library: PathBuf,               // the library that contains it
    pub manifest: PathBuf,              // appmanifest_250900.acf
    pub edition: Edition,               // highest DLC owned
    pub dlcs: Vec<Dlc>,                 // installed DLCs, from the depots
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Edition {
    Rebirth,          // base 250900 only
    Afterbirth,       // + 401920
    AfterbirthPlus,   // + 570660
    Repentance,       // + 1426300
    RepentancePlus,   // + 3353470
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Dlc { Afterbirth, AfterbirthPlus, Repentance, RepentancePlus }

#[derive(Debug, Clone, Serialize)]
pub struct SaveCandidate {
    pub path: PathBuf,
    pub slot: u8,                       // 1/2/3, from the filename
    pub source: SaveSource,
    pub prefix: SavePrefix,             // rep_ vs rep+
    pub modified: Option<SystemTime>,
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum SaveSource {
    SteamCloud { account_id: String },  // .../userdata/<id>/250900/remote
    Documents { folder: PathBuf },      // Documents\My Games\Binding of Isaac Repentance[+]
    Override,                           // from the save_dir passed manually
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SavePrefix { Rep, RepPlus }    // rep_ | rep+

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Diagnostic {
    SteamNotFound,
    GameNotFound,
    NoSavesFound,
    UnreadablePath { path: PathBuf, reason: String },
    MalformedManifest { path: PathBuf },
}
```

All model types derive `Serialize` (for the future Tauri bridge). `discover` never
returns `Err`: degradation lives in `diagnostics`.

## Orchestration of `discover`

1. **Steam:** if `opts.steam_root` is given → `SteamSource::Override`. Otherwise try
   `steamlocate`; if that fails, read the registry: first `HKCU\Software\Valve\Steam`,
   value `SteamPath` (written by Steam itself, always up to date); if that also fails,
   fall back to `HKLM\SOFTWARE\Wow6432Node\Valve\Steam`, value `InstallPath`. If nothing
   → `Diagnostic::SteamNotFound`, `steam = None` (proceed anyway: game/save overrides
   may be enough).
2. **Libraries:** from `steamlocate` (or from the `libraryfolders.vdf` under the root
   found). Any library path that isn't readable → `Diagnostic::UnreadablePath`, skipped.
3. **Game:** if `opts.game_dir` is given → use it. Otherwise look for `250900` in *all*
   libraries in order. For each library that contains `appmanifest_250900.acf`:
   - if the file is readable and valid → use `installdir` from the manifest, return the
     game;
   - if the file is malformed → `Diagnostic::MalformedManifest`; then try the canonical
     folder `steamapps/common/The Binding of Isaac Rebirth` (the installdir is always
     the same for appid 250900): if it exists, return the game with `edition = Rebirth`
     (no DLC detectable from the manifest) and stop; otherwise proceed to the next
     library.
   `Diagnostic::GameNotFound` is only emitted at the end, if no library (nor the
   override) produced a `GameInstall`. It's never emitted alongside a game that was
   found.
4. **Edition/DLC:** from the depots' `dlcappid` → owned `Dlc`s; `edition` = the highest
   one (see map below).
5. **Saves:** union of three sources, each independent and error-tolerant:
   - `opts.save_dir` override → scan that folder;
   - Steam Cloud: for **every** account under `<root>/userdata/*`, the folder
     `250900/remote/` → files `rep*persistentgamedata<slot>.dat`;
   - Documents: `Documents\My Games\Binding of Isaac Repentance` and `…Repentance+`.
   Every file found → a `SaveCandidate`. Zero candidates → `Diagnostic::NoSavesFound`.

## Edition detection (verified against real data)

From the `.acf`'s `InstalledDepots`, field `dlcappid`:

| dlcappid  | DLC              | Edition if it's the highest |
|-----------|------------------|------------------------------|
| (none)    | base 250900      | `Rebirth`                    |
| 401920    | Afterbirth       | `Afterbirth`                 |
| 570660    | Afterbirth+      | `AfterbirthPlus`             |
| 1426300   | Repentance       | `Repentance`                 |
| 3353470   | Repentance+      | `RepentancePlus`             |

`edition` = the maximum of the `Edition` ordering over the owned DLCs. Verified: the real
`.acf` lists depots `250905/401920`, `250908/570660`, `250911/1426300`, `3353471/3353470`
→ `RepentancePlus`.

## Parsing the save filename (pure function)

`rep_persistentgamedata2.dat` → `(prefix: Rep, slot: 2)`;
`rep+persistentgamedata1.dat` → `(prefix: RepPlus, slot: 1)`.
Rule: prefix `rep_` or `rep+`, then `persistentgamedata`, then a slot digit, then `.dat`.
Names that don't match don't produce candidates (they aren't profile saves).

## Tests

### Pure functions, against real strings (always run)

- **`.acf` parsing** → `installdir` and the set of `dlcappid`s, from a fixture string
  taken from the real `.acf`. Asserts `installdir == "The Binding of Isaac Rebirth"` and
  the four expected `dlcappid`s.
- **Depot→edition map**: `{401920,570660,1426300,3353470}` → `RepentancePlus`;
  `{}` → `Rebirth`; `{401920}` → `Afterbirth`; `{401920,570660}` → `AfterbirthPlus`.
- **`libraryfolders.vdf` parsing** → list of library paths and which one contains
  `250900`, from a real fixture string (two libraries, `250900` in the second).
- **Filename parsing** → `(prefix, slot)` for `rep_`/`rep+` and slots 1/2/3; invalid
  names → no match.

### Filesystem scanning, on fake trees in `tempdir` (always run, dev-dep `tempfile`)

- Tree `userdata/<a>/250900/remote/` and `userdata/<b>/250900/remote/` with files of
  different slots → candidates include both accounts, with the correct `account_id`.
- Cloud + Documents together → candidates from both sources, distinct `source`.
- `remote/` directory absent (Isaac never launched) → no candidate from that account,
  no panic.
- Simulated unreadable directory (where possible) or nonexistent path →
  `Diagnostic::UnreadablePath`/no crash.
- Malformed `.acf` → `Diagnostic::MalformedManifest`, fallback edition.

### End-to-end integration on the real machine (skipped if absent)

`discover(&Options::default())`: if Steam and Isaac are present, it must find the game
folder (on any disk) and at least one `SaveCandidate`; otherwise it prints `skip` and
passes. It validates precisely the real "game and saves on different disks" case,
without hardcoding the paths in the assertions (it asserts *that* it finds, not
*where*).

## Out of scope

- Reading/parsing save contents → `core-save`.
- Extracting the `.a` archives → `unpack`.
- Choosing slot/account → UI (M1, later).
- Non-Windows paths and registry → beyond v1 (the seam is set up for it).

## Completion criteria

- `cargo test -p discovery` green with and without Steam/Isaac present.
- No hardcoded path, account, or disk in the production logic.
- `discover` never returns `Err`; every missing state is a `Diagnostic`.
- On the real machine, `discover(default)` finds the game on `D:` and the saves on `C:`.
- No writes to disk; no automatic choice among candidates.
