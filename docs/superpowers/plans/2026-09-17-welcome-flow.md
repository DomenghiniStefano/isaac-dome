# 3.8 — Welcome flow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** On launch, with no profile settled, the app asks which save you are playing with and
shows what each one holds — and, when it found nothing, says which link of the chain broke and
lets you point at a folder by hand.

**Architecture:** A full-screen takeover drawn **above the router** (no route, no tab, nothing in
the session document), fed by `setup_state`, which now carries a per-candidate preview computed in
`ipc` from the `.dat` alone. The saves reach `ipc` through a reader closure, so the crate stays
pure. B14's folder picking is a Rust-side dialog: the path never enters JavaScript.

**Tech Stack:** Rust (`ipc`, `app`, `core-save`, `discovery`), `tauri-plugin-dialog` 2, Vue 3 +
TypeScript, Pinia, Vitest.

**Spec:** `docs/superpowers/specs/2026-09-17-welcome-flow-design.md`

## Global Constraints

Copied from `CLAUDE.md` and the spec; every task inherits them.

- **Read-only on saves.** Nothing here opens a `.dat` for writing, ever.
- **No path, offset or raw byte crosses the IPC.** `pathHint` is redacted and display-only.
- **Every struct crossing the IPC** carries `#[serde(rename_all = "camelCase")]`; every enum with
  data-carrying variants adds `tag = "kind"` **and** `rename_all_fields = "camelCase"`; a fieldless
  enum stays a bare camelCase string.
- **New types crossing the IPC must be declared in `crates/ipc/src/contract.rs`**, or
  `pnpm ipc:types` will not generate them and `scripts/check` fails on the regenerated file.
- **Exhaustiveness:** no `_ =>` arm on a closed enum, Rust or TypeScript (`assertNever`).
- **Never `panic!`/`unwrap()`** outside tests on data read from disk.
- **Frontend:** no `<style>` in SFCs; no hardcoded visual constants (every value a token in
  `@theme`); no `invoke()` outside `ui/src/lib/ipc/`; no raw `<button>`/`<input>`; no string
  unions — `const X = {…} as const`, the only exception being a tagged union's own `kind`.
- **Test-first**, and the expected value comes from this plan, never from the code's output. A
  failing test is first a hypothesis of a bug in the code.
- **Real-data tests declare or skip** through `test-support`; never `env!("CARGO_MANIFEST_DIR")`.
- Commits: `type(scope): subject`, English, atomic. **No `Co-Authored-By`, no AI trailer, ever.**
- Branch: `feature/welcome-flow`, already cut from `develop`. Nothing lands on `develop` directly.

### Two decisions this plan takes that the spec left open

Both are corrections the reading made, and both belong in the report.

1. **`CandidatePreview` carries no `diagnostics`.** §4 of the spec listed
   `Vec<SaveDiagnostic>`. A `SectionOverrun { section: 7 }` says nothing on a welcome card, the
   three counts already say which part could not be read, and `SaveSummary` still carries the
   diagnostics for the Profile screen where they are read. Dropping it also keeps `CandidatePreview`
   free of a type declared later in `contract.rs`.
2. **The chosen folders never enter `ipc::Settings`.** That struct **crosses the IPC** — the
   `settings` command returns it — and a `PathBuf` in it would break the boundary's first rule.
   The app persists them in a wrapper it owns, `settings_file::Stored`, with
   `#[serde(flatten)]` so the file's shape does not change.

---

### Task 1: `preview_of` — the three counts, in `ipc`, pure

**Files:**
- Create: `crates/ipc/src/preview.rs`
- Modify: `crates/ipc/src/marks.rs` (extract `marks_totals`), `crates/ipc/src/lib.rs` (exports)
- Test: `crates/ipc/tests/preview.rs`

**Interfaces:**
- Consumes: `core_save::{Kind, Save, Section}`, `crate::marks::{BOSSES, CHARACTERS, MarksTotals}`.
- Produces:
  - `pub fn marks_totals(counters: &[u32]) -> MarksTotals`
  - `pub enum PreviewCount { Read { done: u32, of: u32 }, Unread }`
  - `pub struct CandidatePreview { achievements, items, marks: PreviewCount, unreadable_cells: u32 }`
  - `pub fn preview_of(save: &Save) -> CandidatePreview`

- [ ] **Step 1: Write the failing test**

Create `crates/ipc/tests/preview.rs`:

```rust
//! The welcome's three counts. Slot 0 is neither an achievement nor an item — the same
//! `.skip(1)` the Unlock and Collection views already count by — and a section that was not
//! read reports `Unread`, never `Read { done: 0 }`: a zero is a profile at the start.

use core_save::{Kind, Save, Section};
use ipc::{preview_of, CandidatePreview, PreviewCount};
use serde_json::json;

fn section(kind: Kind, bytes: Vec<u8>) -> Section {
    let per = kind.bytes_per_entry().unwrap_or(1);
    Section {
        kind,
        count: (bytes.len() / per) as u32,
        f2: 0,
        offset: 0,
        bytes,
    }
}

fn save(sections: Vec<Section>) -> Save {
    Save {
        unknown_0x10: 0,
        sections,
        diagnostics: Vec::new(),
    }
}

/// Four flags: slot 0 (which is no achievement) plus three achievements, two of them done.
fn three_achievements() -> Section {
    section(Kind::Achievements, vec![1, 1, 0, 1])
}

#[test]
fn slot_zero_is_in_neither_the_numerator_nor_the_denominator() {
    let p = preview_of(&save(vec![three_achievements()]));
    assert_eq!(p.achievements, PreviewCount::Read { done: 2, of: 3 });
}

#[test]
fn a_section_that_was_not_read_says_so_rather_than_counting_zero() {
    let p = preview_of(&save(vec![three_achievements()]));
    assert_eq!(p.items, PreviewCount::Unread, "no section 4 in this save");
    assert_eq!(p.marks, PreviewCount::Unread, "no section 2 in this save");
    assert_eq!(
        p.unreadable_cells, 0,
        "no counters section is not 408 unreadable cells: it is nothing to say"
    );
}

#[test]
fn an_empty_counters_section_is_read_and_every_cell_is_unreadable() {
    let p = preview_of(&save(vec![section(Kind::Counters, Vec::new())]));
    assert_eq!(p.marks, PreviewCount::Read { done: 0, of: 0 });
    assert_eq!(p.unreadable_cells, 34 * 12);
}

/// The wire shape. Silent in TypeScript when it is wrong, so it is pinned here.
#[test]
fn the_json_is_camel_case_and_the_count_is_tagged() {
    let p = CandidatePreview {
        achievements: PreviewCount::Read { done: 379, of: 640 },
        items: PreviewCount::Unread,
        marks: PreviewCount::Read { done: 92, of: 120 },
        unreadable_cells: 8,
    };
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        json!({
            "achievements": { "kind": "read", "done": 379, "of": 640 },
            "items": { "kind": "unread" },
            "marks": { "kind": "read", "done": 92, "of": 120 },
            "unreadableCells": 8
        })
    );
}
```

- [ ] **Step 2: Run it and watch it fail**

Run: `cargo test -p ipc --test preview`
Expected: FAIL — `unresolved import ipc::preview_of`.

- [ ] **Step 3: Extract `marks_totals` so there is one definition of "a mark is taken"**

In `crates/ipc/src/marks.rs`, replace `fn totals_of(rows: &[CharacterRow]) -> MarksTotals` with a
pair, and add the public entry point. The counting rules move unchanged — do not retype them:

```rust
/// The totals alone, without the matrix. The welcome's preview needs the numbers and none of
/// the art, and a second definition of "a mark is taken" would drift from this one.
pub fn marks_totals(counters: &[u32]) -> MarksTotals {
    let cells: Vec<Cell> = (0..CHARACTERS.len())
        .flat_map(|c| (0..BOSSES.len()).map(move |b| cell_at(counters, c, b)))
        .collect();
    totals_from(&cells)
}

fn totals_of(rows: &[CharacterRow]) -> MarksTotals {
    let cells: Vec<Cell> = rows.iter().flat_map(|r| r.cells.iter().copied()).collect();
    totals_from(&cells)
}

fn totals_from(cells: &[Cell]) -> MarksTotals {
    let count = |f: fn(&Cell) -> bool| cells.iter().filter(|&c| f(c)).count();
    MarksTotals {
        cells: cells.len(),
        readable: count(|c| matches!(c, Cell::Known { .. })),
        unknown: count(|c| matches!(c, Cell::Unknown)),
        unexpected: count(|c| matches!(c, Cell::Unexpected { .. })),
        // Bit 0 or bit 1: the unconfirmed bit alone draws nothing in the grid (the
        // frontend's `markVisual`), and a total that counts what its grid doesn't show lies.
        normal: count(|c| matches!(c, Cell::Known { bits } if *bits & 3 != 0)),
        // Bit 1 alone decides hard, and a bare 2 is hard: the value replaces the one before
        // it rather than accumulating, so requiring bit 0 as well would drop the cells where
        // the second level overwrote the first (B58, 2026-09-17).
        hard: count(|c| matches!(c, Cell::Known { bits } if *bits & 2 != 0)),
    }
}
```

`Cell` derives `Clone` but not `Copy`; if `.copied()` does not compile, add `Copy` to `Cell`'s
derive list — it is a two-field, fieldless-or-integer enum — or use `.cloned()`.

- [ ] **Step 4: Write `preview.rs`**

```rust
//! What a candidate's save says about itself, read from the `.dat` alone.
//!
//! No `catalog`: counting needs the file's own layout, which lives in `core-save`; naming a
//! character needs the game's archives and is a different question. So this answers on a
//! machine with no game installed, which is where the welcome matters most.

use core_save::{Kind, Save};
use serde::Serialize;

use crate::marks::marks_totals;

/// A count the file may not let us make. **Never `Read { done: 0 }` for a section that was
/// not read**: a zero is a profile at the start, and the two are not one sentence. Same
/// choice as `Verdict::Partial` in `graph` and `Generated::NotSaid` in `run`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum PreviewCount {
    Read { done: u32, of: u32 },
    Unread,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct CandidatePreview {
    pub achievements: PreviewCount,
    pub items: PreviewCount,
    /// Cells that reached a level, out of the cells the file lets us read — the denominator
    /// Completion already uses, where what cannot be read stays outside it.
    pub marks: PreviewCount,
    /// Cells the file does not let us read. Zero when the matrix is whole, and zero as well
    /// when there is no counters section at all: that absence is `marks: Unread`, not 408
    /// unreadable cells.
    pub unreadable_cells: u32,
}

pub fn preview_of(save: &Save) -> CandidatePreview {
    let totals = save.u32s(Kind::Counters).map(|c| marks_totals(&c));
    CandidatePreview {
        achievements: slots(save.flags(Kind::Achievements).as_deref()),
        items: slots(save.flags(Kind::Items).as_deref()),
        marks: match totals {
            Some(t) => PreviewCount::Read {
                done: t.normal as u32,
                of: t.readable as u32,
            },
            None => PreviewCount::Unread,
        },
        unreadable_cells: totals.map_or(0, |t| (t.unknown + t.unexpected) as u32),
    }
}

/// Slot 0 is neither an achievement nor an item — `unlock_view` skips it and
/// `collection_view` says so out loud — so it is out of the numerator *and* the denominator.
/// This is the first place a **user** sees an achievements total: `UnlockTotals.slots` counts
/// the file's slots, 642, and is shown only on the development-only verification page.
fn slots(flags: Option<&[bool]>) -> PreviewCount {
    match flags {
        None => PreviewCount::Unread,
        Some(f) => PreviewCount::Read {
            done: f.iter().skip(1).filter(|&&b| b).count() as u32,
            of: f.len().saturating_sub(1) as u32,
        },
    }
}
```

In `crates/ipc/src/lib.rs`, beside the other module declarations and `pub use` lines:

```rust
mod preview;
pub use preview::{preview_of, CandidatePreview, PreviewCount};
```

and in `crates/ipc/src/marks.rs`'s export line in `lib.rs`, add `marks_totals` to what `marks`
already publishes.

- [ ] **Step 5: Run the test**

Run: `cargo test -p ipc --test preview`
Expected: PASS, 4 tests.

- [ ] **Step 6: Check nothing else moved**

Run: `cargo test -p ipc`
Expected: PASS. `marks_matrix`'s own tests must be untouched — they are what says the extraction
changed no count.

- [ ] **Step 7: Commit**

```bash
git add crates/ipc/src/preview.rs crates/ipc/src/marks.rs crates/ipc/src/lib.rs crates/ipc/tests/preview.rs
git commit -m "feat(ipc): a candidate's save says what it holds, in three counts"
```

---

### Task 2: the preview on real saves — the properties

**Files:**
- Test: `crates/ipc/tests/preview_real.rs`

**Interfaces:**
- Consumes: `ipc::{preview_of, PreviewCount}`, `test_support::dated_series`, `core_save::Save`.
- Produces: nothing. This task is the evidence for Task 1.

This machine holds 16 saves: a `rep_` series of 15 across Jan–Jun 2024 and one `rep+` of
2025-01-12. **Properties, never pinned values** — a pinned count is a fixture of an era, and the
`of` a file declares is exactly what a patch moves.

- [ ] **Step 1: Write the failing test**

Create `crates/ipc/tests/preview_real.rs`:

```rust
//! The preview, against the real series. Properties rather than values: the denominator is
//! read from the file, so it is the first thing a patch moves — a 2024 save declares 638
//! achievements and a 2025 one 641 (`docs/STATUS.md`), which is `of` 637 and 640 here.
//!
//! Each series is walked **on its own**. The two editions are two profiles, and a jump
//! between them would read as progress — the lesson `marks_real.rs` carries since B58.

use core_save::Save;
use ipc::{preview_of, CandidatePreview, PreviewCount};
use test_support::dated_series;

const SERIES: [&str; 2] = ["rep_persistentgamedata1.dat", "rep+persistentgamedata1.dat"];

fn previews(suffix: &str) -> Vec<CandidatePreview> {
    dated_series(suffix)
        .iter()
        .filter_map(|p| Save::open(p).ok())
        .map(|s| preview_of(&s))
        .collect()
}

fn read(count: PreviewCount) -> Option<(u32, u32)> {
    match count {
        PreviewCount::Read { done, of } => Some((done, of)),
        PreviewCount::Unread => None,
    }
}

#[test]
fn no_count_ever_exceeds_what_the_file_declares() {
    let mut seen = 0;
    for suffix in SERIES {
        for p in previews(suffix) {
            for count in [p.achievements, p.items, p.marks] {
                if let Some((done, of)) = read(count) {
                    assert!(done <= of, "{done} of {of}");
                    seen += 1;
                }
            }
        }
    }
    if seen == 0 {
        test_support::skip("no dated series in samples/: nothing to hold the property over");
    }
}

#[test]
fn a_series_never_regresses_and_at_least_one_window_moves() {
    for suffix in SERIES {
        let dones: Vec<u32> = previews(suffix)
            .iter()
            .filter_map(|p| read(p.achievements))
            .map(|(done, _)| done)
            .collect();
        if dones.len() < 2 {
            test_support::skip(&format!("{suffix}: fewer than two snapshots, no window to walk"));
            continue;
        }
        for pair in dones.windows(2) {
            assert!(pair[1] >= pair[0], "{} then {}", pair[0], pair[1]);
        }
        // The vacuity guard: "never regresses" holds trivially over a flat series, and a
        // test that cannot fail reports coverage that is not there.
        assert!(
            dones.last() > dones.first(),
            "{suffix}: the series never moves, so the property held over nothing"
        );
    }
}

#[test]
fn the_two_editions_declare_different_totals_and_the_later_one_declares_more() {
    let declared = |suffix: &str| -> Option<u32> {
        let all: Vec<u32> = previews(suffix)
            .iter()
            .filter_map(|p| read(p.achievements))
            .map(|(_, of)| of)
            .collect();
        let first = *all.first()?;
        assert!(
            all.iter().all(|&of| of == first),
            "{suffix}: one series, two declared totals"
        );
        Some(first)
    };
    match (declared(SERIES[0]), declared(SERIES[1])) {
        (Some(rep), Some(rep_plus)) => assert!(
            rep_plus > rep,
            "Repentance+ declares {rep_plus}, Repentance {rep}"
        ),
        _ => test_support::skip("both series are needed to compare two eras"),
    }
}
```

- [ ] **Step 2: Run it**

Run: `cargo test -p ipc --test preview_real -- --nocapture`
Expected: PASS. On this machine the third test skips (`rep+` is one file, so its `of` is read from
one snapshot — which still works: `declared` needs only one). Read the `sample:` and `skip:` lines
and check they name what you expect. **A `sample:` line does not mean the test used it** — if a
property returned early, the declaration is still printed.

- [ ] **Step 3: Commit**

```bash
git add crates/ipc/tests/preview_real.rs
git commit -m "test(ipc): the preview answers to the real series, as properties"
```

---

### Task 3: the preview reaches the wire with the candidates

**Files:**
- Modify: `crates/ipc/src/profile.rs`, `crates/ipc/src/contract.rs`, `crates/app/src/commands/profile.rs`
- Test: `crates/ipc/tests/setup.rs` (create if absent — check first with `ls crates/ipc/tests`)

**Interfaces:**
- Consumes: `preview_of` from Task 1.
- Produces:
  - `CandidateView.preview: Option<CandidatePreview>`
  - `pub fn setup_state(d: &Discovery, saved: Option<&ProfileId>, read: impl Fn(&Path) -> Option<Save>) -> SetupState`
  - `ipc::candidates` keeps its signature and leaves `preview: None`.

- [ ] **Step 1: Write the failing test**

In `crates/ipc/tests/setup.rs`:

```rust
//! `setup_state` carries the previews **with** the candidates, in one answer. Two commands
//! could not promise they describe the same list: a save that appears or disappears between
//! them leaves a card showing the numbers of a file no longer offered (N8, M4's Live screen).

use core_save::{Kind, Save, Section};
use discovery::{Discovery, SaveCandidate, SavePrefix, SaveSource};
use ipc::{setup_state, PreviewCount};
use std::path::{Path, PathBuf};

fn candidate(path: &str) -> SaveCandidate {
    SaveCandidate {
        path: PathBuf::from(path),
        prefix: SavePrefix::RepPlus,
        slot: 1,
        source: SaveSource::Override,
        modified: None,
        size: 12_000,
    }
}

fn discovery(saves: Vec<SaveCandidate>) -> Discovery {
    Discovery {
        steam: None,
        game: None,
        saves,
        game_data: None,
        diagnostics: Vec::new(),
    }
}

fn a_save() -> Save {
    Save {
        unknown_0x10: 0,
        sections: vec![Section {
            kind: Kind::Achievements,
            count: 4,
            f2: 0,
            offset: 0,
            bytes: vec![1, 1, 0, 1],
        }],
        diagnostics: Vec::new(),
    }
}

#[test]
fn every_candidate_carries_what_its_file_says() {
    let s = setup_state(&discovery(vec![candidate("a.dat")]), None, |_: &Path| {
        Some(a_save())
    });
    let preview = s.candidates[0].preview.expect("the reader answered");
    assert_eq!(preview.achievements, PreviewCount::Read { done: 2, of: 3 });
}

#[test]
fn a_candidate_whose_file_cannot_be_read_still_travels() {
    let s = setup_state(&discovery(vec![candidate("gone.dat")]), None, |_: &Path| None);
    assert_eq!(s.candidates.len(), 1, "it is offered");
    assert!(
        s.candidates[0].preview.is_none(),
        "and it says it could not be read, rather than showing zeros"
    );
}

#[test]
fn the_preview_follows_its_own_candidate_when_the_list_is_sorted() {
    // `candidates` sorts, so a preview matched by position would follow the wrong row.
    let d = discovery(vec![candidate("first.dat"), candidate("second.dat")]);
    let s = setup_state(&d, None, |p: &Path| {
        if p.ends_with("second.dat") {
            Some(a_save())
        } else {
            None
        }
    });
    let with_preview: Vec<bool> = s.candidates.iter().map(|c| c.preview.is_some()).collect();
    assert_eq!(
        with_preview.iter().filter(|&&b| b).count(),
        1,
        "exactly one file answered"
    );
}
```

Check `SaveCandidate`'s fields against `crates/discovery/src/lib.rs` before running; if a field
name differs, fix the fixture, not the crate.

- [ ] **Step 2: Run it and watch it fail**

Run: `cargo test -p ipc --test setup`
Expected: FAIL — `setup_state` takes 2 arguments, and `CandidateView` has no `preview`.

- [ ] **Step 3: Implement**

In `crates/ipc/src/profile.rs`:

```rust
pub struct CandidateView {
    // … existing fields …
    /// What the file says about itself. `None` when it could not be opened or parsed at all:
    /// the row is still offered — "degrade, never fail" shows it and says so, it does not
    /// decide a stranger has no save.
    pub preview: Option<CandidatePreview>,
}
```

`view_of` sets `preview: None`, so `candidates()` keeps its signature and its meaning: it is the
validation list `select_profile` uses, and it reads no files.

```rust
/// The previews are filled here rather than in `candidates`, and matched **by id**: the list
/// is sorted, so matching by position would hand a row its neighbour's numbers.
pub fn setup_state(
    d: &Discovery,
    saved: Option<&ProfileId>,
    read: impl Fn(&Path) -> Option<core_save::Save>,
) -> SetupState {
    let views: Vec<CandidateView> = candidates(&d.saves)
        .into_iter()
        .map(|v| {
            let path = d.saves.iter().find(|s| profile_id(&s.path) == v.id);
            let preview = path
                .and_then(|s| read(&s.path))
                .map(|save| crate::preview_of(&save));
            CandidateView { preview, ..v }
        })
        .collect();
    let active = resolve_active(saved, &views, d.steam.is_some(), d.game.is_some());
    // … the rest unchanged …
}
```

`ActiveProfile::Active` clones a `CandidateView`, so the active profile carries its preview too —
which is what the indicator and the re-entry need, at no extra read.

Add `core-save` to `crates/ipc/Cargo.toml`'s dependencies if it is not already a normal one (it is
used by `summary.rs`, so it should be).

In `crates/ipc/src/contract.rs`, **before** `decl::<crate::CandidateView>`:

```rust
decl::<crate::PreviewCount>(&cfg, &mut out);
decl::<crate::CandidatePreview>(&cfg, &mut out);
```

- [ ] **Step 4: Wire the app**

In `crates/app/src/commands/profile.rs`:

```rust
/// The I/O the pure crate does not do. A save is 11–12 KB and there is a handful of
/// candidates, so this is a read per launch and nothing worth caching: `SaveCache` has one
/// slot, for the active profile, and a failed read is never remembered.
fn read_save(path: &std::path::Path) -> Option<core_save::Save> {
    core_save::Save::open(path).ok()
}
```

and pass `read_save` at both call sites of `ipc::setup_state` in that file.

- [ ] **Step 5: Run everything and regenerate the contract**

```sh
cargo test -p ipc
pnpm ipc:types
```
Expected: tests pass; `ui/src/lib/ipc/types.ts` gains `CandidatePreview` and `PreviewCount`, and
`CandidateView` gains `preview`. **Never edit that file by hand.**

- [ ] **Step 6: Commit**

```bash
git add crates/ipc/src/profile.rs crates/ipc/src/contract.rs crates/ipc/tests/setup.rs crates/app/src/commands/profile.rs ui/src/lib/ipc/types.ts
git commit -m "feat(ipc): the candidates travel with what their files hold"
```

---

### Task 4: `welcomeState` — which state draws the takeover

**Files:**
- Create: `ui/src/lib/profile/welcomeView.ts`
- Modify: `ui/src/lib/profile/gateView.ts` (loses two states), `ui/src/screens/ProgressGate.vue`
- Test: `ui/src/lib/profile/welcomeView.test.ts`, and update `gateView`'s existing test if there is
  one (`ls ui/src/lib/profile`)

**Interfaces:**
- Consumes: `SetupState`, `ActiveProfile`, `MissingReason`, `LoadStatus`.
- Produces: `export const welcomeState = (setup, status, picking) => WelcomeState`, with
  `WelcomeState = { kind: 'hidden' } | { kind: 'failed' } | { kind: 'choose'; savedGone: boolean } | { kind: 'empty'; reason: MissingReason }`.

- [ ] **Step 1: Write the failing test**

`ui/src/lib/profile/welcomeView.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import { LoadStatus } from '@/stores/loadStatus'
import { MissingReason } from '@/lib/ipc/types'
import type { ActiveProfile, SetupState } from '@/lib/ipc/types'
import { candidates } from '@/lib/ipc/fixtures/profile'
import { welcomeState } from './welcomeView'

const setupWith = (active: ActiveProfile): SetupState => ({
  steam: null,
  game: null,
  candidates,
  active,
  diagnostics: [],
})

const active: ActiveProfile = {
  kind: 'active',
  profile: candidates[0],
  autoSelected: false,
}
const needsChoice: ActiveProfile = {
  kind: 'needsChoice',
  reason: { kind: 'neverChosen' },
  suggested: null,
}
const savedGone: ActiveProfile = {
  kind: 'needsChoice',
  reason: { kind: 'savedProfileGone', was: 'abc' },
  suggested: null,
}
const none: ActiveProfile = {
  kind: 'none',
  reason: MissingReason.SteamNotFound,
}

describe('welcomeState', () => {
  it('does not draw before the setup is known, so it never flashes', () => {
    expect(welcomeState(null, LoadStatus.Loading, false)).toEqual({
      kind: 'hidden',
    })
  })

  it('stands aside for a settled profile', () => {
    expect(welcomeState(setupWith(active), LoadStatus.Ready, false)).toEqual({
      kind: 'hidden',
    })
  })

  it('opens over a settled profile when the picker was asked for', () => {
    expect(welcomeState(setupWith(active), LoadStatus.Ready, true)).toEqual({
      kind: 'choose',
      savedGone: false,
    })
  })

  it('asks for a choice, and says when the saved one is gone', () => {
    expect(welcomeState(setupWith(needsChoice), LoadStatus.Ready, false)).toEqual(
      { kind: 'choose', savedGone: false },
    )
    expect(welcomeState(setupWith(savedGone), LoadStatus.Ready, false)).toEqual({
      kind: 'choose',
      savedGone: true,
    })
  })

  it('takes the broken chain too, with the link that broke', () => {
    expect(welcomeState(setupWith(none), LoadStatus.Ready, false)).toEqual({
      kind: 'empty',
      reason: MissingReason.SteamNotFound,
    })
  })

  it('lets a settled profile win over a failed reload', () => {
    // A reload that fails must not blank a window that was working a second ago — the rule
    // `gateState` already states for the gate.
    expect(welcomeState(setupWith(active), LoadStatus.Failed, false)).toEqual({
      kind: 'hidden',
    })
    expect(welcomeState(null, LoadStatus.Failed, false)).toEqual({
      kind: 'failed',
    })
  })
})
```

Check `ActiveProfile`'s exact field names in `ui/src/lib/ipc/types.ts` before running; if
`suggested` is absent or named otherwise, fix the fixture.

- [ ] **Step 2: Run it and watch it fail**

Run: `pnpm --filter ui test welcomeView`
Expected: FAIL — module not found.

- [ ] **Step 3: Implement**

```ts
import { assertNever } from '@/lib/assertNever'
import type { MissingReason, SetupState } from '@/lib/ipc/types'
import { LoadStatus } from '@/stores/loadStatus'

/**
 * Whether the window draws the welcome instead of the shell, and what it says. The takeover
 * is a **state**, not a route: here a route is a tab, and the session document would save and
 * restore it (3.7b).
 */
export type WelcomeState =
  /** The shell draws — a profile is settled, or nothing is known yet and a guess would flash. */
  | { kind: 'hidden' }
  /** The setup could not be read and there is no profile to fall back on. */
  | { kind: 'failed' }
  /** There are candidates. `savedGone` is the one case with a sentence of its own. */
  | { kind: 'choose'; savedGone: boolean }
  /** Nothing to choose, and we know which link of the chain broke. */
  | { kind: 'empty'; reason: MissingReason }

export const welcomeState = (
  setup: SetupState | null,
  status: LoadStatus,
  picking: boolean,
): WelcomeState => {
  const settled = setup !== null && setup.active.kind === 'active'
  if (settled && !picking) return { kind: 'hidden' }
  if (status === LoadStatus.Failed) return { kind: 'failed' }
  if (setup === null) return { kind: 'hidden' }
  switch (setup.active.kind) {
    case 'active':
      return { kind: 'choose', savedGone: false }
    case 'needsChoice':
      return {
        kind: 'choose',
        savedGone: setup.active.reason.kind === 'savedProfileGone',
      }
    case 'none':
      return { kind: 'empty', reason: setup.active.reason }
    default:
      return assertNever(setup.active)
  }
}
```

- [ ] **Step 4: Narrow the gate, which can no longer see two of its states**

With the welcome taking `needsChoice` and `none`, `ProgressGate` is only ever mounted with a
settled profile. Leaving `selection` and `blocked` there would be dead branches that read as
alternatives.

In `gateView.ts` remove the `selection` and `blocked` variants and the `fromActive` arms that
produce them — `case 'needsChoice'` and `case 'none'` both become `{ kind: 'waiting' }`, with a
comment saying the welcome owns them. In `ProgressGate.vue` remove the `Empty` block, the `Alert`,
the `ProfileScreen` import and the `openProfile` emit; in `App.vue` remove the `@open-profile`
binding. Update `gateView`'s existing test to match.

- [ ] **Step 5: Run the tests**

Run: `pnpm ui:test`
Expected: PASS. A red `gateView` test here is the expected consequence of Step 4, not a bug —
update it rather than restoring the branches.

- [ ] **Step 6: Commit**

```bash
git add ui/src/lib/profile/welcomeView.ts ui/src/lib/profile/welcomeView.test.ts ui/src/lib/profile/gateView.ts ui/src/screens/ProgressGate.vue ui/src/App.vue
git commit -m "feat(ui): the welcome decides its own state, and the gate stops guessing"
```

---

### Task 5: the takeover — the screen, the cards, the empty branch

**Files:**
- Create: `ui/src/screens/welcome/WelcomeScreen.vue`, `ui/src/screens/welcome/SaveCard.vue`,
  `ui/src/screens/welcome/NothingFound.vue`
- Modify: `ui/src/App.vue`, `ui/src/components/shell/TitleBar.vue`, `ui/src/stores/profile.ts`,
  `ui/src/i18n/messages/it.ts`, `ui/src/i18n/messages/en.ts`,
  `ui/src/lib/ipc/fixtures/profile.ts`, `ui/src/screens/ProfileScreen.vue`
- Delete: `ui/src/screens/profile/CandidatesCard.vue`
- Test: `ui/src/lib/profile/previewView.test.ts`

**Interfaces:**
- Consumes: `welcomeState` (Task 4), `CandidatePreview`/`PreviewCount` (Task 3), the profile store.
- Produces: `previewLines(preview: CandidatePreview | null): PreviewLine[]` in
  `ui/src/lib/profile/previewView.ts`, and `picking` / `pick()` / `stopPicking()` on the store.

- [ ] **Step 1: Write the failing test for the card's lines**

`ui/src/lib/profile/previewView.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import type { CandidatePreview } from '@/lib/ipc/types'
import { previewLines, unreadableNote } from './previewView'

const full: CandidatePreview = {
  achievements: { kind: 'read', done: 379, of: 640 },
  items: { kind: 'read', done: 612, of: 732 },
  marks: { kind: 'read', done: 92, of: 120 },
  unreadableCells: 0,
}

describe('previewLines', () => {
  it('gives one line per count, in the order the card reads', () => {
    expect(previewLines(full).map((l) => l.label)).toEqual([
      'welcome.card.achievements',
      'welcome.card.items',
      'welcome.card.marks',
    ])
    expect(previewLines(full)[0]).toMatchObject({ done: 379, of: 640 })
  })

  it('says a count could not be read instead of showing a zero', () => {
    const [line] = previewLines({ ...full, achievements: { kind: 'unread' } })
    expect(line.done).toBeNull()
    expect(line.of).toBeNull()
  })

  it('gives no lines at all when the file could not be parsed', () => {
    expect(previewLines(null)).toEqual([])
  })
})

describe('unreadableNote', () => {
  it('is silent on a whole file', () => {
    expect(unreadableNote(full)).toBeNull()
  })

  it('counts the cells when there are any', () => {
    expect(unreadableNote({ ...full, unreadableCells: 8 })).toEqual({
      key: 'welcome.card.unreadableCells',
      count: 8,
    })
  })

  it('speaks for a file that could not be read at all', () => {
    expect(unreadableNote(null)).toEqual({ key: 'welcome.card.unreadable', count: 0 })
  })
})
```

- [ ] **Step 2: Run it and watch it fail**

Run: `pnpm --filter ui test previewView`
Expected: FAIL — module not found.

- [ ] **Step 3: Write `previewView.ts`, then the three components**

`previewLines` maps the three counts onto `{ label, done, of }` with `null` for `Unread`;
`unreadableNote` returns the ⚠ line or `null`. Both are pure and take no `t`.

`SaveCard.vue` draws one candidate: edition and slot, where it was found, the three lines, the
relative date (`formatModified` from `profileView.ts`), the `suggested` badge, and the ⚠ note. It
is a `Button`-based selectable card — **no raw `<button>`** — and every size, gap and opacity is a
token, **no `w-[…]`, no `opacity-50`, no `:size="16"`**.

`NothingFound.vue` draws the `empty` branch: the sentence for the `MissingReason`
(`missingReasonLabel` already maps them), the diagnostics, a retry, and the two folder buttons
Task 6 wires. Until Task 6 they are not rendered at all — **a button that does nothing is worse
than none**.

`WelcomeScreen.vue` switches on `welcomeState` and emits `choose`.

- [ ] **Step 4: Put it above the router**

In `ui/src/App.vue`:

```vue
<TitleBar v-if="welcome.kind !== 'hidden'" :bare="true" :tabs="[]" :active-id="null" … />
<WelcomeScreen v-if="welcome.kind !== 'hidden'" :state="welcome" … />
<template v-else> … the shell as it is … </template>
```

with `const welcome = computed(() => welcomeState(profile.setup, profile.status, profile.picking))`.
`useWindowSession()` stays in the setup where it already is: it must keep running behind the
takeover, or a window would stop holding its tabs while the welcome is up.

In `TitleBar.vue` add `bare?: boolean` (default `false`) and `v-if="!bare"` on `<TabStrip>`. The
drag region and `WindowControls` stay: a window without them cannot be moved or closed.

In `stores/profile.ts` add `picking`, `pick()`, `stopPicking()`; `choose()` clears it. In
`App.vue`'s `AppEvent.ProfileChanged` handler call `profile.stopPicking()` beside `profile.load()`
— another window's choice settles this one too. Point `ProfileIndicator`'s `@open` at
`profile.pick()` instead of navigating to the Profile route.

- [ ] **Step 5: Fixtures, so it can be seen without a game**

In `ui/src/lib/ipc/fixtures/profile.ts` give the four candidates a `preview` each: the first full,
the second with `items: { kind: 'unread' }`, the third full with `unreadableCells: 8`, the fourth
`preview: null`. All four shapes then draw on `?fixture=pick`.

- [ ] **Step 6: Strip the Profile screen and delete the table**

Remove `CandidatesCard` from `ProfileScreen.vue` along with the `changing` ref and the `choose`
handler; delete `ui/src/screens/profile/CandidatesCard.vue`. Keep `ChainCard`, `SectionsCard`,
`NoSavesCard` and `ProfileError`. Move the `profile.pick.*` messages that the welcome reuses into
the new `welcome.*` block and delete the ones nothing reads.

- [ ] **Step 7: Run everything and look at it**

```sh
pnpm ui:test
pnpm typecheck
pnpm scan
pnpm ui:dev    # then open ?fixture=pick, ?fixture=none, ?fixture=active
```
Expected: green, and the three scenarios draw. `pnpm scan` is the one that catches a hardcoded
pixel or a visible string in a template.

- [ ] **Step 8: Commit**

```bash
git add ui/src/screens/welcome ui/src/lib/profile/previewView.ts ui/src/lib/profile/previewView.test.ts ui/src/App.vue ui/src/components/shell/TitleBar.vue ui/src/stores/profile.ts ui/src/i18n/messages ui/src/lib/ipc/fixtures/profile.ts ui/src/screens/ProfileScreen.vue
git rm ui/src/screens/profile/CandidatesCard.vue
git commit -m "feat(ui): the app opens by asking which save you are playing with"
```

---

### Task 6: B14 — the folder you point at by hand

**Files:**
- Modify: `crates/app/Cargo.toml`, `crates/app/src/lib.rs`, `crates/app/src/settings_file.rs`,
  `crates/app/src/commands/profile.rs`, `crates/ipc/src/profile.rs` (one `MissingReason` variant),
  `ui/src/lib/constants/commands.ts`, `ui/src/lib/ipc/setup.ts`,
  `ui/src/screens/welcome/NothingFound.vue`, the two i18n files
- Test: `crates/ipc/tests/setup.rs` (the new reason), `ui/src/lib/profile/welcomeView.test.ts`

**Interfaces:**
- Consumes: `setup_state` (Task 3), `discovery::Options`.
- Produces: commands `choose_game_folder` and `choose_saves_folder`, both
  `async fn(AppHandle) -> Result<SetupState, IpcError>`; `MissingReason::NoSavesInChosenFolder`.

**What the documentation says** (checked 2026-09-17 against `v2.tauri.app/plugin/dialog/` and
docs.rs): `pick_folder` is the callback form and is *"not a blocking operation, and should be used
when running on the main thread to avoid deadlocks with the event loop"*; `blocking_pick_folder`
*"should NOT be used when running on the main thread"*. Which thread a **synchronous** Tauri
command runs on is not stated on that page, so this plan takes the form that needs no inference:
an `async` command plus the callback, bridged by `tauri::async_runtime::channel` (a re-export of
`tokio::sync::mpsc`, verified in `tauri-2.11.5/src/async_runtime.rs`). `try_send`, never
`blocking_send`: the latter panics if the callback happens to run inside a runtime thread, and
this repo does not panic outside tests.

The capability file needs **no `dialog:*` entry**: capabilities gate `invoke` from the webview, and
this dialog is opened in Rust. The npm package `@tauri-apps/plugin-dialog` is not installed, which
is also what frontend rule 3 wants.

- [ ] **Step 1: Write the failing test for the new reason**

Add to `crates/ipc/tests/setup.rs`:

```rust
/// A folder you pointed at that holds no save is **not** "we found nothing". Two sentences,
/// because the reader can act on one of them and not on the other.
#[test]
fn a_chosen_folder_with_no_save_says_so() {
    let mut d = discovery(Vec::new());
    d.diagnostics.push(discovery::Diagnostic::NoSavesFound);
    let s = setup_state(&d, None, |_: &Path| None);
    assert!(matches!(
        s.active,
        ipc::ActiveProfile::None {
            reason: ipc::MissingReason::NoSaves
        }
    ));
    // …and with an override in play, the reason names it.
    let chosen = ipc::setup_state_with_override(&d, None, |_: &Path| None, true);
    assert!(matches!(
        chosen.active,
        ipc::ActiveProfile::None {
            reason: ipc::MissingReason::NoSavesInChosenFolder
        }
    ));
}
```

- [ ] **Step 2: Run it and watch it fail**

Run: `cargo test -p ipc --test setup`
Expected: FAIL — no such variant, no such function.

- [ ] **Step 3: Implement the reason**

Add `NoSavesInChosenFolder` to `MissingReason` (fieldless — it stays a bare camelCase string), and
give `missing_reason` the extra input. Rather than a second public entry point, add the flag to
`setup_state` itself and let the existing call sites pass `false`; if that churns too many call
sites, keep `setup_state_with_override` as the wider function and make `setup_state` call it with
`false` — pick one and make the test match it.

- [ ] **Step 4: The plugin and the persisted folders**

`crates/app/Cargo.toml`:

```toml
tauri-plugin-dialog = "2.7.3"
```

`crates/app/src/lib.rs`, beside the other plugins:

```rust
.plugin(tauri_plugin_dialog::init())
```

`crates/app/src/settings_file.rs` — the folders live here and **never in `ipc::Settings`**, which
crosses the IPC and may hold no path:

```rust
/// The whole settings file. `ipc::Settings` is the half that crosses the IPC; the two folders
/// are paths, so they stay on this side of the boundary and are read only by `discover`.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Stored {
    #[serde(flatten)]
    pub settings: Settings,
    pub game_dir: Option<PathBuf>,
    pub save_dir: Option<PathBuf>,
}
```

`load` returns `Stored`, `save` takes `&Stored`. Every existing caller of `load(&app)` gains
`.settings`, and every `save(&app, &settings)` becomes a `Stored { settings, ..load(&app) }` — the
same "update only the field that changed" shape `select_profile` already uses, and for the same
reason: a partial write here would silently forget the folder the user chose.

Add a helper the commands share:

```rust
/// The options `discover` is given: what the user pointed at, first.
pub fn options(app: &AppHandle) -> Options {
    let stored = load(app);
    Options {
        steam_root: None,
        game_dir: stored.game_dir,
        save_dir: stored.save_dir,
    }
}
```

and replace every `discover(&Options::default())` in `commands/profile.rs` with
`discover(&settings_file::options(&app))`.

- [ ] **Step 5: The two commands**

```rust
/// Asks for a folder and answers the discovery that comes out of it. **The path travels
/// inward only**: what goes back is a `SetupState`, whose `pathHint` is redacted.
///
/// `async` plus the callback form is the documented-safe pair: `blocking_pick_folder` must not
/// run on the main thread, and which thread a synchronous command runs on is not documented.
/// `try_send` rather than `blocking_send`: the channel has room for the one message, and
/// `blocking_send` panics if the callback runs inside a runtime thread.
async fn ask_for_folder(app: &AppHandle) -> Option<PathBuf> {
    let (tx, mut rx) = tauri::async_runtime::channel(1);
    app.dialog().file().pick_folder(move |picked| {
        let _ = tx.try_send(picked);
    });
    rx.recv().await.flatten().and_then(|p| p.into_path().ok())
}

#[tauri::command]
pub async fn choose_saves_folder(app: AppHandle) -> Result<SetupState, IpcError> {
    let Some(dir) = ask_for_folder(&app).await else {
        // Cancelled: not an error, and not a change. The state as it is.
        return setup_state_now(&app);
    };
    let stored = Stored {
        save_dir: Some(dir),
        ..settings_file::load(&app)
    };
    settings_file::save(&app, &stored)?;
    announce(&app, PROFILE_CHANGED);
    setup_state_now(&app)
}
```

`choose_game_folder` is the same with `game_dir`. `setup_state_now` is the two lines
`setup_state` already runs, extracted so the three commands share them. Register both in
`generate_handler!` in `crates/app/src/lib.rs`, and add `ChooseGameFolder: 'choose_game_folder'`
and `ChooseSavesFolder: 'choose_saves_folder'` to `ui/src/lib/constants/commands.ts` with their
wrappers in `ui/src/lib/ipc/setup.ts`.

- [ ] **Step 6: Render the two buttons**

In `NothingFound.vue` add them, calling the typed wrappers through the store — **no `invoke()` in
a component**. Add the sentence for `NoSavesInChosenFolder` to both message files, and extend
`missingReasonLabel` in `profileLabels.ts`; the `assertNever` there will fail the typecheck until
you do, which is the point.

- [ ] **Step 7: Run it**

```sh
cargo test --workspace
pnpm ipc:types && pnpm typecheck && pnpm ui:test && pnpm scan
```
Expected: green. The dialog itself cannot be verified here — it needs a real window, and that is
one line of the UAT list, not a step of this plan.

- [ ] **Step 8: Commit**

```bash
git add crates/app crates/ipc ui/src/lib/constants/commands.ts ui/src/lib/ipc/setup.ts ui/src/lib/ipc/types.ts ui/src/lib/profile/profileLabels.ts ui/src/screens/welcome ui/src/i18n/messages
git commit -m "feat(app): the folder you point at by hand, and the sentence for an empty one"
```

---

### Task 7: the documents, the floor, the board

**Files:**
- Create: `docs/superpowers/reports/2026-09-17-welcome-flow-report.md`
- Modify: `docs/STATUS.md`, `docs/BACKLOG.md`, `docs/completed/2026-09.md`, `scripts/test-floor`

- [ ] **Step 1: Run the whole gate**

Run: `pnpm check`
Expected: everything green, and a line saying the test counts rose, with the numbers to paste.

- [ ] **Step 2: Raise the floor**

Put the printed `RUST_TESTS` and `UI_TESTS` into `scripts/test-floor`. **In this commit**, not a
later one: the floor is what makes a shrinking suite audible.

- [ ] **Step 3: Write the report**

`docs/superpowers/reports/2026-09-17-welcome-flow-report.md`: what landed, the two corrections
this plan made to its own spec (no `diagnostics` on the preview; the folders out of
`ipc::Settings`), the achievements denominator being `declared - 1` and why the Unlock screen's
642 is a different number, and **what nobody has seen** — the takeover in a real Tauri window, and
the folder dialog, which needs one.

- [ ] **Step 4: Move the state**

In `docs/STATUS.md`: M1's line — B17's welcome flow is no longer what keeps it open. Add the
window checks to *"What only a window can say"*. In `docs/BACKLOG.md`: B17 and B14 close with a
pointer to the report; the prose stays, the state does not. On the Trello board move both cards to
`UAT` — they are built and waiting to be looked at, which is exactly what that list is for.

- [ ] **Step 5: Commit and push**

```bash
git add docs scripts/test-floor
git commit -m "docs: the welcome flow lands, and what only a window can say grows by two"
git push
```

---

## Self-review

**Spec coverage.** §2 takeover above the router → Tasks 4 and 5. §3 one command with the reader
closure → Task 3. §4 the three counts and their `Unread` → Tasks 1 and 2. §5 the way back and the
Profile screen → Task 5, steps 4 and 6. §6 B14 and the empty chosen folder → Task 6. §7 what is
not decided → nothing to implement. §8 tests → inside each task, plus Task 7's floor. §9 files →
the table matches the tasks, with `preview.rs` created in Task 1.

**Type consistency.** `preview_of`, `CandidatePreview`, `PreviewCount`, `marks_totals`,
`welcomeState`, `previewLines`, `unreadableNote`, `picking`/`pick`/`stopPicking`,
`choose_game_folder`/`choose_saves_folder`, `Stored`, `MissingReason::NoSavesInChosenFolder` —
each is defined in one task and used under the same name in the others.

**Two places the implementer has to look before typing**, called out in the steps rather than
guessed here: `SaveCandidate`'s field names (Task 3, Step 1) and `ActiveProfile`'s shape in
`types.ts` (Task 4, Step 1).
