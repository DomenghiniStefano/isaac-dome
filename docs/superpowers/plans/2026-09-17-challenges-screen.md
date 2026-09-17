# 3.11 — the Challenges screen: implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this
> plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** a `Sfide` screen under Progress: the 45 challenges, what you have done, what each one
needs and unlocks, and its reward into the Plan's queue.

**Architecture:** one `ipc::challenges_view` joining section 7, the catalog's `challenges.xml` and
the wiki's challenge pages; one read command; one screen on 3.10's `FilterBar`. No write path is
added — `queue_add` already takes an achievement, and a challenge's reward is one.

**Tech Stack:** Rust (ipc, app), Vue 3 + TypeScript, Tailwind v4, vue-i18n, Vitest.

**Spec:** `docs/superpowers/specs/2026-09-17-challenges-screen-design.md`

## Global Constraints

- **Tauri commands return `Result<T, IpcError>`**, never `Result<T, String>`.
- **Every struct crossing the IPC** carries `#[serde(rename_all = "camelCase")]`; a **tagged** enum
  with struct variants needs `rename_all_fields = "camelCase"` **in addition**, or a field comes
  out `snake_case` and TypeScript reads `undefined` in silence.
- **No `_ =>` arm on a closed enum**, no `unwrap()`/`panic!` outside tests on data read from disk.
- **An expected case is not an error**: no game, no wiki, an unread section all travel in the
  payload as diagnostics.
- **The entry count is read from the file, never hardcoded** — `totals.slots` is section 7's own
  length, never 46.
- **`ui/src/lib/ipc/types.ts` is generated**: `pnpm ipc:types`, never edited by hand.
- **Frontend**: no `<style>` in SFCs, no hardcoded visual constants, no `invoke()` in components,
  no raw `<button>`/`<input>`, no string unions, every visible word through `t()`.
- **Test-first** for anything with logic; the expected value comes from the spec.
- **Commits**: `type(scope): subject`, scopes `ipc`, `app`, `ui`, or none for repo-wide docs.
  English, no `Co-Authored-By`, never a reference to Claude.
- **Branch**: `feature/challenges`, already cut from `develop`, already holding the measurement
  and the spec. Nothing merges into `master`.
- **`docs/architecture.md` IS redrawn by this work** — a route and a command both move, and its
  header pins 14 and 30. Same commit as the code that moves them.

---

### Task 1: the view-model and its judgment

**Files:**
- Create: `crates/ipc/src/challenges.rs`
- Modify: `crates/ipc/src/lib.rs` (declare and re-export the module)
- Create: `crates/ipc/tests/challenges.rs`

**Interfaces:**
- Consumes: `catalog::Catalog`, `wiki::Dataset`, `ipc::IconRef`, `core_save` flags as `&[bool]`.
- Produces:
  ```rust
  pub fn challenges_view(
      catalog: Option<&Catalog>,
      dataset: Option<&Dataset>,
      challenges: Option<&[bool]>,   // section 7
      achievements: Option<&[bool]>, // section 1
      icon: impl FnMut(&IconRef) -> Option<String>,
  ) -> ChallengesView
  ```
  with `ChallengesView`, `ChallengeRow`, `ChallengeStateView`, `RewardView`, `ChallengeTotals`,
  `ChallengesDiagnostic`.

- [ ] **Step 1: Write the failing tests**

Create `crates/ipc/tests/challenges.rs`. The synthetic catalog is built the way
`crates/ipc/tests/collection.rs` builds one — XML strings, no sample needed:

```rust
//! The forty-five as the UI sees them: section 7 joined with `challenges.xml`. What the tests
//! defend is that "unread" never reads as "not done", and that a gate that is not done is
//! *named* rather than summarised into a colour.

use catalog::Catalog;
use ipc::{challenges_view, ChallengeStateView, ChallengesDiagnostic, ChallengesView};
use serde_json::{json, to_value};

// Challenge 1 is free, 2 is gated by achievement 1, 3 by achievements 1 and 2.
const CHALLENGES: &[u8] = b"<challenges><challenge id=\"1\" name=\"Pitch Black\" startingitems=\"1\" /><challenge id=\"2\" name=\"High Brow\" achievements=\"1\" /><challenge id=\"3\" name=\"Head Trauma\" achievements=\"1,2\" /></challenges>";
const ACH: &[u8] = b"<achievements gfxroot=\"gfx/ui/achievement/\"><achievement id=\"1\" text=\"t1\" gfx=\"1.png\" /><achievement id=\"2\" text=\"t2\" gfx=\"2.png\" /></achievements>";

fn catalog() -> Catalog {
    Catalog::build(|p| match p {
        "challenges.xml" => Some(CHALLENGES.to_vec()),
        "achievements.xml" => Some(ACH.to_vec()),
        _ => None,
    })
}

// Section 7, cell 0 unused: challenge 1 done, 2 and 3 not.
const CELLS: [bool; 4] = [false, true, false, false];
// Achievement 1 done, 2 not.
const DONE: [bool; 3] = [false, true, false];

fn view(cells: Option<&[bool]>, achievements: Option<&[bool]>) -> ChallengesView {
    challenges_view(Some(&catalog()), None, cells, achievements, |_| None)
}

#[test]
fn the_state_shapes_are_pinned() {
    assert_eq!(
        to_value(ChallengeStateView::Done).expect("serializes"),
        json!({ "kind": "done" })
    );
    // The field is `missing`, camelCased inside a struct variant: without
    // `rename_all_fields` this key would come out snake_case and the screen would read
    // undefined with no error at all.
    assert_eq!(
        to_value(ChallengeStateView::Blocked {
            missing: vec![2],
        })
        .expect("serializes"),
        json!({ "kind": "blocked", "missing": [2] })
    );
}

#[test]
fn a_set_cell_is_done_and_the_cell_is_the_challenge_number() {
    let v = view(Some(&CELLS), Some(&DONE));
    let one = v.challenges.iter().find(|c| c.number == 1).expect("challenge 1");
    assert_eq!(one.state, ChallengeStateView::Done);
    let two = v.challenges.iter().find(|c| c.number == 2).expect("challenge 2");
    assert_ne!(two.state, ChallengeStateView::Done);
}

#[test]
fn a_challenge_with_no_gate_is_available_and_one_with_every_gate_done_too() {
    // Nothing done at all: challenge 1 is free, so it is available; 2 and 3 are not.
    let none = [false, false, false];
    let v = challenges_view(Some(&catalog()), None, Some(&[false; 4]), Some(&none), |_| None);
    let state = |n: u32| v.challenges.iter().find(|c| c.number == n).expect("row").state.clone();
    assert_eq!(state(1), ChallengeStateView::Available);
    assert_eq!(state(2), ChallengeStateView::Blocked { missing: vec![1] });
}

#[test]
fn a_blocked_challenge_names_the_gates_it_is_waiting_for() {
    // Achievement 1 done, 2 not: challenge 3 waits for 2 alone, and says so.
    let v = view(Some(&CELLS), Some(&DONE));
    let three = v.challenges.iter().find(|c| c.number == 3).expect("challenge 3");
    assert_eq!(three.state, ChallengeStateView::Blocked { missing: vec![2] });
}

#[test]
fn an_unread_section_is_unknown_and_never_not_done() {
    let v = view(None, Some(&DONE));
    assert!(v
        .challenges
        .iter()
        .all(|c| c.state == ChallengeStateView::Unknown));
    assert_eq!(v.totals.done, 0);
    assert_eq!(v.totals.slots, 0, "no section, no length to state");
    assert!(v
        .diagnostics
        .contains(&ChallengesDiagnostic::NoChallengesSection));
}

#[test]
fn the_totals_read_the_section_length_from_the_file() {
    let v = view(Some(&CELLS), Some(&DONE));
    assert_eq!(v.totals.slots, 4, "the sample's own length, never 46");
    assert_eq!(v.totals.challenges, 3);
    assert_eq!(v.totals.done, 1);
}

#[test]
fn without_a_catalog_there_are_no_rows_and_the_count_still_speaks() {
    let v = challenges_view(None, None, Some(&CELLS), Some(&DONE), |_| None);
    assert!(v.challenges.is_empty());
    assert_eq!(v.totals.done, 1, "the cells are still countable without names");
    assert!(v.diagnostics.contains(&ChallengesDiagnostic::NoCatalog));
}

#[test]
fn without_the_wiki_the_conditions_are_absent_and_said_once() {
    let v = view(Some(&CELLS), Some(&DONE));
    assert!(v.challenges.iter().all(|c| c.blindfolded.is_none()));
    assert_eq!(
        v.diagnostics
            .iter()
            .filter(|d| **d == ChallengesDiagnostic::NoWiki)
            .count(),
        1,
        "one diagnostic, not one per row"
    );
}
```

- [ ] **Step 2: Run them and watch them fail**

Run: `cargo test -p ipc --test challenges`
Expected: FAIL to compile — `challenges_view` does not exist.

- [ ] **Step 3: Write the module**

Create `crates/ipc/src/challenges.rs`. The shape mirrors `collection.rs`, which is the same kind
of list:

```rust
//! The forty-five challenges as the UI reads them: section 7 joined with `challenges.xml`, and
//! the wiki's page for the conditions that decide whether one is playable tonight.
//!
//! **Challenge `n` is cell `n`, and cell 0 is unused** — measured 2026-09-17, see
//! `docs/save-format.md`. The count comes from the section, never from the constant 46.

use catalog::Catalog;
use serde::Serialize;
use wiki::{Dataset, Inline, Infobox, Target};

use crate::icon::IconRef;

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ChallengesView {
    pub challenges: Vec<ChallengeRow>,
    pub totals: ChallengeTotals,
    pub diagnostics: Vec<ChallengesDiagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeTotals {
    /// Section 7's own length; 0 when it wasn't read. Never the constant 46.
    pub slots: u32,
    pub challenges: u32,
    pub done: u32,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeRow {
    pub number: u32,
    pub name: String,
    pub state: ChallengeStateView,
    /// What finishing it grants. Empty for the challenges that grant nothing.
    pub rewards: Vec<RewardView>,
    /// The wiki's, when it names one. `None` is "no page for this challenge", never "no
    /// character".
    pub character: Option<Target>,
    pub goal: Option<Vec<Inline>>,
    pub blindfolded: Option<bool>,
    pub page: Option<Target>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct RewardView {
    pub achievement: u32,
    pub text: Option<String>,
    pub icon_url: Option<String>,
    pub page: Option<Target>,
    /// Whether the save says it is already earned. `None` when section 1 wasn't read.
    pub done: Option<bool>,
}

/// Tagged, because `Blocked` carries the gates it waits for — the repo's rule, and the same
/// shape `LockView` has.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ChallengeStateView {
    Done,
    Available,
    /// The achievements that are not done yet. Named rather than counted: the reading of
    /// `unlocked_by` as "all of these" is not settled (spec §4), so a reader has to be able
    /// to see the claim and disbelieve it.
    Blocked { missing: Vec<u32> },
    /// Section 7 wasn't read. Never "not done".
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ChallengesDiagnostic {
    NoCatalog,
    NoChallengesSection,
    NoAchievementSection,
    NoWiki,
}

pub fn challenges_view(
    catalog: Option<&Catalog>,
    dataset: Option<&Dataset>,
    challenges: Option<&[bool]>,
    achievements: Option<&[bool]>,
    mut icon: impl FnMut(&IconRef) -> Option<String>,
) -> ChallengesView {
    let slots = challenges.map_or(0, |f| f.len() as u32);
    // Cell 0 is no challenge: it is skipped here exactly as the Collection skips slot 0.
    let done_cells = challenges.map_or(0, |f| f.iter().skip(1).filter(|b| **b).count() as u32);

    let mut diagnostics = Vec::new();
    if challenges.is_none() {
        diagnostics.push(ChallengesDiagnostic::NoChallengesSection);
    }
    if achievements.is_none() {
        diagnostics.push(ChallengesDiagnostic::NoAchievementSection);
    }
    if dataset.is_none() {
        diagnostics.push(ChallengesDiagnostic::NoWiki);
    }

    let Some(c) = catalog else {
        diagnostics.push(ChallengesDiagnostic::NoCatalog);
        return ChallengesView {
            challenges: Vec::new(),
            totals: ChallengeTotals {
                slots,
                challenges: 0,
                done: done_cells,
            },
            diagnostics,
        };
    };

    let mut listed: Vec<_> = c.challenges().collect();
    listed.sort_by_key(|ch| ch.id.0);

    let rows: Vec<ChallengeRow> = listed
        .iter()
        .map(|ch| {
            let number = ch.id.0;
            let finished = challenges.and_then(|f| f.get(number as usize).copied());
            let missing: Vec<u32> = ch
                .unlocked_by
                .iter()
                .filter(|a| !achievements.and_then(|f| f.get(a.0 as usize).copied()).unwrap_or(false))
                .map(|a| a.0)
                .collect();
            let state = match finished {
                None => ChallengeStateView::Unknown,
                Some(true) => ChallengeStateView::Done,
                Some(false) if missing.is_empty() => ChallengeStateView::Available,
                Some(false) => ChallengeStateView::Blocked { missing },
            };
            let page = dataset.and_then(|d| {
                d.entry(&Target::Challenge { number })
                    .map(|_| Target::Challenge { number })
            });
            let infobox = dataset
                .and_then(|d| d.entry(&Target::Challenge { number }))
                .and_then(|e| e.infobox.as_ref());
            let (character, goal, blindfolded) = match infobox {
                Some(Infobox::Challenge {
                    character,
                    goal,
                    blindfolded,
                    ..
                }) => (character.clone(), Some(goal.clone()), Some(*blindfolded)),
                _ => (None, None, None),
            };
            ChallengeRow {
                number,
                name: ch.name.clone(),
                state,
                rewards: ch
                    .rewards
                    .iter()
                    .map(|a| RewardView {
                        achievement: a.0,
                        text: c.achievement(*a).map(|x| x.text.clone()),
                        icon_url: icon(&IconRef::Achievement { id: a.0 }),
                        page: dataset.and_then(|d| {
                            d.entry(&Target::Achievement { id: a.0 })
                                .map(|_| Target::Achievement { id: a.0 })
                        }),
                        done: achievements.and_then(|f| f.get(a.0 as usize).copied()),
                    })
                    .collect(),
                character,
                goal,
                blindfolded,
                page,
            }
        })
        .collect();

    let done = rows
        .iter()
        .filter(|r| r.state == ChallengeStateView::Done)
        .count() as u32;
    ChallengesView {
        totals: ChallengeTotals {
            slots,
            challenges: rows.len() as u32,
            done,
        },
        challenges: rows,
        diagnostics,
    }
}
```

In `crates/ipc/src/lib.rs`, declare the module beside `collection` and re-export its types the
same way that one is re-exported.

**If the catalog's API differs** — `c.challenges()` not being an iterator, `achievement(id)` not
existing, `Challenge.name` needing `c.text(&ch.name, Language::English)` the way items do — follow
what `collection.rs` and `crates/catalog/src/catalog.rs` actually do. Do not invent an accessor.

- [ ] **Step 4: Run the tests and watch them pass**

Run: `cargo test -p ipc --test challenges`
Expected: PASS, 8 tests.

- [ ] **Step 5: Mutate one test to check it can go red**

Temporarily change the `Blocked` arm to `Blocked { missing: Vec::new() }` and confirm
`a_blocked_challenge_names_the_gates_it_is_waiting_for` fails. Put it back.

- [ ] **Step 6: Commit**

```bash
git add crates/ipc/src/challenges.rs crates/ipc/src/lib.rs crates/ipc/tests/challenges.rs
git commit -m "feat(ipc): the forty-five challenges, with the gates a blocked one waits for"
```

---

### Task 2: the guard on real data

**Files:**
- Create: `crates/ipc/tests/challenges_real.rs`

The measurement of 2026-09-17 becomes the property that keeps answering — the same move
`progress_real.rs` makes for the Greed column. It needs `samples/packed` and a dated save, and
**declares its skip** when either is missing.

- [ ] **Step 1: Write the test**

```rust
//! The 2026-09-17 measurement, turned into the property that guards it.
//!
//! Challenge `n` is cell `n`: the instrument is the catalog's reward link, so "the cell is set"
//! and "every reward achievement is done" have to agree. Not a pinned number — it holds whatever
//! the profile does next, and it fails loudly if the mapping ever shifts.

use catalog::Catalog;
use core_save::{Kind, Save};
use ipc::{challenges_view, ChallengeStateView};
use test_support::dated_series;

const SERIES: &str = "rep+persistentgamedata1.dat";

fn real_catalog() -> Option<Catalog> {
    let packed = test_support::packed_dir()?;
    let rs = unpack::ResourceSet::open(&packed);
    Some(Catalog::build(|p| rs.read(p)))
}

#[test]
fn a_finished_challenge_has_earned_every_achievement_it_rewards() {
    let Some(c) = real_catalog() else {
        test_support::skip("samples/packed is missing, no catalog to join");
        return;
    };
    let files = dated_series(SERIES);
    let Some(path) = files.last() else {
        test_support::skip("no rep+ sample: the mapping has nothing to check");
        return;
    };
    let bytes = std::fs::read(path).expect("the sample reads");
    let save = Save::parse(&bytes).expect("the sample parses");
    let view = challenges_view(
        Some(&c),
        wiki::Dataset::embedded().ok().as_ref(),
        save.flags(Kind::Challenges).as_deref(),
        save.flags(Kind::Achievements).as_deref(),
        |_| None,
    );

    // The vacuity guard: a series where nothing is done, or everything is, proves nothing.
    let done = view
        .challenges
        .iter()
        .filter(|r| r.state == ChallengeStateView::Done)
        .count();
    assert!(
        done > 0 && done < view.challenges.len(),
        "this profile has {done} of {} done: with none or all, the property is vacuous",
        view.challenges.len()
    );

    for row in &view.challenges {
        if row.state != ChallengeStateView::Done || row.rewards.is_empty() {
            continue;
        }
        assert!(
            row.rewards.iter().all(|r| r.done == Some(true)),
            "challenge {} reads done, and {:?} is not earned — the cell mapping shifted",
            row.number,
            row.rewards.iter().filter(|r| r.done != Some(true)).map(|r| r.achievement).collect::<Vec<_>>()
        );
    }
}

#[test]
fn every_challenge_has_a_wiki_page_in_the_committed_snapshot() {
    let Some(c) = real_catalog() else {
        test_support::skip("samples/packed is missing, no challenge list to check");
        return;
    };
    let view = challenges_view(
        Some(&c),
        wiki::Dataset::embedded().ok().as_ref(),
        None,
        None,
        |_| None,
    );
    let without: Vec<u32> = view
        .challenges
        .iter()
        .filter(|r| r.page.is_none())
        .map(|r| r.number)
        .collect();
    assert!(
        without.is_empty(),
        "the snapshot has no page for {without:?} — a row would lose its conditions in silence"
    );
}
```

- [ ] **Step 2: Run it**

Run: `cargo test -p ipc --test challenges_real -- --nocapture`
Expected: PASS, with a `sample:` line naming the file it used. On a machine without the game it
prints `skip:` and passes.

- [ ] **Step 3: Commit**

```bash
git add crates/ipc/tests/challenges_real.rs
git commit -m "test(ipc): a finished challenge has earned what it rewards, on the real series"
```

---

### Task 3: the command

**Files:**
- Create or modify: `crates/app/src/commands/graph.rs` (beside `collection`)
- Modify: `crates/app/src/lib.rs` (`generate_handler!`)
- Modify: `ui/src/lib/ipc/types.ts` (generated, never by hand)

- [ ] **Step 1: Write the command**

In `crates/app/src/commands/graph.rs`, directly under `collection`:

```rust
/// The forty-five challenges for the active profile. Wiring only: the join is `ipc`'s.
#[tauri::command]
pub fn challenges(
    app: AppHandle,
    state: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
) -> Result<ipc::ChallengesView, IpcError> {
    let (_, save) = active_save(&app)?;
    // Game not installed is expected: the view goes out without a catalog and says so.
    let resources = resources.get();
    let catalog = resources.and_then(|rs| state.get_or_build(rs));
    let challenges = save.flags(Kind::Challenges);
    let achievements = save.flags(Kind::Achievements);
    Ok(ipc::challenges_view(
        catalog,
        wiki::Dataset::embedded().ok().as_ref(),
        challenges.as_deref(),
        achievements.as_deref(),
        icon_url,
    ))
}
```

Copy the `#[tauri::command]` attribute and the argument style from `collection` right above it —
if that one is `async`, this one is too.

- [ ] **Step 2: Register it**

In `crates/app/src/lib.rs`, add `graph::challenges,` to `generate_handler!`, beside
`graph::collection`.

- [ ] **Step 3: Regenerate the contract**

Run: `pnpm ipc:types`
Expected: `ui/src/lib/ipc/types.ts` gains `ChallengesView`, `ChallengeRow`, `ChallengeStateView`,
`RewardView`, `ChallengeTotals`, `ChallengesDiagnostic`. **Read the diff**: a struct variant whose
field came out `snake_case` means a missing `rename_all_fields`, and it is silent at runtime.

- [ ] **Step 4: Check it builds**

Run: `cargo clippy --all-targets -- -D warnings`
Expected: clean.

- [ ] **Step 5: Commit**

```bash
git add crates/app/src/commands/graph.rs crates/app/src/lib.rs ui/src/lib/ipc/types.ts
git commit -m "feat(app): the challenges command, and the contract it regenerates"
```

---

### Task 4: the frontend's plumbing

**Files:**
- Modify: `ui/src/lib/constants/commands.ts` (`Challenges: 'challenges'`)
- Modify: `ui/src/lib/constants/stores.ts` (a `StoreId` for it)
- Create: `ui/src/lib/ipc/challenges.ts`
- Modify: `ui/src/stores/views.ts`
- Create: `ui/src/lib/ipc/fixtures/challenges.ts`
- Modify: `ui/src/lib/ipc/fixtures/index.ts`

- [ ] **Step 1: The typed wrapper**

Create `ui/src/lib/ipc/challenges.ts`, exactly the shape of `collection.ts`:

```ts
import { Command } from '../constants/commands'
import { call } from './transport'
import type { ChallengesView } from './types'

// The active profile's challenges joined with the catalog's `challenges.xml`.
export const challenges = (): Promise<ChallengesView> => call(Command.Challenges)
```

- [ ] **Step 2: The store**

In `ui/src/stores/views.ts`, beside `useCollectionStore`:

```ts
// The active profile's challenges.
export const useChallengesStore = defineViewStore<ChallengesView>(
  StoreId.Challenges,
  challenges,
)
```

- [ ] **Step 3: The fixture**

Create `ui/src/lib/ipc/fixtures/challenges.ts` answering with a handful of rows that cover
**every** state — done, available, blocked with two gates named, and unknown — plus one row with
no wiki conditions, because that is the shape a screen gets wrong. Follow
`ui/src/lib/ipc/fixtures/collection.ts` for how a fixture declares what it is synthesising
(it warns on the console; this one says the conditions are hand-written until the design pack
carries challenges).

Wire it in `ui/src/lib/ipc/fixtures/index.ts`:

```ts
  [Command.Challenges]: (_args, scenario) =>
    whenActive(scenario, () => challengesFixture()),
```

- [ ] **Step 4: Check**

Run: `pnpm typecheck && pnpm lint`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add ui/src/lib ui/src/stores/views.ts
git commit -m "feat(ui): the challenges view reaches the frontend, with a fixture per state"
```

---

### Task 5: the screen

**Files:**
- Create: `ui/src/screens/ChallengesScreen.vue`
- Create: `ui/src/screens/challenges/ChallengesTable.vue`
- Create: `ui/src/screens/challenges/ChallengeRow.vue`
- Create: `ui/src/screens/challenges/challengeLabels.ts`
- Create: `ui/src/lib/challenges/challengeFacets.ts` + `challengeFacets.test.ts`
- Create: `ui/src/lib/challenges/challengeState.ts` + `challengeState.test.ts`
- Modify: `ui/src/i18n/messages/it.ts`, `ui/src/i18n/messages/en.ts`

**Interfaces:**
- Consumes: `FilterBar` and `FacetSlot` from 3.10, `queueAdd` the Plan already calls, the wiki
  inline renderer the Wiki screen uses.
- Produces: `challengeFaceting`, `ChallengeFacet`, `challengeState(row)`, `challengeStateCounts`
  are **not** needed — the bar counts states itself since 3.10.

- [ ] **Step 1: Write the facet tests first**

`ui/src/lib/challenges/challengeFacets.test.ts`, on the engine `lib/facets/faceting.ts`:

```ts
import { describe, expect, it } from 'vitest'
import { ChallengeFacet, challengeFaceting } from './challengeFacets'
import type { ChallengeRow } from '@/lib/ipc/types'

const row = (over: Partial<ChallengeRow>): ChallengeRow => ({
  number: 1,
  name: 'Pitch Black',
  state: { kind: 'available' },
  rewards: [],
  character: null,
  goal: null,
  blindfolded: null,
  page: null,
  ...over,
})

describe('the challenge facets', () => {
  it('reads the state as its tag, so a blocked row is one value and not N', () => {
    const rows = [
      row({ number: 1, state: { kind: 'done' } }),
      row({ number: 2, state: { kind: 'blocked', missing: [1, 2] } }),
    ]
    expect(
      challengeFaceting.options(rows, ChallengeFacet.State),
    ).toContain('blocked')
  })

  // A row the wiki has nothing for contributes **no value**: `blindfolded: null` is "we have
  // no page", and offering it as "not blindfolded" would answer a question nobody asked.
  it('a row with no page answers the blindfolded facet with nothing', () => {
    const rows = [row({ blindfolded: null })]
    expect(challengeFaceting.options(rows, ChallengeFacet.Blindfolded)).toEqual([])
  })

  it('the search reads the name and the number', () => {
    const rows = [row({ number: 33, name: 'Onan’s Streak' })]
    const filter = { ...challengeFaceting.empty(), query: '33' }
    expect(challengeFaceting.matches(rows[0], filter)).toBe(true)
  })
})
```

- [ ] **Step 2: Run them and watch them fail**

Run: `pnpm ui:test -- challengeFacets`
Expected: FAIL — the module does not exist.

- [ ] **Step 3: Write `challengeFacets.ts`**

Four facets on the shared engine, the same shape `runFacets.ts` has: `State`, `Character`,
`Unlocks`, `Blindfolded`. `values()` returns `[]` for what the row does not know — never a
made-up value. The search text is the name plus the number.

- [ ] **Step 4: Run them and watch them pass**

Run: `pnpm ui:test -- challengeFacets`
Expected: PASS.

- [ ] **Step 5: Write the screen**

`ChallengesScreen.vue` follows `CollectionScreen.vue` line for line — `useOnActiveProfile`,
`useTabView`, `DiagnosticsList`, the `Card` with `FilterBar` as its header, the table, the empty
state through `emptyList`/`isFiltering`, `ProfileError`. The bar's props are the ones 3.10 fixed:

```vue
        <FilterBar
          :shown="rows.length"
          :total="all.length"
          :query="filter.query"
          :rows="all"
          :faceting="challengeFaceting"
          :filter="filter"
          :facets="challengeSlots"
          :state="{
            facet: ChallengeFacet.State,
            order: challengeStateOrder,
            dot: challengeStateDot,
            text: challengeStateText,
          }"
          :title="challengeFacetTitle"
          :value-label="valueLabel"
          :labels="barLabels"
          @update:query="setQuery"
          @update:picks="setPicks"
          @reset="reset"
        />
```

`challengeSlots` puts **Personaggio** in view and `Cosa sblocca` and `Bendato` behind the fold
(spec §7).

A row draws: the number and name; the state `Badge` (`Done`/`Now`/`Blocked`/`Unknown` variants,
a blocked one reading "bloccata da N"); the rewards with their icons; the goal's inline through
the Wiki's own renderer; and the buttons — open the page, and add the reward to the Plan. The
queue button is the one `UnlockTable` already draws: copy its props, its `busy` and its
"already queued" state rather than inventing a second one.

- [ ] **Step 6: Words**

Add a `challenges.*` block to both locales: `intro`, `rows`, `search`, `facet.*`,
`state.*`, the column headers, `empty`, `noResults`, and the diagnostics' four sentences. The
shared filter words are `filters.*` already and are **not** repeated (3.10).

- [ ] **Step 7: Check**

Run: `pnpm typecheck && pnpm lint && pnpm scan && pnpm ui:test`
Expected: PASS, 0 violations.

- [ ] **Step 8: Commit**

```bash
git add ui/src/screens ui/src/lib/challenges ui/src/i18n
git commit -m "feat(ui): the Challenges screen, on the filter bar"
```

---

### Task 6: the route, the sidebar, and the window

**Files:**
- Modify: `ui/src/router/routeTable.ts` (`RouteName.Challenges`, path, title, origin, icon)
- Modify: `ui/src/router/routes.ts` (the component)
- Modify: `ui/src/i18n/messages/it.ts`, `en.ts` (`routes.challenges`)

- [ ] **Step 1: Add the route**

`RouteName.Challenges = 'challenges'`, path `/progress/challenges`, title `routes.challenges`,
origin `TabOrigin.Progress`, icon `FlagIcon` (already imported — check it is not already taken by
another route; if it is, pick another Lucide icon from `@lucide/vue` and import it).

- [ ] **Step 2: Check the tables stayed exhaustive**

Run: `pnpm typecheck`
Expected: PASS. Every `Record<RouteName, …>` in `routeTable.ts` is total, so a forgotten entry is
a type error — that is the mechanism, and it is why there is nothing else to check by hand here.

- [ ] **Step 3: Look at it**

Run: `pnpm ui:dev`, open Progressi → Sfide.
Check: the five states draw; the fold holds `Cosa sblocca` and `Bendato`; a blocked row names its
gates; `?catalog=none` says the game is missing and does not read as "no challenges"; a reward
added to the Plan shows as queued.

- [ ] **Step 4: Commit**

```bash
git add ui/src/router ui/src/i18n
git commit -m "feat(ui): Sfide is the fifth entry under Progress"
```

---

### Task 7: the documents, the gate and the board

- [ ] **Step 1: Redraw `docs/architecture.md`**

Its header pins **16 crates, 30 commands, 4 events, 14 routes, 5 store migrations**. Commands
become **31** and routes **15**. Update the header's counts, add `challenges` to the command
diagram beside `collection`, and add the route to the routes diagram with the command behind it.
No crate, event or migration moved.

- [ ] **Step 2: The whole gate**

Run: `pnpm check`
Expected: all green. Paste the new test totals into `scripts/test-floor` when it prints them.

- [ ] **Step 3: `docs/STATUS.md`**

Two places. In *"What only a window can say"*, a group for 3.11 with the five lines from §9 of the
spec, and the preamble's count raised. In *"What only a machine with the game can answer"*, the
measurement §4 waits on: **a challenge with some of its gates done, checked against the game's own
challenge menu** — it decides whether `unlocked_by` is "all of" or "any of", and the save cannot.

- [ ] **Step 4: `docs/BACKLOG.md`**

Close B3 with the date and the pointers, and record what the entry got wrong: it was written
before the Collection and before the filter bar, so two of its three halves were already closed
and nobody had read it since 2026-09-05.

- [ ] **Step 5: The report**

`docs/superpowers/reports/2026-09-17-challenges-report.md`: what was built, what the work
corrected in the entry, the three measurements and which of them is *not* a confirmation, and what
a window still owes.

- [ ] **Step 6: The board**

Move B3's card to `UAT`, add the `NEEDS WINDOW` label, and update its description the way B29's
was. The card is `https://trello.com/c/of86MPLa`.

- [ ] **Step 7: Commit and finish**

```bash
git add docs scripts
git commit -m "docs: the Challenges screen lands, and what only the game's own menu can settle"
```

Then the repo's own rule for finishing: merge into `develop` with `--no-ff` and `pnpm check`
green, push, delete the branch after checking **both** conditions at the moment of deletion
(`git rev-list --count develop..feature/challenges` and
`git rev-list --count feature/challenges --not --remotes`), and **stop there** — `master` is
frozen.

---

## Self-review

**Spec coverage.** §2 → Tasks 5 and 6. §3 → Task 5 (the row, the page link, the queue button).
§4 → Task 1 (the state model, `Blocked` naming its gates) and Task 7 step 3 (the measurement it
waits on). §5 → Task 1. §6 → Task 1's degradation tests and Task 4's fixture. §7 → Task 5. §8 →
nothing to build. §9 → Task 7 step 3.

**Type consistency.** `ChallengesView`/`ChallengeRow`/`ChallengeStateView`/`RewardView` are
defined in Task 1 and used unchanged in Tasks 3, 4 and 5. The frontend reads them from the
generated `types.ts`, so a name that drifts is a type error and not a silent one.

**The one thing to watch.** `Infobox::Challenge`'s fields are the wiki crate's, and this plan
names three of them (`character`, `goal`, `blindfolded`). If their types are not what Task 1
assumes — `goal` an `Inline` tree rather than a `Vec<Inline>`, `blindfolded` an `Option<bool>` —
follow the crate and change the view-model to match, rather than converting at the boundary: the
wiki's types already cross the IPC, and a second shape for the same thing is the defect 3.10 spent
a sub-project removing.
