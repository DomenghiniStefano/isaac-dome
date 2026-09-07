# IPC contracts for the graph screens — Unlock, Next steps, Plan (design)

**Date:** 2026-09-05
**Sub-project:** delivery to design, step 2 (`docs/STATO.md`, "Delivery to design")
**Status:** design approved verbally; pending spec review

## Context

Three of the seven screens — Unlock, Next steps, Plan — depend on the unlock graph (M2)
and the derived plan (M3), neither of which exist. The 2026-09-04 decision is to **fix
the shape of their view-models now**, so design works against stable contracts and M2
proceeds in parallel without touching the types the frontend consumes.

The rule that makes this possible is "Vue only receives already-resolved JSON": the only
dependency between the webapp and the backend is the shape of the types in `ipc`. This
spec fixes that shape, and decides **what's real and what's a placeholder**: data that
already exists (catalog, save) arrives real; the graph arrives as a **typed, declared**
placeholder, never as a value that looks real.

It also carries the three preliminary decisions that came out of reviewing `catalog`'s
plan B, all made against real data on 2026-09-05.

## Applicable non-negotiable constraints

1. **Read-only on saves.** The write commands introduced here (the goals) write to the
   app's settings file, never to the `.dat`. It's a property by construction:
   `core-save` has no write functions.
2. **No hardcoded count.** How many achievements, how many slots: the files say. The
   numbers in real-data tests are fixtures from a known era (Repentance+, profile
   `live.rep+…1.dat` from 2026-08-31), with the date in the comment.
3. **Degrade, never fail.** A save slot with no matching achievement in the catalog
   isn't an error: it's an `unknown` node. A missing catalog produces an empty Unlock
   with a diagnostic, not an `Err`.
4. **IPC boundary.** Every struct `camelCase`; enums tagged with `kind` using struct
   variants; no paths, offsets, or raw bytes; JSON shape pinned by tests, including the
   `unknown` and `stub` cases.
5. **Never fake a value.** No field with a value that looks computed but isn't. What
   the graph doesn't know yet travels as `{ kind: "stub" }`, and facets that depend on
   an interpretation that doesn't exist yet **don't enter** the contract.

## The three preliminary decisions (verified against real data)

### 1. Achievement ↔ section-1 slot linkage: `slot[i] = achievement i`

`achievements.xml` has 637 achievements with contiguous ids 1..=637; the save's section
1 has 642 slots (1 byte each). Verified on 2026-09-05 against the real profile:

| hypothesis | items *seen* (sec. 4) with `unlocked_by` whose achievement turns out *done* |
|---|---|
| `slot[id − 1]` | 145 out of 171 |
| **`slot[id]`** | **169 out of 171** |
| `slot[id + 1]` | 142 out of 171 |

Slot 0 is unused (`false`). The two disagreements with `slot[id]` are items that can be
*seen* without having their unlock (D8 in a challenge; Eternal D6): "seen" in section 4
doesn't imply "unlocked", and these don't contradict the alignment.

Slots **638–641** lie beyond the file's last achievement: they're achievements added by
a catalog patch newer than this one. In this profile three of them are `true`. The
contract exposes them as `{ kind: "unknown", slot }`: *done, but the catalog doesn't
know what it is*. This is the brief's "known index, unknown name" case, and now it has a
number.

### 2. Reverse index `AchievementId → what it unlocks`

Today `catalog` only carries the item/character/boss/challenge → achievement direction
(`unlocked_by`). The graph and Unlock need the inverse, and building it 637 times via a
linear scan is the wrong way to do it. It's built **once in `Catalog::build`**:

```rust
pub enum Unlock {
    Item { kind: ItemKind, id: ItemId },
    Character { id: CharacterId },
    Boss { id: BossId },
    Challenge { id: ChallengeId },
}
impl Catalog {
    /// What an achievement unlocks. Empty if nothing references it: that's data, not an error.
    pub fn unlocks(&self, id: AchievementId) -> &[Unlock];
}
```

Deterministic order (by variant, then by id). Real-data test: the sum of edges matches
that of the files (370 items + 40 characters + 27 bosses + the challenges'
achievements, counting the lists), and every `unlocked_by` in the catalog appears
exactly once in the index.

### 3. `unlock_condition` is text, and `None` is normal

283 achievements out of 637 have the condition in the XML comment; 354 don't. In the
contract the condition is called **`hint`**, is `string | null`, and is **displayable
English text**: it's not structured data, nothing is derived from it, and `null` is the
normal state for more than half the rows — not missing data that needs flagging to the
user.

## The contracts

### The node: one for three screens

Next steps and Plan are **selections** of Unlock, not different types. A single node:

```rust
/// An achievement as the interface sees it: what it is, whether it's done, what it
/// unlocks, and what the graph can say about it (today: nothing).
#[serde(rename_all = "camelCase")]
pub struct UnlockNode {
    pub achievement: AchievementRef,
    /// From the save, section 1: REAL.
    pub done: bool,
    /// From the catalog (reverse index): REAL. Empty if the achievement unlocks nothing known.
    pub unlocks: Vec<UnlockTarget>,
    /// Origin DLC of the first item unlocked, if any: REAL.
    pub origin: Option<Origin>,
    /// What the graph knows. Until M2 exists: `Stub`.
    pub graph: GraphInfo,
}

#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum AchievementRef {
    /// In the catalog: name, text, icon.
    Known { id: u32, text: String, hint: Option<String>, icon_url: Option<String> },
    /// In the save but not in the catalog (a newer patch of the file): only the slot.
    Unknown { slot: u32 },
}

#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum UnlockTarget {
    Item { kind: ItemKindView, id: u32, name: String, icon_url: Option<String> },
    Character { id: u32, name: String },
    Boss { id: u32, name: String },
    Challenge { id: u32, name: String },
}

#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum GraphInfo {
    /// The graph doesn't exist: design draws it as a state, not as data.
    Stub,
    /// M2: computed.
    Computed { available_now: bool, blocked_by: u32, fan_out: u32, steps_missing: u32 },
}

#[serde(rename_all = "camelCase")]
pub enum Origin { Rebirth, Afterbirth, AfterbirthPlus, Repentance }
```

`icon_url` is an already-resolved `data:` URL, or `None` when the sprite doesn't
extract: the row still shows. The `origin` field comes from `catalog::Origin`
(redefined in `ipc` so a domain type doesn't cross the IPC: `ipc` already has
`ItemKindView` for the same reason).

**What doesn't go in, and why.** The Unlock facets from §07 of `PROJECT.md` that depend
on the graph (`unlockable now`, `blocked by N`, fan-out, missing steps) live in
`GraphInfo::Computed` and arrive with M2. The ones that depend on an **interpretation**
of the 283 English conditions (required ending, mode, shape of the effort) and Steam
rarity (network) **have no field**: making them permanent `null`s would be faking data.
The contract will grow when someone exists to compute them, by adding fields — not
changing them.

### Unlock

```rust
#[serde(rename_all = "camelCase")]
pub struct UnlockView {
    pub nodes: Vec<UnlockNode>,        // one per slot 1..=N of section 1
    pub totals: UnlockTotals,
    pub diagnostics: Vec<UnlockDiagnostic>,
}
#[serde(rename_all = "camelCase")]
pub struct UnlockTotals { pub slots: u32, pub done: u32, pub known: u32, pub unknown: u32 }

#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum UnlockDiagnostic {
    /// The save declares more slots than achievements the catalog knows about.
    SlotsBeyondCatalog { count: u32 },
    /// The catalog knows achievements beyond the save's slots (an older edition).
    CatalogBeyondSlots { count: u32 },
    NoCatalog,
}
```

Built by a pure function `unlock_view(&Catalog, flags: &[bool], icon: impl FnMut(&str)
-> Option<Vec<u8>>) -> UnlockView` in `ipc`: `flags` is section 1, already read.
Filtering and sorting live in the frontend (TanStack Table): the contract carries the
data, not the interface's state.

### Next steps

```rust
#[serde(rename_all = "camelCase")]
pub struct NextSteps {
    pub steps: Vec<UnlockNode>,   // at most STEPS, not done
    pub basis: StepsBasis,
}
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum StepsBasis {
    /// Without a graph: the first not-done ones in slot order. Declared, not hidden.
    Stub,
    /// M2: ordered by fan-out.
    FanOut,
}
```

`STEPS = 5` is a presentation constant in `ipc`, not a domain count: the project
document says "the 5 things worth doing right now".

### Plan: materialized goals

Decision from 2026-09-05: goals are **entities the user creates and saves**, not a
view. They keep few ("a couple"), but **no hardcoded limit**.

```rust
#[serde(rename_all = "camelCase")]
pub struct Goal {
    pub id: GoalId,               // newtype over String, opaque, app-generated
    pub target: UnlockTarget,     // what I want: an item, a character, a boss, a challenge
    pub created_unix: i64,
    pub note: Option<String>,
}
#[serde(rename_all = "camelCase")]
pub struct PlanView {
    pub goals: Vec<Goal>,
    pub expansion: PlanExpansion,
}
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum PlanExpansion {
    /// M3 doesn't exist: goals are visible, the steps aren't.
    Stub,
    /// M3: every goal expanded into its prerequisites, grouped by run.
    Computed { steps: Vec<PlanStep> },
}
#[serde(rename_all = "camelCase")]
pub struct PlanStep { pub goal: GoalId, pub node: UnlockNode, pub done: bool }
```

**Persistence: `store` is born.** Decision from 2026-09-05, revised from the first
draft (which put goals in `settings.json`). The project document assigns to `store`
(SQLite, a single file in the data folder) four things: run archive, profile snapshots,
normalized catalog, **the user's plans**. Goals *are* the user's plans; having them born
in a settings file with a migration promised "some day" is the structural debt not to
take on. So `store` is born now, small:

- crate `crates/store`, `rusqlite` (bundled, as per the stack), **a single file**
  `isaacdome.db` in the app's data folder; `settings.json` stays for the active
  profile, which is a setting, not data;
- **versioned schema** with `PRAGMA user_version` and sequential migrations: migration 1
  creates `goals (id TEXT PRIMARY KEY, target_json TEXT NOT NULL, created_unix INTEGER
  NOT NULL, note TEXT)`. Run and snapshot tables will come as migrations 2+, without
  breaking the file of anyone who already has the app; **no empty tables ahead of time**
  (YAGNI);
- minimal, typed API: `Store::open(path) -> Result<Store, StoreError>`, `goals() ->
  Result<Vec<Goal>>`, `add_goal(&Goal)`, `remove_goal(&GoalId)`; `Store` stays in Rust,
  the frontend never touches the disk;
- **read-only on saves stays true by construction**: `store` only writes to its own
  file, and `core-save` has no write functions.

`target` is saved as the JSON of the `UnlockTarget` (it's already a serializable type,
and a column per variant would mean a schema that changes with every new unlock type).
Goals are **per installation, not per profile**: a goal like "I want Tainted Lost" holds
for whoever plays on that machine, and the Plan will show, profile by profile, what's
missing.

**The second migration is already decided, not yet written.** The game creates
`save_backups\` on its own (dated backups of the profile: 28 on this machine) and
`online_logs\` (snapshots before and after each co-op session): this is the free
historical series "the plan that updates itself" rests on. `store`'s migration 2 will be
the **snapshots** table (sections 1 and 4 of a `.dat`, with date and origin), imported
from there. It's mentioned here so the structure anticipates it; the code arrives with
M3.

**Commands** (in `app`, wiring): `plan() -> PlanView`, `add_goal(target: UnlockTarget) ->
PlanView`, `remove_goal(id: GoalId) -> PlanView`. These are the app's first write
commands. `Store` lives in a managed Tauri state, opened once; a database that fails to
open (permissions, corrupted file) degrades: `plan()` responds with `goals: []` and a
`StoreUnavailable` diagnostic, while `add_goal` returns
`IpcError::StoreUnavailable { reason }`. `add_goal` rejects a `target` the catalog
doesn't know with `IpcError::UnknownTarget`.

### The TypeScript mirror

`ui/src/lib/ipc/types.ts` mirrors every type (discriminated unions on `kind`), and the
wrappers in `ui/src/lib/ipc/graph.ts` expose `unlock()`, `nextSteps()`, `plan()`,
`addGoal()`, `removeGoal()`. No components: the verification screen only shows Unlock's
totals and the first five steps, to check the channel works.

## What's real and what's a stub, row by row

| field | source | state |
|---|---|---|
| `achievement.known.{id, text, hint, iconUrl}` | catalog | ✅ real |
| `achievement.unknown.slot` | save, beyond the catalog | ✅ real |
| `done` | save, section 1, `slot[id]` | ✅ real |
| `unlocks[]` | catalog's reverse index | ✅ real |
| `origin` | `catalog::Origin` of the first item unlocked | ✅ real |
| `graph` | — | 🔲 `Stub` |
| `NextSteps.steps` | the first 5 not-done, in slot order | ✅ real, `basis: stub` |
| `PlanView.goals` | `store` (SQLite, `isaacdome.db`) | ✅ real |
| `PlanView.expansion` | — | 🔲 `Stub` |

When M2 arrives, only the **values** of `graph`, `basis`, and `expansion` change: no
type changes, no component breaks.

## Boundaries and what changes in existing code

- **`catalog`**: only the reverse index (`Unlock`, `unlocks()`), built in `build` after
  all sources are read. No other change.
- **`store`** (new crate): `Store`, `Goal`, `GoalId`, `StoreError`, migration 1. Depends
  on `rusqlite` and `serde`; the `Goal`/`GoalId`/`UnlockTarget` types it shares with
  `ipc` live in `ipc`, and `store` imports them (`ipc` doesn't depend on `store`: the
  view-model doesn't know where the data lives).
- **`ipc`**: new `graph.rs` module (the types and the pure functions `unlock_view`,
  `next_steps`, `plan_view(goals: &[Goal], store_ok: bool)`), `Goal`, `GoalId`;
  `IpcError` in `app` gains `UnknownTarget` and `StoreUnavailable { reason }`. The
  existing `data_url` is reused for the icons.
- **`app`**: the five commands; `Store` opened once in a managed state (like the
  catalog); the save is read with `active_save` as today and section 1 with
  `save.flags(Kind::Achievements)`; the catalog from the managed state.
- **UI**: types and wrappers; in the verification screen a section "Unlock: N done out
  of M, K unknown" and the five steps.
- **Documents**: `DESIGN-BRIEF.md` §4 with Unlock, Next steps, and Plan
  "designable against the contract", and a new §6 carrying the types with the
  real/stub table above.

## Tests

### Unit tests (pure, hand-written fragments)

- `unlock_view`: a catalog with 3 achievements and `flags` for 6 slots → 5 nodes (slots
  1..=5): three `known` with `done` from the right slot, two `unknown`,
  `SlotsBeyondCatalog { 2 }`; with `flags` for 3 slots and 5 achievements →
  `CatalogBeyondSlots { 2 }`; without a catalog → empty nodes and `NoCatalog`.
- A node's `unlocks[]`: an achievement referenced by an item and by a character → two
  targets, in the declared order; one referenced by nothing → empty.
- `origin` = that of the first `Item` in `unlocks`, `None` if the first target isn't an
  item.
- `next_steps`: 7 nodes, 3 of which done → the first 4 not-done in slot order, `basis:
  Stub`; with `STEPS` not-done nodes → exactly `STEPS`.
- `plan_view`: goals from `Settings` in creation order; `expansion: Stub`.
- **JSON shape** pinned for every enum: `{"kind":"known",…}`,
  `{"kind":"unknown","slot":640}`, `{"kind":"stub"}`,
  `{"kind":"computed","availableNow":…}`, `{"kind":"item","kind"…}` — note:
  `UnlockTarget::Item` has a field `kind: ItemKindView` that collides with the `kind`
  tag: **the field is named `item_kind`** (`itemKind` in the JSON). A test pins this,
  because it's exactly the kind of silent error the boundary produces.
- `store`: on a temp file, `open` creates the schema at `user_version = 1`; `add_goal` →
  `goals()` returns it; `remove_goal` removes it; reopening the same file finds the
  goals and **doesn't** re-run the migration; a file with a `user_version` higher than
  supported → `StoreError::NewerSchema { found, supported }` (the app degrades, it
  doesn't destroy); a non-SQLite file → `StoreError::Unreadable`.
- `Settings` doesn't change: goals don't fit there (non-regression test: the existing
  file deserializes identically).

### On real data (skipped with a note if `samples/` is missing)

- On profile `live.rep+persistentgamedata1.dat` (2026-08-31): 642 slots, **379 done**,
  637 `known`, 4 `unknown` (slots 638-641: slot 0 isn't a node), of which **3 done**
  (slots 638, 639, 641), `SlotsBeyondCatalog { 4 }`.
- Linkage: among the items seen in section 4 with `unlocked_by`, **169 out of 171** have
  the `done` node — the fixture that pins `slot[id]`. If it ever drops, it's the
  alignment that broke, not a number to adjust.
- Reverse index: sum of edges equal to the links in the files; no `unlocked_by` lost.
- `next_steps` on the profile: 5 nodes, all `done: false`, in increasing slot order.

## Unknowns, and what happens if they don't hold

1. **Slots 638–641 might not be "new" achievements but padding.** The hypothesis holds
   because three of them are `true` in a played profile and the count declared by the
   file grew by one between June 2025 and 2026 (`CLAUDE.md`). If a more recent catalog
   ever names them, they become `known` on their own: the contract doesn't change.
2. **`unlocked_by` on items might not be the only way an achievement unlocks something**
   (rooms, pickups, cards from §07, for instance). Those categories aren't in the files
   we read: `unlocks[]` is empty for those achievements, and that's an honest fact. The
   `UnlockTarget` enum will grow by addition.

## Out of scope

The graph (M2) and the derived plan (M3); interpretation of the conditions; Steam
rarity; `store`'s migrations 2+ (snapshots, runs) and the import from `save_backups\`;
the components of the three screens (they arrive with the design system); filtering and
sorting (frontend state).

## Completion criteria

- `cargo test --workspace`, `cargo fmt --check`, `cargo clippy --all-targets -- -D
  warnings`, `pnpm typecheck`, `pnpm lint`, `pnpm scan`, `pnpm format:check` clean.
- The verification screen shows "Unlock: 379 done out of 642, 4 unknown" and five steps
  with name and icon, on this profile.
- A goal that's added survives an app restart, in `isaacdome.db`; `settings.json`
  contains no goals.
- `store` exists with migration 1 and the tests above; `docs/STATO.md` records it as a
  module born with this step, not with M4.
- `docs/STATO.md`: step 2 of the delivery checked off with the three decisions;
  `DESIGN-BRIEF.md` with the contracts and the real/stub table.
