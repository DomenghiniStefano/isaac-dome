# app-shell and ipc — Tauri scaffold and the boundary with the UI (design)

**Date:** 2026-09-02
**Sub-project:** M1 · fourth piece
**Status:** pending spec review

## Context

Fourth sub-project of M1, taken up in place of `catalog` — which stays blocked until the
game's XMLs are available (see `docs/STATO.md`, *Open blockers*).

The three completed modules (`core-save`, `discovery`, `unpack`) are libraries with no
consumers: today nothing calls them except the tests. This sub-project builds the first
real consumer — the application — and with it the **boundary between Rust and the
interface**, which is the architectural decision this piece exists to make.

It doesn't build finished screens: the design system is being worked on outside the
repo. It builds the end-to-end chain and a deliberately rough verification screen that
demonstrates that real data reaches all the way to the browser.

## Load-bearing constraint: the frontend doesn't know about the disk

`CLAUDE.md` states it twice, and it's the constraint that shapes the whole module:

> Rust does everything that touches the disk. Vue only receives already-resolved JSON:
> the frontend knows nothing about offsets, file names, or log strings.

Operational consequences, all verifiable by reading the JSON that comes out of the
commands:

1. **No file path as an identifier.** Candidates travel with an opaque id. The path
   stays a supporting field, showable in diagnostics ("found here"), never the key the
   UI uses to talk about the profile.
2. **No offset, no raw byte.** `Section.bytes` (~14 KB per save) never crosses the IPC.
   What goes out are counts, labels, and already-built matrices.
3. **No path parameter in the commands.** The active profile is a backend *state*, not
   an argument the UI carries on every call.
4. **Diagnostics come out already in readable form.** If a section can't be read, the
   JSON says what's missing, not a code the UI would have to know how to interpret.

## Load-bearing constraint: don't choose on the user's behalf

On a real machine **six saves** coexist — three slots for the `rep_` edition and three
for `rep+` — and every number the app shows depends on which one is being read. Explicit
requirement from the client:

> If I have multiple saves and multiple versions on my PC, I need to select the one I
> want, not have it pick one at random.

This forbids silently falling back in any form, including the most tempting one: "open
the most recent". The most recent is **suggested**; it's never chosen behind the user's
back. The case that makes the rule non-negotiable is the third one below: a saved choice
that no longer exists. Falling back there means showing another profile's numbers
without saying so — an error the user never reports, because the app appears to work.

## Crates and boundaries

```
crates/ipc/      NEW  pure logic: from Discovery/Save to view-model. No dependency
                       on Tauri, no I/O. Testable with `cargo test`.
crates/app/      NEW  Tauri binary: command wiring, settings I/O.
ui/              NEW  Vue 3 + TypeScript frontend (pnpm, Vite).
```

Rust stays entirely under `crates/*`: the workspace's `members = ["crates/*"]` glob
keeps working unchanged. `ui/` sits outside because it isn't a crate.

**`crates/app` is a single crate**, with `lib.rs` and a `[[bin]] main.rs` together. It
must not be split into two crates: separating the entrypoint from the Tauri library is
a documented cause of CLI failures (`a bin target must be available for cargo run`).
`tauri dev` also watches the workspace crates the app depends on, so a change to
`core-save` rebuilds the application with no extra configuration.

The split of logic follows a single rule: **if it has a return value worth verifying, it
lives in `ipc`.** What's left in `app` is what can't usefully be tested — command
registration, opening the window, reading and writing a settings file.

## Modules of `ipc`

| Module | Responsibility |
|---|---|
| `profile` | From `Vec<SaveCandidate>` to presentable candidates; opaque id derivation; active-profile resolution rule. |
| `marks` | Index tables → (boss, character) carried over from the Python reference; construction of the marks matrix. |
| `summary` | Per-section counts and labels, read from the file. |
| `settings` | Type of the persisted settings and its serialization. No I/O. |

## Model and public API

All types derive `Serialize` with `#[serde(rename_all = "camelCase")]`; data-carrying
enums use `#[serde(tag = "kind")]`, so the frontend can `switch` on a string instead of
guessing from the shape.

**Verified trap:** `rename_all = "camelCase"` on an enum renames the **variant names**,
not the fields inside struct variants. Without `rename_all_fields = "camelCase"` on top
of it, `ActiveProfile::Active { auto_selected }` comes out as `auto_selected`, and the
TypeScript, expecting `autoSelected`, reads `undefined` with no error — on exactly the
flag that distinguishes "the app chose" from "the user chose". Every enum in this module
with struct variants carries both attributes, and the shape of the JSON is pinned by a
test.

### Installation state

```rust
pub struct SetupState {
    pub steam: Option<SteamView>,
    pub game: Option<GameView>,
    pub candidates: Vec<CandidateView>,
    pub active: ActiveProfile,
    pub diagnostics: Vec<SetupDiagnostic>,
}

pub struct CandidateView {
    pub id: ProfileId,            // opaque, stable across runs
    pub prefix: SavePrefix,       // rep_ or rep+, from the filename
    pub slot: u8,                 // 1..3
    pub source: CandidateSource,  // Steam userdata · Documents · manual choice
    pub modified_unix: Option<u64>, // seconds since the epoch, or absent
    pub size_bytes: u64,
    pub suggested: bool,          // the most recent one: a suggestion, not a choice
    pub path_hint: String,        // supporting detail, never the key; redacted (see below)
}

/// Opaque key matching a persisted choice to a candidate.
/// Newtype over `String` to prevent a path from ending up in it by mistake.
pub struct ProfileId(String);
```

**`CandidateSource` is not `SaveSource`.** `discovery`'s type carries data that must not
cross the IPC: `SteamCloud { account_id }` exposes the Steam account id, `Documents
{ folder }` a path. The view-model keeps only the provenance:

```rust
#[serde(tag = "kind")]
pub enum CandidateSource { SteamCloud, Documents, Manual }
```

**`path_hint` must be redacted of the Steam account id.** With Steam Cloud, a save's
path is `…\userdata\<account_id>\250900\remote\rep+persistentgamedata1.dat`: the
account id **is a folder segment**. Allowing a "view-only" path while also forbidding
the account id would be a contradiction — the first contains the second.

The field therefore keeps the full path with **only the id segment replaced**:
`SaveSource::SteamCloud { account_id }` already carries it, so the substitution is exact
rather than heuristic. The path isn't truncated to its last components: `path_hint`
exists to tell the user *"found here"*, and truncating it would defeat exactly that use
in the manual-fallback case.

A test that verifies the id's absence must use a **realistic** path, including the
`userdata/<id>/` segment. A convenience path that doesn't have that segment verifies the
absence of something it never had: it gives a false guarantee.

**Casing of serialized enums.** Enums **defined by `ipc`** use
`rename_all = "camelCase"` (`"steamCloud"`, `"needsChoice"`, `"known"`). Enums that come
from the domain crates — `SavePrefix`, `Edition`, `Dlc`, `Kind` — keep the `snake_case`
they're already serialized with (`"rep_plus"`, `"repentance_plus"`, `"achievements"`),
and the TypeScript types mirror that. Wrapping them in as many view-models just to unify
the casing would be duplication with no functional gain; the rule only holds, though, if
it's written down and verified, so the serialized value of these enums must be **pinned
by a test**.

**The date comes out as seconds since the epoch, not as a string.** Formatting a date is
a presentation decision and depends on the language: the frontend renders it with
`Intl`, which already knows how to do that for Italian and English. Returning an
already-formatted string would mean deciding the format in Rust and then being unable to
change it per language — and adding a date dependency that serves nothing else.

**`prefix` is not `edition`.** They're two different pieces of provenance and must be
kept separate: `discovery` derives `Edition` (Rebirth → Repentance+) from the DLCs
declared in the game's `appmanifest`, while only `SavePrefix` (`rep_` or `rep+`) can be
derived from the save's filename. A save file doesn't declare which edition is
installed. Edition therefore lives in `GameView`, once; the candidate carries the
prefix, which is what actually distinguishes the six files from each other.

### Active profile

```rust
#[serde(tag = "kind")]
pub enum ActiveProfile {
    /// No candidate: the chain broke earlier.
    None { reason: MissingReason },
    /// Multiple candidates and no valid choice: the user decides.
    NeedsChoice { reason: ChoiceReason, suggested: Option<ProfileId> },
    /// A choice was made, or there is a single candidate.
    Active { profile: CandidateView, auto_selected: bool },
}

#[serde(tag = "kind")]
pub enum ChoiceReason {
    /// First run, or a choice never made.
    NeverChosen,
    /// The saved choice no longer matches any candidate.
    SavedProfileGone { was: String },
}
```

`auto_selected` is what lets the interface say *"only profile found"* instead of making
the user believe they chose it themselves. The distinction is declared by the data, not
reconstructed by the UI.

### Marks matrix

```rust
pub struct MarksMatrix {
    pub characters: Vec<CharacterRow>,   // 34
    pub bosses: Vec<String>,             // 10 verified
    pub totals: MarksTotals,
}

pub struct CharacterRow {
    pub character: String,
    pub group: CharacterGroup,           // Original · Forgotten · Later
    pub cells: Vec<Cell>,                // one per boss, in the order of `bosses`
}

#[serde(tag = "kind")]
pub enum Cell {
    /// Bitmask read from the file: bit 0 and 1 = mark levels,
    /// bit 2 = third level of meaning, unconfirmed. Values 0..=7.
    Known { bits: u8 },
    /// Index not located in the tables, or outside the section read.
    Unknown,
    /// Value outside 0..=7: not a mark bitmask, so the table that produced
    /// this index is probably wrong.
    Unexpected { value: u32 },
}

pub struct MarksTotals {
    pub cells: usize,       // 340
    pub readable: usize,    // 321
    pub unknown: usize,     //  19
    pub unexpected: usize,  //   0 on healthy data
    pub started: usize,     // readable cells with bits != 0
}
```

**Why `Unexpected` exists.** The mark cells only ever take the values 0, 1, 2, 3, 5, or
7: that's the observation the entire interpretation rests on. A value outside that range
isn't a strange mark, it's the symptom that the index points somewhere else — index 385,
for instance, is 49. Truncating to `u8` or masking the low three bits would hide the
error by producing a plausible cell; `Unexpected` makes it visible and turns a table
transcription mistake into a fact the tests can assert. On healthy saves, the count is
zero.

**The `Cell` type is the heart of the module.** `CLAUDE.md` forbids computing completion
percentages while the third bit remains unknown, and the design brief forbids confusing
"never done" with "unreadable". Modeling the cell as a sum makes both errors
unexpressible: there is no way to write `Unknown` that evaluates to zero.

`MarksTotals` exists for the same reason: the honest denominator is `readable`, not
`cells`. No percentage field is exposed.

### Save summary

```rust
pub struct SaveSummary {
    pub profile: ProfileId,            // which profile we're talking about
    pub sections: Vec<SectionCount>,   // kind + count read from the file
    pub diagnostics: Vec<SaveDiagnostic>,
}
```

**Diagnostics are enums, not `Debug` strings.** The `format!("{x:?}")` shortcut on
someone else's type prints *all* of its fields, including ones this boundary forbids:
`discovery::Diagnostic::UnreadablePath` carries a `PathBuf` that, under `userdata\`,
contains the Steam account id and, in any case, the Windows username, and
`core_save::Diagnostic` carries byte offsets and lengths. Both would end up in the JSON
of the app's very first command.

`SetupDiagnostic` and `SaveDiagnostic` are therefore defined here and carry only what's
needed: for paths, **only the last component** (`remote`, `appmanifest_250900.acf`),
which says what wasn't read without any identifying segment; for save diagnostics, the
meaning without the positions. Side benefit: a tagged enum is translatable on the
frontend, a Rust-style `Debug` string isn't — and the same rule applies here as for why
`SectionCount` carries the `Kind` rather than an already-translated label.

`SectionCount` carries the serialized `Kind`, **not an already-translated label**: a
section's readable name is an interface string, so it lives in the i18n files with all
the others. Returning it from Rust would mean having two places where it's translated.

No expected count is hardcoded: `count` is whatever the file declares. A save from
January 2025 declares 521 counters where a recent one declares 523, and the interface
must be able to show both without treating either as wrong.

## Active profile resolution rule (pure function)

```rust
pub fn resolve_active(saved: Option<&ProfileId>, candidates: &[CandidateView]) -> ActiveProfile
```

| Situation | Outcome |
|---|---|
| No candidate | `None { reason }` — the chain broke at Steam, the game, or the saves |
| One candidate, no saved choice | `Active { auto_selected: true }` — nothing to choose, but it's declared |
| Multiple candidates, no saved choice | `NeedsChoice { NeverChosen, suggested: the most recent one }` |
| A saved choice is present among the candidates | `Active { auto_selected: false }` |
| A saved choice is absent from the candidates | `NeedsChoice { SavedProfileGone { was }, suggested: the most recent one }` |

The last row is the rule the module exists to guarantee. There is no branch that returns
`Active` without either the user having chosen or the candidate being unique.

**The suggestion** is the candidate with the most recent `modified`; if tied, or if
dates are absent, the order is deterministic (edition, then slot), because a suggestion
that changes on every launch is worse than no suggestion at all.

## Deriving the opaque id

```rust
pub fn profile_id(path: &Path) -> ProfileId
```

A pure, deterministic function of the normalized absolute path: same path, same id,
across runs. Needed because the persisted choice must be re-matchable to the candidate
on the next restart, without the id being the path.

It's not a cryptographic requirement: it's a matching key. It does, however, need to be
**stable across casing and separators**, because on Windows the same file arrives
written in different ways depending on how it was discovered (registry,
`libraryfolders.vdf`, manual choice).

## The marks tables, carried over from the Python reference

`reference/isaac_counters.py` contains the tables that translate a section-2 index into
a *(boss, character)* pair: `BLOCKS_14` (10 bosses × 14 original characters),
`FORGOTTEN` (10 single cells) and `BLOCKS_19` (9 bosses × 19 characters, **derived and
undocumented**).

**It's not a translation, it's a revision.** The reference builds the matrix with
`m[boss].get(character, 0)`: cells it doesn't know become zeros indistinguishable from
"never done". Verified by running it against the real sample — the Delirium column
reports `0` for all 19 characters of the third group, and the total it prints
(`93 out of 340`) uses a denominator that includes 19 unreadable cells.

The Rust version must produce `Unknown` exactly there, and a total out of 321.

### Indices outside the file's range

Section 2 doesn't have a fixed length: the January 2025 sample declares **521** values,
recent ones **523**, and nothing guarantees a future patch won't remove some. Building
the matrix therefore reads by index **with range checking**: an index the file doesn't
contain produces `Unknown`, never a panic and never a value read from somewhere else.

It's the same rule as the known gaps, applied to a different cause — there it's the
table that's missing, here it's the data — and it produces the same outcome, which is
the point: the user sees "we don't know", not a made-up zero. On the two samples present
this case doesn't occur (the highest index used is 384), but the rule holds regardless,
because it's exactly the kind of change a patch introduces without warning.

### Known gaps, to be represented rather than hidden

- `BLOCKS_19` stops at Hush: the **Delirium** column isn't localized for those 19
  characters → 19 `Unknown` cells.
- **Mother** and **The Beast** have no column for anyone → they don't appear among the
  `bosses`.
- Index 385 and the 404–522 tail remain unattributed → they don't enter the matrix.

## Tauri commands

```rust
#[tauri::command] fn setup_state(...) -> Result<SetupState, IpcError>
#[tauri::command] fn select_profile(id: ProfileId, ...) -> Result<SetupState, IpcError>
#[tauri::command] fn save_summary(...) -> Result<SaveSummary, IpcError>
#[tauri::command] fn completion(...) -> Result<MarksMatrix, IpcError>
```

None of them take a path: `save_summary` and `completion` read the active profile from
the state. If there is no active profile, they return `IpcError::NoActiveProfile`, which
the UI treats as "go to selection", not as an error to display.

`IpcError` is a tagged, serialized enum, not a string: the UI must be able to
distinguish "no active profile" from "unreadable file" without parsing text. A Tauri
command's error type must implement `Serialize`, so `IpcError` has an explicit `impl`.

**Errors are the exception.** In line with *"degrade, never fail"*, expected cases —
Steam absent, game not installed, unreadable section — are not errors: they travel
inside the payload, in `diagnostics` and in the state enums. `Err` is reserved for what
makes the command unable to respond at all.

## Persisting the choice

A `settings.json` file in the application's config folder, resolved with Tauri's APIs:

```json
{ "activeProfileId": "…" }
```

Reading and writing live in `crates/app`; the type and its serialization live in `ipc`.
An absent, unreadable, or malformed file is equivalent to "no saved choice" and produces
a diagnostic — never a fatal error, never a silent rewrite of the user's file.

## Frontend

`ui/`, created with the `vue-ts` Vite template. Dependencies for this step:
`@tauri-apps/api` (from which `invoke` is imported as `@tauri-apps/api/core`, no longer
from `/tauri` as in Tauri 1) and Tailwind v4 with the `@tailwindcss/vite` plugin.

**Deliberately out of this step:** shadcn-vue, Reka UI, Pinia, vue-i18n, TanStack Table
and Virtual. They arrive with the design system, and introducing them earlier would mean
guessing the token structure only to redo it later. Tailwind goes in now because its
`@theme` is where those tokens will live, and it's less work to set it up now than to
add it afterward.

The code in `ui/` follows `docs/frontend-conventions.md` from the very first commit —
in particular: no `<style>` block in SFCs, no hardcoded visual value, and no `invoke()`
outside `src/lib/ipc/`. The verification screen is no exception: it's the frontend's
first code, and therefore the place where the conventions either take root or get
abandoned forever.

The tooling that enforces them goes in at the same step, not later:

- **Prettier** with `prettier-plugin-tailwindcss`, and the `tailwindStylesheet` option
  pointing to the CSS with `@theme` (in Tailwind v4 there's no longer a JavaScript
  config file to read).
- **ESLint** flat config: `eslint-plugin-vue`, `typescript-eslint`,
  `@vue/eslint-config-typescript`.
- **`vue-tsc --noEmit`** for type-checking.
- **`ui/scripts/scan-conventions.mjs`**, a committed script runnable with `pnpm scan`,
  for the three rules none of these tools cover: `<style>` blocks, hardcoded visual
  values, `invoke()` outside the IPC layer. Verified that `eslint-plugin-vue` offers no
  official rule for the first two.

The single screen is a **verification screen**: it prints the installation state, the
candidates with their fields, the section counts, and the marks matrix in raw table
form, with `Unknown` cells visibly distinct from zeros. It has no aesthetic ambitions and
isn't meant to survive: it exists to prove the chain holds up and to serve as a test bed
for the design system when it arrives.

**Prerequisite:** Node 20.19+ or 22.12+. Node 18 is below the minimum for Vite 7 and the
Tailwind v4 toolchain.

## Tests

### Pure logic in `ipc` (always run)

- `resolve_active`: one case per row of the table, including the two that matter most —
  a single auto-selected candidate declared as such, and a vanished saved choice that
  produces `NeedsChoice` and **not** a fallback.
- Suggestion: the most recent one wins; with tied dates the order is stable across two
  runs.
- `profile_id`: deterministic; invariant to casing and separators; different paths give
  different ids.
- `settings`: round-trip; a malformed file is equivalent to "no choice".
- Matrix: the 19 cells of the Delirium column for the third group are `Unknown`; the
  totals report 340 cells, 321 readable, 19 unknown.
- Matrix on a section shorter than expected: given a truncated section, cells whose
  index falls outside become `Unknown` and the totals update accordingly. No panic, no
  value read from an adjacent index.

### Cross-check against the Python reference (skipped if absent)

The reference is the only independent source to verify the carried-over tables against:
a transcription error would produce a plausible and wrong matrix, which no test written
from the same code would catch.

The comparison happens **only on the 321 cells the reference actually knows**, because
on the other 19 the reference is wrong by construction. On the January 2025 sample the
expected value is **93 cells started out of 321**.

The reference is invoked as `python` (not `python3`, absent on this machine), and the
test skips with a note if Python or the sample is missing.

### On real saves (skipped if absent)

- `save_summary` on the two samples present in `samples/`: the counts match what the
  files declare, which differ from each other. Also covers the `rep_` edition, never
  exercised until now.

### Not tested

The command wiring in `crates/app` and the verification screen. They're connective code
with no return value worth verifying; their test is that the application starts and
shows the data.

## Out of scope

- `catalog` — item and achievement names. Blocked, and nothing here assumes it.
- `graph` — "unlockable now", fan-out, plans.
- Sprite extraction (`unpack` is ready, but no screen uses it yet).
- `store` / SQLite: the settings are a JSON file, not a database.
- Design system, i18n, final dark theme, shadcn-vue components.
- `log-watch` and everything about runs.
- Installer, updater, signing.

## Completion criteria

1. `cargo test -p ipc` green, with `resolve_active`'s tests covering all five rows of the
   resolution table.
2. The cross-check against Python passes on the 321 comparable cells, and the remaining
   19 come out as `Unknown`.
3. `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` clean on the new
   crates.
4. The application starts and the verification screen shows, with real data: the
   discovery chain, this machine's six candidates, the `NeedsChoice` state, and — after a
   selection — the section counts and the matrix with the unknown cells distinguishable.
5. The selection survives an application restart.
6. In the JSON crossing the IPC there is no **offset, raw section byte, Steam account
   id**, nor any path used as an identifier: profiles are addressed only by `ProfileId`.
   The only allowed path is `pathHint`, view-only, which no command accepts as input.
7. `pnpm lint`, `pnpm format:check` and `vue-tsc --noEmit` clean, and
   `node scripts/scan-conventions.mjs` with no violations.
