# M4, sub-project 1b — the log watcher and the run archive

**Branch:** `feature/log-watch`, cut from `develop`.
**Spec:** `docs/superpowers/specs/2026-09-12-m4-run-model-design.md`, corrected in place before the
plan was written.
**Plan:** `docs/superpowers/plans/archive/2026-09-13-log-watch.md`, 11 tasks.

1a gave the model: `Tail`, the rules file, the fold. 1b gives it a disk and a place to live —
`discovery` finds the folder, `log-watch` reads it, `store` keeps it, `ipc` shows it, and the app
fills the archive at launch and then follows the log.

---

## What the work measured, and what each measurement changed

Four of these contradicted a document. That is the whole reason they are written down.

**1. The 4 KiB prefix is the machine describing itself.** The spec identified a launch of
`log.txt` by its first 4 KiB and its length. Measured across `samples/logs/`: that window holds
OpenGL, the driver, the OpenAL banner and the path of the game's own DLL — none of it changes
between two launches on one machine. The only byte separating the two logs of 2026-09-12 inside
it is `load archives: 2633 milliseconds` against `2549`, at byte 1633. **The discriminating power
of the prefix is a millisecond timing.**

So identity rests on an **anchor**: the 64 bytes that end at the offset already consumed, which
are run content and never banner. The prefix stays as a cheap filter. The two mistakes do not
cost the same — reading a new launch as the old one loses runs, reading the old one as new
imports every run in it twice — and the anchor makes the expensive one impossible.

**2. `samples/logs/` holds three launches, not four.**
`20260908-run-megasatan-judas.log.txt` is a **strict prefix** of `20260908-s1.log.txt` (`cmp`
reports EOF on the shorter with no differing byte): one launch copied at two moments. The folder
therefore contains no pair of *different* launches at all, which is the direction the resume rule
most needs and the one real data cannot test today.

**3. `online_logs\` is not flat, and it is not 22 sessions.**

```
online_logs\sessions\MM_DD_YYYY__HH_MM_SS\           26 folders, each with log.txt
online_logs\desyncs\MM_DD_YYYY__HH_MM_SS__Name\      desync_log.txt — a crash report, not a run
online_logs\desyncs\sessions\MM_DD_YYYY__HH_MM_SS\   2 more real sessions, nested
```

**28 sessions**, checked against the real folder. A walk of the first level finds two directories
and no sessions; one that recurses into `desyncs\` and takes any folder files crash reports as
runs. Both are wrong in a way that looks like it worked, which is why `sessions()` has a test
with the real shape.

**4. `discovery` could not say where any of this is.** It reached `My Games` only to look for
`.dat` files, and the folder surfaced only inside `SaveSource::Documents { folder }` — that is,
only when a save happened to live there. With Steam Cloud on it never does. `Discovery` gains
`game_data`, probed always, not `Serialize`, and its absence is not a `Diagnostic`: that enum
reaches TypeScript as `SetupDiagnostic` and is about saves, not logs.

**5. The bug a test found, which is the one that mattered.** `head(log, PREFIX_BYTES)` on a file
**shorter than 4 KiB** returns the whole file. A log is a few hundred bytes for its first
instants, so the prefix hash changed with every line the game wrote, every read looked like a new
source, and **everything already in the archive would have been imported again**. The window is
stored with its length now and stays the window it was; two tests in `run` pin both directions.
It was found by `a_log_that_grew_appends_only_what_is_new`, written before the code.

---

## The counters: measured, and the answer is that it cannot be measured yet

The spec called the agreement with `STREAK_COUNTER [22]` *the property this sub-project is tested
by*, and named 2026-09-08 as the one window that could answer it — the other two sources were
excluded in advance (the co-op Greed session did not move counter 22 at all; `20260912-solo-judas`
is an `Open` run where nothing moving is correct).

Measured on that window:

```
runs in the log: 1 (won 1, died 0, abandoned 0)
achievements gained across the window: 0
counters that moved:                   [5] 10973 -> 10974,  [199] 1873 -> 1874
DEATHS [10]:      263 -> 263
STREAK [22]:        1 -> 1
BEST_STREAK [23]:   3 -> 3
```

**The window does not contain the run.** The reading rests on the achievements rather than the
counters, because it depends on no mapping: the log calls `unlock steam achievement` twice, and
not one of 642 slots turns on between the two saves. The dated backup of the 8th was written
before that run was played.

So nothing in the repo asserts an agreement nobody has seen. `crates/ipc/tests/runs_real.rs`
keeps both halves answerable — the log folds to one win; the window holds no unlock and fewer
than ten moved counters — and it **fails the day a window does hold a run**, which is when the
question can finally be asked. What would close it: one solo, non-Greed win with a snapshot
either side, which `core-save`'s `live_probe` takes.

---

## Run against the real folders, which found two more things

The archive was built from this machine's own `online_logs\` into a throwaway database — no
window, no app:

```
backfilled 28 sessions, 0 errors
live log: 476 events, 4 runs
archive: 95 runs — won 16, died 23, abandoned 36, open 20
second backfill imported: 0
```

**One event per `INSERT` is one commit per event.** The first pass took **212 seconds** in
release. The events of a source are now written in one transaction together with the offset they
belong to: **0.67 s**, the same 95 runs out of the same files. Speed is the smaller half of that
change — events written while the offset stays behind are events the next read finds again and
files a second time, which is the duplicate-runs failure the anchor exists to prevent, reached
through a crash instead of through a bad guess. So `append_to_log` does both or neither, and
`import_session` writes a session's row and its events together for the same reason.

**`Died` had coverage all along, in files nobody had copied.** 1a asserted that no log in
`samples/logs/` contained `Game Over`, and wrote a test to fail the day one did. It failed here:
the real folder holds deaths in **16 of its 28 sessions**, and the archive folded **23**. The gap
was never in the data, it was in which files were sampled. One of those sessions is a sample now
(`20260824-online-deaths.log.txt`), the guard test is replaced by the real one, and
`Event::Died` is covered by data the game wrote.

---

## What was built

| Crate | What landed | Tests |
|---|---|---|
| `run` | `SourceKey` / `Resume` / `resume` / `fingerprint` (FNV-1a by hand — `DefaultHasher` is not stable across releases and this number is stored); `Tail::pending`; serde on `Event`, `SeedKind`, `Run`, `Floor`, `Outcome` | +9 source, +4 serde, +1 tail |
| `discovery` | `GameDataFolder`, `Discovery::game_data`, probed independently of the saves | +6 |
| `store` | Migration 4: `sources`, `events`, `runs`. A session is keyed by its folder name, a launch by nothing — `NULL`, so two launches are two rows and the older keeps its events | +11 |
| `log-watch` | New crate: positional reads, `sessions()`, `Ingest` (backfill and live are one function), `watch()` on the folder with a 2 s debounce | +17 |
| `ipc` | `RunsView` and everything under it, `CatalogKinds`, seven types in the generated contract | +7 shape, +2 real |
| `app` | `ArchiveState`, the `runs` command, `runs-changed`, the thread that backfills and then watches | none — the Tauri crate is wiring |
| `ui` | `fetchRuns` wrapper, the fourth event name, a Runs section on `#verify` | covered by `typecheck` / `scan` |

Two decisions inside the code worth naming. **Item lookup goes through the three collectible
kinds and never trinkets**: the catalog is keyed by `(kind, id)`, a trinket can carry a
collectible's number, and the line the fold reads says `Adding collectible`. **Without a catalog
everything is passive**: item kinds change which active a run is carrying and never change an
outcome, so the archive is built on a machine without the game rather than waiting for one.

---

## What is not done

- **Neither screen.** `Live` and `Runs` stay placeholders, which is the spec's own boundary: a
  layout written now would get a vote on a contract that has not been used yet.
- **The agreement with the game's counters**, above. An `Unknown`, with what would close it.
- **`Won` is the outcome with the thinnest real-data coverage now**, which is the opposite of
  what 1a expected: one sample log ends in a win, against two deaths in another and twenty open
  or abandoned runs across the sessions.
- **Identity is still a heuristic**, and the plan says so rather than hiding it: a 64-byte anchor
  inside run content. Its failure mode is re-reading a log, never merging two.
- **Not seen in a real Tauri window.** The archive was verified against this machine's real
  folders outside the app; the watcher firing on a game actually being played has not been
  watched.
