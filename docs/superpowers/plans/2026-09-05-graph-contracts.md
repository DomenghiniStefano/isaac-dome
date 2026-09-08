# Graph screens' IPC contracts — implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** the view-models for Unlock, Next Steps, and Plan exist in `ipc` with a TypeScript mirror, pinned by tests on the JSON shape; behind them sit real data (catalog, save, goals in `store`) and a typed placeholder where the graph is needed; `store` is born with its first migration.

**Architecture:** `catalog` gains the reverse index `unlocks(AchievementId)`; the `store` crate is born (rusqlite, one file, versioned schema); `ipc` gets a pure `graph.rs` module with the types and three functions (`unlock_view`, `next_steps`, `plan_view`); `app` wires up five commands with `Store` in managed state; the UI receives types and a wrapper, and the verification screen shows the Unlock totals and the five steps.

**Tech Stack:** Rust 2021, `rusqlite = "0.40"` with the `bundled` feature, `serde`, `serde_json`; Tauri 2 managed state; Vue 3 / TypeScript.

**Spec:** `docs/superpowers/specs/2026-09-05-graph-contracts-design.md`

## Global Constraints

- **Read-only on saves**: no task touches `core-save` in write mode; `store` only writes to `isaacdome.db`.
- No hardcoded counts; real tests pin numbers as fixtures of a known era (profile `live.rep+persistentgamedata1.dat` from 2026-08-31, Repentance+ catalog from 2026-09-03) with the date in the comment. **If a real test fails, the code is the first suspect**: investigate and report, don't adjust the number.
- Degrade, never fail: no `panic!`/`unwrap()`/`expect()` outside tests on data read from disk.
- IPC boundary: every struct `#[serde(rename_all = "camelCase")]`; tagged enums `#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]` with struct variants, never newtype; never `format!("{:?}")` across the boundary; JSON shape pinned by a test for every enum, including `unknown` and `stub`.
- **Never fake a value**: whatever the graph doesn't know travels as `{ kind: "stub" }`.
- Exhaustiveness: no `_ =>` on our own enums. No new `#[allow]`.
- Before declaring a task done: `cargo test --workspace`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`; for UI tasks also `pnpm typecheck`, `pnpm lint`, `pnpm scan`, `pnpm format:check` from the root.
- Commits: prefix `catalog:` / `store:` / `ipc:` / `app:` / `ui:` / `docs:`, messages in English, **no** `Co-Authored-By` trailer or references to Claude. No pushing from tasks.

---

## File structure

```
crates/catalog/src/catalog.rs        + unlocks: BTreeMap<AchievementId, Vec<Unlock>>, unlocks(id), build_unlocks()
crates/catalog/src/unlock.rs         NEW: enum Unlock
crates/catalog/tests/build.rs        + unit tests for the index
crates/catalog/tests/real_data.rs    + real test for the index

crates/store/Cargo.toml              NEW crate
crates/store/src/lib.rs              Store, StoreError, re-export of Goal/GoalId from ipc
crates/store/src/migrations.rs       MIGRATIONS, apply()
crates/store/tests/goals.rs          integration test on a temp file

crates/ipc/src/graph.rs              NEW: contract types + unlock_view, next_steps, plan_view, STEPS
crates/ipc/src/goals.rs              NEW: Goal, GoalId (shared with store)
crates/ipc/tests/graph.rs            unit + JSON shape
crates/ipc/tests/graph_real.rs       on the real profile + real catalog

crates/app/src/error.rs              + UnknownTarget, StoreUnavailable { reason }
crates/app/src/lib.rs                + StoreState, five commands
ui/src/lib/ipc/types.ts, ui/src/lib/ipc/graph.ts, ui/src/lib/constants/commands.ts, ui/src/App.vue
docs/STATUS.md, DESIGN-BRIEF.md, docs/superpowers/plans/2026-09-05-graph-contracts-report.md
```

---

### Task 1: `catalog` — the reverse index `unlocks(AchievementId)`

**Files:**
- Create: `crates/catalog/src/unlock.rs`
- Modify: `crates/catalog/src/catalog.rs`, `crates/catalog/src/lib.rs`
- Test: `crates/catalog/tests/build.rs`, `crates/catalog/tests/real_data.rs`

**Interfaces:**
- Produces: `catalog::Unlock` (`enum Unlock { Item { kind: ItemKind, id: ItemId }, Character { id: CharacterId }, Boss { id: BossId }, Challenge { id: ChallengeId } }`, `Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord`), `Catalog::unlocks(&self, id: AchievementId) -> &[Unlock]`.

- [ ] **Step 1: the unit tests** — appended to `crates/catalog/tests/build.rs`:

```rust
use catalog::Unlock;

#[test]
fn unlocks_index_inverts_every_link_in_a_deterministic_order() {
    let items: &[u8] = b"<items gfxroot=\"gfx/items/\"><passive id=\"2\" gfx=\"a.png\" name=\"A\" achievement=\"7\" /><trinket id=\"1\" gfx=\"t.png\" name=\"T\" achievement=\"7\" /><active id=\"9\" gfx=\"b.png\" name=\"B\" achievement=\"8\" /></items>";
    let players: &[u8] = b"<players portraitroot=\"gfx/ui/stage/\"><player id=\"3\" name=\"#X_NAME\" portrait=\"x.png\" achievement=\"7\" /></players>";
    let bosses: &[u8] = b"<bosses root=\"gfx/ui/boss/\"><boss id=\"5\" name=\"Y\" portrait=\"y.png\" achievement=\"8\" /></bosses>";
    let challenges: &[u8] = b"<challenges><challenge name=\"C\" id=\"4\" achievements=\"7,8\" /></challenges>";
    let c = Catalog::build(|p| match p {
        "items.xml" => Some(items.to_vec()),
        "players.xml" => Some(players.to_vec()),
        "bossportraits.xml" => Some(bosses.to_vec()),
        "challenges.xml" => Some(challenges.to_vec()),
        _ => None,
    });
    // Order: by variant (Item < Character < Boss < Challenge), then by (kind, id).
    assert_eq!(
        c.unlocks(AchievementId(7)),
        &[
            Unlock::Item { kind: ItemKind::Passive, id: ItemId(2) },
            Unlock::Item { kind: ItemKind::Trinket, id: ItemId(1) },
            Unlock::Character { id: CharacterId(3) },
            Unlock::Challenge { id: ChallengeId(4) },
        ]
    );
    assert_eq!(
        c.unlocks(AchievementId(8)),
        &[Unlock::Item { kind: ItemKind::Active, id: ItemId(9) }, Unlock::Boss { id: BossId(5) }, Unlock::Challenge { id: ChallengeId(4) }]
    );
    assert!(c.unlocks(AchievementId(99)).is_empty(), "an achievement that unlocks nothing: empty, not an error");
}
```

Add `BossId, ChallengeId, CharacterId` to the existing `catalog::{...}` import at the top of the file (without duplicating the `use` line).

- [ ] **Step 2: red** — Run: `cargo test -p catalog --test build unlocks_index` → FAIL (`Unlock` doesn't exist).

- [ ] **Step 3: `unlock.rs`**

```rust
//! What an achievement unlocks: the inverse of the `unlocked_by` links scattered
//! across the files. Built once in `Catalog::build`; the graph (M2) and Unlock read it.

use crate::ids::{AchievementId, BossId, ChallengeId, CharacterId, ItemId};
use crate::items::ItemKind;

/// An edge achievement -> what it unlocks. The derived order (variant, then id) makes
/// the index deterministic without a hand-written comparator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Unlock {
    Item { kind: ItemKind, id: ItemId },
    Character { id: CharacterId },
    Boss { id: BossId },
    Challenge { id: ChallengeId },
}

/// Marks the type for readers; the actual work is done by `Catalog::build`.
pub(crate) type Index = std::collections::BTreeMap<AchievementId, Vec<Unlock>>;
```

- [ ] **Step 4: `catalog.rs`** — field `unlocks: unlock::Index`, and in `build`, right before `diagnose_unresolved_keys(...)`:

```rust
        let unlocks = build_unlocks(&items, &characters, &bosses, &challenges);
```

the function, at the bottom of the file:

```rust
/// The inverse index: for each achievement, what it unlocks. One pass over the four
/// files that carry `unlocked_by`; each vector is sorted, so the order doesn't depend
/// on reading order.
fn build_unlocks(
    items: &BTreeMap<(ItemKind, ItemId), Item>,
    characters: &BTreeMap<CharacterId, Character>,
    bosses: &BTreeMap<BossId, Boss>,
    challenges: &BTreeMap<ChallengeId, Challenge>,
) -> unlock::Index {
    let mut index: unlock::Index = BTreeMap::new();
    let mut push = |a: AchievementId, u: Unlock| index.entry(a).or_default().push(u);
    for ((kind, id), item) in items {
        if let Some(a) = item.unlocked_by {
            push(a, Unlock::Item { kind: *kind, id: *id });
        }
    }
    for (id, c) in characters {
        if let Some(a) = c.unlocked_by {
            push(a, Unlock::Character { id: *id });
        }
    }
    for (id, b) in bosses {
        if let Some(a) = b.unlocked_by {
            push(a, Unlock::Boss { id: *id });
        }
    }
    for (id, ch) in challenges {
        for a in &ch.unlocked_by {
            push(*a, Unlock::Challenge { id: *id });
        }
    }
    for v in index.values_mut() {
        v.sort();
    }
    index
}
```

and the accessor, next to `achievement`:

```rust
    /// What an achievement unlocks. Empty if nothing cites it: that's data, not an error.
    pub fn unlocks(&self, id: AchievementId) -> &[Unlock] {
        self.unlocks.get(&id).map(Vec::as_slice).unwrap_or(&[])
    }
```

`lib.rs`: `mod unlock;` and `pub use unlock::Unlock;`. Import in `catalog.rs`: `use crate::unlock::{self, Unlock};` plus `Boss`, `Challenge`, `BossId`, `ChallengeId` if not already present.

- [ ] **Step 5: real test** — appended to `crates/catalog/tests/real_data.rs`:

```rust
#[test]
fn the_unlock_index_carries_every_link_exactly_once() {
    let Some((c, _)) = build_or_skip() else { return };
    let from_files = c.items().filter(|i| i.unlocked_by.is_some()).count()
        + c.characters().filter(|ch| ch.unlocked_by.is_some()).count()
        + c.bosses().filter(|b| b.unlocked_by.is_some()).count()
        + c.challenges().map(|ch| ch.unlocked_by.len()).sum::<usize>();
    let in_index: usize = c.achievements().map(|a| c.unlocks(a.id).len()).sum();
    // 370 items + 40 characters + 27 bosses + the challenge lists (2026-09-03).
    assert_eq!(in_index, from_files);
    assert!(from_files > 400, "the links number in the hundreds: {from_files}");
    // Achievement 1 unlocks Magdalene (players.xml: id 1, achievement="1").
    assert_eq!(c.unlocks(AchievementId(1)), &[Unlock::Character { id: CharacterId(1) }]);
}
```

Note: `in_index` only counts achievements **in the catalog**; an `unlocked_by` pointing to an id outside 1..=637 would end up in the index but not in the sum. If `assert_eq!` fails by one or two, that's why: report it, don't adjust it.

- [ ] **Step 6: green and commit**

Run: `cargo test -p catalog && cargo fmt --check && cargo clippy --all-targets -- -D warnings`

```bash
git add crates/catalog
git commit -m "catalog: reverse index achievement -> what it unlocks, built once"
```

---

### Task 2: `ipc::goals` — `Goal` and `GoalId`

**Files:**
- Create: `crates/ipc/src/goals.rs`
- Modify: `crates/ipc/src/lib.rs`, `crates/ipc/Cargo.toml` (+ `serde_json` in `[dependencies]` if it's not there: needed by `store`, not here — check)
- Test: `crates/ipc/tests/goals.rs`

**Interfaces:**
- Produces: `ipc::GoalId(String)` opaque newtype (`Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug`, `#[serde(transparent)]`, `GoalId::new() -> GoalId` random, `as_str()`), `ipc::Goal { id: GoalId, target: UnlockTarget, created_unix: i64, note: Option<String> }`, `ipc::UnlockTarget` (defined here because `Goal` carries it; `graph.rs` re-exports it).

- [ ] **Step 1: the tests** — `crates/ipc/tests/goals.rs`:

```rust
use ipc::{Goal, GoalId, ItemKindView, UnlockTarget};

#[test]
fn goal_json_shape_is_pinned() {
    let g = Goal {
        id: GoalId::from_str_unchecked("g1"),
        target: UnlockTarget::Item { item_kind: ItemKindView::Passive, id: 555, name: "Golden Razor".into(), icon_url: None },
        created_unix: 1_700_000_000,
        note: Some("stasera".into()),
    };
    let v = serde_json::to_value(&g).unwrap();
    assert_eq!(v["id"], "g1");
    assert_eq!(v["target"]["kind"], "item");
    // `ItemKindView` is already a tagged enum (`{"kind":"passive"}`): the field is called
    // `itemKind` because `kind` is UnlockTarget's tag, and its value is an object.
    assert_eq!(v["target"]["itemKind"]["kind"], "passive");
    assert_eq!(v["target"]["id"], 555);
    assert_eq!(v["target"]["iconUrl"], serde_json::Value::Null);
    assert_eq!(v["createdUnix"], 1_700_000_000);
    assert_eq!(v["note"], "stasera");
    let back: Goal = serde_json::from_value(v).unwrap();
    assert_eq!(back, g);
}

#[test]
fn every_target_variant_round_trips_with_its_tag() {
    for (t, tag) in [
        (UnlockTarget::Character { id: 21, name: "T. Isaac".into() }, "character"),
        (UnlockTarget::Boss { id: 100, name: "The Beast".into() }, "boss"),
        (UnlockTarget::Challenge { id: 44, name: "Red Redemption".into() }, "challenge"),
    ] {
        let v = serde_json::to_value(&t).unwrap();
        assert_eq!(v["kind"], tag);
        assert_eq!(serde_json::from_value::<UnlockTarget>(v).unwrap(), t);
    }
}

#[test]
fn fresh_goal_ids_are_distinct_and_opaque() {
    let a = GoalId::new();
    let b = GoalId::new();
    assert_ne!(a, b);
    assert!(a.as_str().len() >= 16);
}
```

- [ ] **Step 2: red** — Run: `cargo test -p ipc --test goals` → FAIL.

- [ ] **Step 3: `goals.rs`**

```rust
//! The user's goals: what they want to unlock. The type lives here because it crosses
//! the IPC; where it gets saved (`store`) isn't the view-model's concern.

use serde::{Deserialize, Serialize};

use crate::catalog_view::ItemKindView;

/// Opaque id, generated by the app. Newtype: it's not a number, it's not a name, and
/// the frontend never constructs one.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GoalId(String);

impl GoalId {
    /// A fresh id: 128 random bits in hex. No dependency needed: `std::time` and the
    /// address of an allocation are enough entropy to distinguish goals on the same
    /// machine.
    pub fn new() -> GoalId {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        std::time::SystemTime::now().hash(&mut h);
        let probe = Box::new(0u8);
        (&*probe as *const u8 as usize).hash(&mut h);
        let a = h.finish();
        std::process::id().hash(&mut h);
        let b = h.finish();
        GoalId(format!("{a:016x}{b:016x}"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Only for tests and for `store`, which reads back ids already generated.
    pub fn from_str_unchecked(s: &str) -> GoalId {
        GoalId(s.to_string())
    }
}

impl Default for GoalId {
    fn default() -> Self {
        GoalId::new()
    }
}

/// What an achievement unlocks, or what a goal wants. The field for the item's type is
/// called `item_kind` because `kind` is the enum's tag: calling it `kind` would produce
/// JSON with two identical keys, and serde would accept that silently.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum UnlockTarget {
    Item { item_kind: ItemKindView, id: u32, name: String, icon_url: Option<String> },
    Character { id: u32, name: String },
    Boss { id: u32, name: String },
    Challenge { id: u32, name: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Goal {
    pub id: GoalId,
    pub target: UnlockTarget,
    pub created_unix: i64,
    pub note: Option<String>,
}
```

`ItemKindView` must also derive `Deserialize`, `Hash`: add them in `catalog_view.rs` (today it has `Debug, Clone, Copy, PartialEq, Eq, Serialize`). `lib.rs`: `mod goals;` and `pub use goals::{Goal, GoalId, UnlockTarget};`.

- [ ] **Step 4: green and commit**

```bash
git add crates/ipc
git commit -m "ipc: Goal, opaque GoalId and UnlockTarget, with the JSON shape pinned"
```

---

### Task 3: the `store` crate

**Files:**
- Create: `crates/store/Cargo.toml`, `crates/store/src/lib.rs`, `crates/store/src/migrations.rs`, `crates/store/tests/goals.rs`
- Modify: root `Cargo.toml` (the `members = ["crates/*"]` already includes it on its own: no change)

**Interfaces:**
- Produces: `store::Store` (`open(path: &Path) -> Result<Store, StoreError>`, `goals(&self) -> Result<Vec<Goal>, StoreError>`, `add_goal(&self, g: &Goal) -> Result<(), StoreError>`, `remove_goal(&self, id: &GoalId) -> Result<bool, StoreError>`), `store::StoreError` (`Unreadable { reason: String }`, `NewerSchema { found: u32, supported: u32 }`, `Corrupt { reason: String }`), `store::SCHEMA_VERSION: u32 = 1`.

- [ ] **Step 1: manifest**

```toml
[package]
name = "store"
version = "0.1.0"
edition = "2021"
description = "The app's persistence: one SQLite file, versioned schema. It writes only to its own file."

[dependencies]
rusqlite = { version = "0.40", features = ["bundled"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
ipc = { path = "../ipc" }

[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 2: the tests** — `crates/store/tests/goals.rs`:

```rust
use ipc::{Goal, GoalId, ItemKindView, UnlockTarget};
use store::{Store, StoreError, SCHEMA_VERSION};

fn goal(id: &str, name: &str) -> Goal {
    Goal {
        id: GoalId::from_str_unchecked(id),
        target: UnlockTarget::Item { item_kind: ItemKindView::Passive, id: 1, name: name.into(), icon_url: None },
        created_unix: 1_700_000_000,
        note: None,
    }
}

#[test]
fn open_creates_the_schema_at_the_current_version() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("isaacdome.db");
    let s = Store::open(&path).unwrap();
    assert_eq!(s.schema_version().unwrap(), SCHEMA_VERSION);
    assert!(s.goals().unwrap().is_empty());
}

#[test]
fn goals_survive_reopening_and_the_migration_does_not_rerun() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("isaacdome.db");
    {
        let s = Store::open(&path).unwrap();
        s.add_goal(&goal("a", "Sad Onion")).unwrap();
        s.add_goal(&goal("b", "Inner Eye")).unwrap();
    }
    let s = Store::open(&path).unwrap();
    let names: Vec<String> = s.goals().unwrap().iter().map(|g| match &g.target {
        UnlockTarget::Item { name, .. } => name.clone(),
        UnlockTarget::Character { name, .. } | UnlockTarget::Boss { name, .. } | UnlockTarget::Challenge { name, .. } => name.clone(),
    }).collect();
    assert_eq!(names, vec!["Sad Onion", "Inner Eye"], "in insertion order");
    assert_eq!(s.schema_version().unwrap(), SCHEMA_VERSION);
}

#[test]
fn remove_goal_reports_whether_it_existed() {
    let dir = tempfile::tempdir().unwrap();
    let s = Store::open(&dir.path().join("x.db")).unwrap();
    s.add_goal(&goal("a", "A")).unwrap();
    assert!(s.remove_goal(&GoalId::from_str_unchecked("a")).unwrap());
    assert!(!s.remove_goal(&GoalId::from_str_unchecked("a")).unwrap());
    assert!(s.goals().unwrap().is_empty());
}

#[test]
fn adding_the_same_id_twice_replaces_not_duplicates() {
    let dir = tempfile::tempdir().unwrap();
    let s = Store::open(&dir.path().join("x.db")).unwrap();
    s.add_goal(&goal("a", "prima")).unwrap();
    s.add_goal(&goal("a", "dopo")).unwrap();
    let all = s.goals().unwrap();
    assert_eq!(all.len(), 1);
    assert!(matches!(&all[0].target, UnlockTarget::Item { name, .. } if name == "dopo"));
}

#[test]
fn a_newer_schema_is_refused_not_destroyed() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("x.db");
    {
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.pragma_update(None, "user_version", SCHEMA_VERSION + 5).unwrap();
        conn.execute_batch("CREATE TABLE future (x INTEGER);").unwrap();
    }
    match Store::open(&path) {
        Err(StoreError::NewerSchema { found, supported }) => {
            assert_eq!((found, supported), (SCHEMA_VERSION + 5, SCHEMA_VERSION));
        }
        other => panic!("expected NewerSchema, got {other:?}"),
    }
    // The file is intact: the future table is still there.
    let conn = rusqlite::Connection::open(&path).unwrap();
    let n: i64 = conn.query_row("SELECT count(*) FROM sqlite_master WHERE name='future'", [], |r| r.get(0)).unwrap();
    assert_eq!(n, 1);
}

#[test]
fn a_file_that_is_not_sqlite_is_unreadable_not_a_panic() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("x.db");
    std::fs::write(&path, b"this is not a database").unwrap();
    assert!(matches!(Store::open(&path), Err(StoreError::Unreadable { .. })));
}
```

Also add `rusqlite` to the crate's `[dev-dependencies]` (the test uses it directly).

- [ ] **Step 3: red** — Run: `cargo test -p store` → FAIL (crate missing).

- [ ] **Step 4: `migrations.rs`**

```rust
//! The schema, versioned with `PRAGMA user_version`. Each migration takes it from N-1 to N
//! and never reruns: whoever already has the app only gets the new ones. Never tables
//! "ahead of time".

use rusqlite::Connection;

use crate::StoreError;

/// The version this binary knows how to read and write.
pub const SCHEMA_VERSION: u32 = 1;

/// Index = version − 1. Append at the end, never modify a migration that's already shipped.
const MIGRATIONS: [&str; 1] = [
    // 1: the user's goals. `target_json` is the serialized UnlockTarget: a column per
    // variant would be a schema that changes with every new kind of unlock.
    "CREATE TABLE goals (
        id TEXT PRIMARY KEY,
        target_json TEXT NOT NULL,
        created_unix INTEGER NOT NULL,
        note TEXT,
        seq INTEGER NOT NULL
    );",
];

pub fn current_version(conn: &Connection) -> Result<u32, StoreError> {
    conn.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
        .map(|v| v as u32)
        .map_err(StoreError::from_sqlite)
}

/// Brings the file to the current version. A file newer than us is refused without
/// being touched: an old binary must never corrupt a newer one's data.
pub fn apply(conn: &Connection) -> Result<(), StoreError> {
    let found = current_version(conn)?;
    if found > SCHEMA_VERSION {
        return Err(StoreError::NewerSchema { found, supported: SCHEMA_VERSION });
    }
    for (i, sql) in MIGRATIONS.iter().enumerate() {
        let target = i as u32 + 1;
        if target <= found {
            continue;
        }
        let tx = conn.unchecked_transaction().map_err(StoreError::from_sqlite)?;
        tx.execute_batch(sql).map_err(StoreError::from_sqlite)?;
        tx.pragma_update(None, "user_version", target).map_err(StoreError::from_sqlite)?;
        tx.commit().map_err(StoreError::from_sqlite)?;
    }
    Ok(())
}
```

- [ ] **Step 5: `lib.rs`**

```rust
//! store — the app's persistence: a single SQLite file in the data folder, versioned
//! schema. Writes exclusively to its own file; it doesn't even know the game's saves
//! exist. The frontend never touches disk itself: it goes through here via `ipc`.

mod migrations;

use std::path::Path;

use ipc::{Goal, GoalId, UnlockTarget};
use rusqlite::{params, Connection};

pub use migrations::SCHEMA_VERSION;

#[derive(Debug)]
pub enum StoreError {
    /// The file won't open or isn't SQLite.
    Unreadable { reason: String },
    /// The file was written by a newer version of the app.
    NewerSchema { found: u32, supported: u32 },
    /// A row doesn't read (malformed target JSON, missing column).
    Corrupt { reason: String },
}

impl StoreError {
    fn from_sqlite(e: rusqlite::Error) -> StoreError {
        StoreError::Unreadable { reason: e.to_string() }
    }
}

pub struct Store {
    conn: Connection,
}

impl Store {
    /// Opens (or creates) the file and brings it to the current schema version.
    pub fn open(path: &Path) -> Result<Store, StoreError> {
        let conn = Connection::open(path).map_err(StoreError::from_sqlite)?;
        // A file that isn't SQLite fails here, on the first query, not on open.
        migrations::current_version(&conn)?;
        migrations::apply(&conn)?;
        Ok(Store { conn })
    }

    pub fn schema_version(&self) -> Result<u32, StoreError> {
        migrations::current_version(&self.conn)
    }

    /// Goals in the order they were added.
    pub fn goals(&self) -> Result<Vec<Goal>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, target_json, created_unix, note FROM goals ORDER BY seq")
            .map_err(StoreError::from_sqlite)?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, Option<String>>(3)?,
                ))
            })
            .map_err(StoreError::from_sqlite)?;
        let mut out = Vec::new();
        for row in rows {
            let (id, target_json, created_unix, note) = row.map_err(StoreError::from_sqlite)?;
            let target: UnlockTarget = serde_json::from_str(&target_json)
                .map_err(|e| StoreError::Corrupt { reason: e.to_string() })?;
            out.push(Goal { id: GoalId::from_str_unchecked(&id), target, created_unix, note });
        }
        Ok(out)
    }

    /// Inserts, or replaces if the id already exists.
    pub fn add_goal(&self, g: &Goal) -> Result<(), StoreError> {
        let target_json = serde_json::to_string(&g.target)
            .map_err(|e| StoreError::Corrupt { reason: e.to_string() })?;
        self.conn
            .execute(
                "INSERT INTO goals (id, target_json, created_unix, note, seq)
                 VALUES (?1, ?2, ?3, ?4, COALESCE((SELECT max(seq) FROM goals), 0) + 1)
                 ON CONFLICT(id) DO UPDATE SET target_json = excluded.target_json,
                     created_unix = excluded.created_unix, note = excluded.note",
                params![g.id.as_str(), target_json, g.created_unix, g.note],
            )
            .map(|_| ())
            .map_err(StoreError::from_sqlite)
    }

    /// `true` if it existed.
    pub fn remove_goal(&self, id: &GoalId) -> Result<bool, StoreError> {
        self.conn
            .execute("DELETE FROM goals WHERE id = ?1", params![id.as_str()])
            .map(|n| n > 0)
            .map_err(StoreError::from_sqlite)
    }
}
```

Note on `ON CONFLICT ... DO UPDATE`: it keeps the original `seq`, so replacing doesn't change the order — that's what the `adding_the_same_id_twice_replaces_not_duplicates` test verifies together with the count.

- [ ] **Step 6: green and commit**

Run: `cargo test -p store && cargo fmt --check && cargo clippy --all-targets -- -D warnings`. The first build compiles SQLite (the `bundled` feature): it can take a minute.

```bash
git add crates/store Cargo.lock
git commit -m "store: born with the goals, one SQLite file and the versioned schema"
```

---

### Task 4: `ipc::graph` — the contract types and JSON shape

**Files:**
- Create: `crates/ipc/src/graph.rs`
- Modify: `crates/ipc/src/lib.rs`
- Test: `crates/ipc/tests/graph.rs` (JSON shape only, in this task)

**Interfaces:**
- Produces: `ipc::{UnlockNode, AchievementRef, GraphInfo, OriginView, UnlockView, UnlockTotals, UnlockDiagnostic, NextSteps, StepsBasis, PlanView, PlanExpansion, PlanStep, STEPS}`; `UnlockTarget` re-exported from `goals`.

- [ ] **Step 1: the shape tests** — `crates/ipc/tests/graph.rs`:

```rust
use ipc::{
    AchievementRef, GraphInfo, ItemKindView, NextSteps, OriginView, PlanExpansion, PlanView, StepsBasis,
    UnlockDiagnostic, UnlockNode, UnlockTarget, UnlockTotals, UnlockView, STEPS,
};
use serde_json::{json, to_value, Value};

fn node(done: bool) -> UnlockNode {
    UnlockNode {
        achievement: AchievementRef::Known { id: 1, text: "You unlocked \"Magdalene\"".into(), hint: Some("have 7 or more max red hearts at one time".into()), icon_url: None },
        done,
        unlocks: vec![UnlockTarget::Character { id: 1, name: "Magdalene".into() }],
        origin: None,
        graph: GraphInfo::Stub,
    }
}

#[test]
fn unlock_node_json_shape_is_pinned() {
    let v = to_value(node(true)).unwrap();
    assert_eq!(v["achievement"]["kind"], "known");
    assert_eq!(v["achievement"]["id"], 1);
    assert_eq!(v["achievement"]["hint"], "have 7 or more max red hearts at one time");
    assert_eq!(v["achievement"]["iconUrl"], Value::Null);
    assert_eq!(v["done"], true);
    assert_eq!(v["unlocks"][0]["kind"], "character");
    assert_eq!(v["origin"], Value::Null);
    assert_eq!(v["graph"], json!({ "kind": "stub" }));
}

#[test]
fn unknown_achievement_and_computed_graph_are_pinned_too() {
    let mut n = node(false);
    n.achievement = AchievementRef::Unknown { slot: 640 };
    n.graph = GraphInfo::Computed { available_now: true, blocked_by: 0, fan_out: 3, steps_missing: 1 };
    n.origin = Some(OriginView::AfterbirthPlus);
    let v = to_value(&n).unwrap();
    assert_eq!(v["achievement"], json!({ "kind": "unknown", "slot": 640 }));
    assert_eq!(v["graph"], json!({ "kind": "computed", "availableNow": true, "blockedBy": 0, "fanOut": 3, "stepsMissing": 1 }));
    assert_eq!(v["origin"], "afterbirthPlus");
}

#[test]
fn item_target_uses_item_kind_not_kind_for_the_item_type() {
    let t = UnlockTarget::Item { item_kind: ItemKindView::Trinket, id: 1, name: "Swallowed Penny".into(), icon_url: Some("data:image/png;base64,AA==".into()) };
    let v = to_value(&t).unwrap();
    assert_eq!(v["kind"], "item");
    assert_eq!(v["itemKind"]["kind"], "trinket");
    assert_eq!(v.as_object().unwrap().len(), 5, "kind, itemKind, id, name, iconUrl: no key gets overwritten");
}

#[test]
fn views_and_diagnostics_are_pinned() {
    let view = UnlockView {
        nodes: vec![node(true)],
        totals: UnlockTotals { slots: 642, done: 379, known: 637, unknown: 4 },
        diagnostics: vec![UnlockDiagnostic::SlotsBeyondCatalog { count: 4 }, UnlockDiagnostic::NoCatalog],
    };
    let v = to_value(&view).unwrap();
    assert_eq!(v["totals"], json!({ "slots": 642, "done": 379, "known": 637, "unknown": 4 }));
    assert_eq!(v["diagnostics"][0], json!({ "kind": "slotsBeyondCatalog", "count": 4 }));
    assert_eq!(v["diagnostics"][1], json!({ "kind": "noCatalog" }));

    let steps = NextSteps { steps: vec![], basis: StepsBasis::Stub };
    assert_eq!(to_value(&steps).unwrap()["basis"], json!({ "kind": "stub" }));
    assert_eq!(STEPS, 5);

    let plan = PlanView { goals: vec![], expansion: PlanExpansion::Stub, store_available: false };
    let v = to_value(&plan).unwrap();
    assert_eq!(v["expansion"], json!({ "kind": "stub" }));
    assert_eq!(v["storeAvailable"], false);
}
```

- [ ] **Step 2: red** — Run: `cargo test -p ipc --test graph` → FAIL.

- [ ] **Step 3: `graph.rs`** (only the types in this task; the functions come in Task 5)

```rust
//! The contracts for the graph screens: Unlock, Next Steps, Plan. A single node shared
//! by all three. Whatever the graph (M2) doesn't know yet travels as a declared `Stub`,
//! never as a value that looks computed.

use serde::{Deserialize, Serialize};

pub use crate::goals::{Goal, GoalId, UnlockTarget};

/// How many "next steps" the screen shows. Presentation, not domain.
pub const STEPS: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockNode {
    pub achievement: AchievementRef,
    /// From the save, section 1: real.
    pub done: bool,
    /// From the catalog, reverse index: real. Empty if it unlocks nothing known.
    pub unlocks: Vec<UnlockTarget>,
    /// Origin DLC of the first item unlocked: real.
    pub origin: Option<OriginView>,
    /// What the graph knows: `Stub` until M2 exists.
    pub graph: GraphInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum AchievementRef {
    Known { id: u32, text: String, hint: Option<String>, icon_url: Option<String> },
    /// In the save but not in the catalog: a patch newer than the file.
    Unknown { slot: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum GraphInfo {
    Stub,
    Computed { available_now: bool, blocked_by: u32, fan_out: u32, steps_missing: u32 },
}

/// `catalog::Origin` doesn't cross the IPC boundary: this is its view, like `ItemKindView` for `ItemKind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OriginView {
    Rebirth,
    Afterbirth,
    AfterbirthPlus,
    Repentance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockView {
    pub nodes: Vec<UnlockNode>,
    pub totals: UnlockTotals,
    pub diagnostics: Vec<UnlockDiagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockTotals {
    pub slots: u32,
    pub done: u32,
    pub known: u32,
    pub unknown: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum UnlockDiagnostic {
    SlotsBeyondCatalog { count: u32 },
    CatalogBeyondSlots { count: u32 },
    NoCatalog,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NextSteps {
    pub steps: Vec<UnlockNode>,
    pub basis: StepsBasis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum StepsBasis {
    Stub,
    FanOut,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanView {
    pub goals: Vec<Goal>,
    pub expansion: PlanExpansion,
    /// `false` when `store` failed to open: goals can't be seen or added, and the UI
    /// must say so instead of showing an empty list.
    pub store_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum PlanExpansion {
    Stub,
    Computed { steps: Vec<PlanStep> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanStep {
    pub goal: GoalId,
    pub node: UnlockNode,
    pub done: bool,
}
```

`lib.rs`: `mod graph;` and `pub use graph::{AchievementRef, GraphInfo, NextSteps, OriginView, PlanExpansion, PlanStep, PlanView, StepsBasis, UnlockDiagnostic, UnlockNode, UnlockTotals, UnlockView, STEPS};`. Note: `PlanView.store_available` is a deviation from the spec (which has a `StoreUnavailable` diagnostic in `plan()`): a boolean in the view-model is the shape the frontend consumes; the report states it.

- [ ] **Step 4: green and commit**

```bash
git add crates/ipc
git commit -m "ipc: the graph screens' contracts, with declared stubs and the JSON shape pinned"
```

---

### Task 5: `ipc::graph` — `unlock_view`, `next_steps`, `plan_view`

**Files:**
- Modify: `crates/ipc/src/graph.rs`, `crates/ipc/src/lib.rs`
- Test: `crates/ipc/tests/graph.rs` (extend)

**Interfaces:**
- Produces: `ipc::unlock_view(catalog: Option<&Catalog>, flags: &[bool], icon: impl FnMut(&str) -> Option<Vec<u8>>) -> UnlockView`, `ipc::next_steps(view: &UnlockView) -> NextSteps`, `ipc::plan_view(goals: Vec<Goal>, store_available: bool) -> PlanView`, `ipc::target_of(catalog: &Catalog, u: &catalog::Unlock, icon: &mut impl FnMut(&str) -> Option<Vec<u8>>) -> UnlockTarget`.

- [ ] **Step 1: the tests** — appended to `crates/ipc/tests/graph.rs`:

```rust
use catalog::Catalog;
use ipc::{next_steps, plan_view, unlock_view};

const ITEMS: &[u8] = b"<items gfxroot=\"gfx/items/\"><passive id=\"2\" gfx=\"a.png\" name=\"A\" achievement=\"1\" /><trinket id=\"1\" gfx=\"t.png\" name=\"T\" achievement=\"3\" /></items>";
const ACH: &[u8] = b"<achievements gfxroot=\"gfx/ui/achievement/\"><!-- c1 --><achievement id=\"1\" text=\"t1\" gfx=\"1.png\" /><achievement id=\"2\" text=\"t2\" gfx=\"2.png\" /><achievement id=\"3\" text=\"t3\" gfx=\"3.png\" /></achievements>";
const PLAYERS: &[u8] = b"<players portraitroot=\"gfx/ui/stage/\"><player id=\"7\" name=\"#Z_NAME\" portrait=\"z.png\" achievement=\"2\" /></players>";

fn catalog() -> Catalog {
    Catalog::build(|p| match p {
        "items.xml" => Some(ITEMS.to_vec()),
        "achievements.xml" => Some(ACH.to_vec()),
        "players.xml" => Some(PLAYERS.to_vec()),
        _ => None,
    })
}

#[test]
fn unlock_view_maps_slots_to_achievements_and_marks_the_ones_beyond_the_catalog() {
    // 6 slots: 0 unused, 1..=3 known, 4..=5 beyond the catalog.
    let flags = [false, true, false, true, true, false];
    let v = unlock_view(Some(&catalog()), &flags, |_| None);
    assert_eq!(v.nodes.len(), 5, "one per slot 1..=5");
    assert!(matches!(&v.nodes[0].achievement, AchievementRef::Known { id: 1, hint: Some(h), .. } if h == "c1"));
    assert_eq!(v.nodes[0].done, true);
    assert_eq!(v.nodes[1].done, false);
    assert_eq!(v.nodes[2].done, true);
    assert_eq!(v.nodes[3].achievement, AchievementRef::Unknown { slot: 4 });
    assert_eq!(v.nodes[3].done, true, "done, but the catalog doesn't know what it is");
    assert_eq!(v.nodes[4].achievement, AchievementRef::Unknown { slot: 5 });
    assert_eq!(v.totals, UnlockTotals { slots: 6, done: 3, known: 3, unknown: 2 });
    assert_eq!(v.diagnostics, vec![UnlockDiagnostic::SlotsBeyondCatalog { count: 2 }]);
    assert!(v.nodes.iter().all(|n| n.graph == GraphInfo::Stub));
}

#[test]
fn unlocks_and_origin_come_from_the_catalog_and_icons_only_when_they_resolve() {
    let flags = [false, false, false, false];
    let v = unlock_view(Some(&catalog()), &flags, |p| (p == "gfx/items/collectibles/a.png").then(|| vec![0x89, b'P', b'N', b'G']));
    let n1 = &v.nodes[0];
    assert_eq!(n1.unlocks.len(), 1);
    assert!(matches!(&n1.unlocks[0], UnlockTarget::Item { item_kind: ItemKindView::Passive, id: 2, name, icon_url: Some(u) } if name == "A" && u.starts_with("data:image/png")));
    assert_eq!(n1.origin, Some(OriginView::Rebirth), "item 2 is from Rebirth");
    let n2 = &v.nodes[1];
    assert!(matches!(&n2.unlocks[0], UnlockTarget::Character { id: 7, name } if name == "Z_NAME"));
    assert_eq!(n2.origin, None, "the first target isn't an item");
    let n3 = &v.nodes[2];
    assert!(matches!(&n3.unlocks[0], UnlockTarget::Item { item_kind: ItemKindView::Trinket, icon_url: None, .. }), "sprite not resolved: the row stays");
}

#[test]
fn catalog_beyond_slots_and_no_catalog_degrade_with_a_diagnostic() {
    let v = unlock_view(Some(&catalog()), &[false, true], |_| None);
    assert_eq!(v.nodes.len(), 1);
    assert_eq!(v.diagnostics, vec![UnlockDiagnostic::CatalogBeyondSlots { count: 2 }]);

    let v = unlock_view(None, &[false, true, true], |_| None);
    assert_eq!(v.nodes.len(), 2);
    assert!(v.nodes.iter().all(|n| matches!(n.achievement, AchievementRef::Unknown { .. })));
    assert_eq!(v.totals.done, 2);
    assert_eq!(v.diagnostics, vec![UnlockDiagnostic::NoCatalog]);
}

#[test]
fn next_steps_are_the_first_not_done_in_slot_order_capped_at_steps() {
    let mut flags = vec![false; 10];
    flags[2] = true;
    flags[5] = true;
    let v = unlock_view(None, &flags, |_| None);
    let s = next_steps(&v);
    assert_eq!(s.basis, StepsBasis::Stub);
    let slots: Vec<u32> = s.steps.iter().map(|n| match n.achievement { AchievementRef::Unknown { slot } => slot, AchievementRef::Known { id, .. } => id }).collect();
    assert_eq!(slots, vec![1, 3, 4, 6, 7]);
    assert_eq!(s.steps.len(), STEPS);
}

#[test]
fn plan_view_keeps_goal_order_and_reports_the_store() {
    let g = |id: &str| ipc::Goal { id: ipc::GoalId::from_str_unchecked(id), target: UnlockTarget::Boss { id: 1, name: "Monstro".into() }, created_unix: 0, note: None };
    let p = plan_view(vec![g("b"), g("a")], true);
    assert_eq!(p.goals.iter().map(|g| g.id.as_str()).collect::<Vec<_>>(), vec!["b", "a"]);
    assert_eq!(p.expansion, PlanExpansion::Stub);
    assert!(p.store_available);
    assert!(!plan_view(vec![], false).store_available);
}
```

- [ ] **Step 2: red** — Run: `cargo test -p ipc --test graph` → FAIL.

- [ ] **Step 3: the functions** — appended to `graph.rs`:

```rust
use catalog::{AchievementId, Catalog, ItemKind, Language, Origin, Unlock};

use crate::catalog_view::ItemKindView;
use crate::resources::data_url;

/// The Unlock view: one node per slot 1..=N of section 1 of the save. `flags[i]` is
/// slot i; slot 0 is unused (the `slot[id]` mapping, verified on 2026-09-05: 169 items
/// out of 171 seen with the achievement done).
pub fn unlock_view(
    catalog: Option<&Catalog>,
    flags: &[bool],
    mut icon: impl FnMut(&str) -> Option<Vec<u8>>,
) -> UnlockView {
    let slots = flags.len() as u32;
    let mut nodes = Vec::with_capacity(flags.len().saturating_sub(1));
    let (mut done, mut known, mut unknown) = (0u32, 0u32, 0u32);
    for (slot, &flag) in flags.iter().enumerate().skip(1) {
        let slot = slot as u32;
        let achievement = catalog.and_then(|c| c.achievement(AchievementId(slot)));
        let (achievement_ref, unlocks, origin) = match (catalog, achievement) {
            (Some(c), Some(a)) => {
                known += 1;
                let unlocks: Vec<UnlockTarget> = c.unlocks(a.id).iter().map(|u| target_of(c, u, &mut icon)).collect();
                let origin = first_item_origin(c, c.unlocks(a.id));
                (
                    AchievementRef::Known {
                        id: a.id.0,
                        text: a.text.clone(),
                        hint: a.unlock_condition.clone(),
                        icon_url: icon(&a.sprite.path).map(|png| data_url(&png)),
                    },
                    unlocks,
                    origin,
                )
            }
            _ => {
                unknown += 1;
                (AchievementRef::Unknown { slot }, Vec::new(), None)
            }
        };
        if flag {
            done += 1;
        }
        nodes.push(UnlockNode { achievement: achievement_ref, done: flag, unlocks, origin, graph: GraphInfo::Stub });
    }

    let mut diagnostics = Vec::new();
    match catalog {
        None => diagnostics.push(UnlockDiagnostic::NoCatalog),
        Some(c) => {
            let in_catalog = c.achievements().count() as u32;
            if unknown > 0 {
                diagnostics.push(UnlockDiagnostic::SlotsBeyondCatalog { count: unknown });
            }
            if in_catalog + 1 > slots {
                diagnostics.push(UnlockDiagnostic::CatalogBeyondSlots { count: in_catalog + 1 - slots });
            }
        }
    }
    UnlockView { nodes, totals: UnlockTotals { slots, done, known, unknown }, diagnostics }
}

/// A catalog edge as the UI sees it: name resolved, icon when it can be extracted.
pub fn target_of(c: &Catalog, u: &Unlock, icon: &mut impl FnMut(&str) -> Option<Vec<u8>>) -> UnlockTarget {
    match *u {
        Unlock::Item { kind, id } => {
            let item = c.item(kind, id);
            UnlockTarget::Item {
                item_kind: kind_view(kind),
                id: id.0,
                name: item.map(|i| c.text(&i.name, Language::English).to_string()).unwrap_or_default(),
                icon_url: item.and_then(|i| icon(&i.sprite.path)).map(|png| data_url(&png)),
            }
        }
        Unlock::Character { id } => UnlockTarget::Character {
            id: id.0,
            name: c.character(id).map(|ch| c.text(&ch.name, Language::English).to_string()).unwrap_or_default(),
        },
        Unlock::Boss { id } => UnlockTarget::Boss { id: id.0, name: c.boss(id).map(|b| b.name.clone()).unwrap_or_default() },
        Unlock::Challenge { id } => UnlockTarget::Challenge { id: id.0, name: c.challenge(id).map(|ch| ch.name.clone()).unwrap_or_default() },
    }
}

fn first_item_origin(c: &Catalog, unlocks: &[Unlock]) -> Option<OriginView> {
    match unlocks.first()? {
        Unlock::Item { kind, id } => c.item(*kind, *id)?.origin.map(origin_view),
        Unlock::Character { .. } | Unlock::Boss { .. } | Unlock::Challenge { .. } => None,
    }
}

fn kind_view(k: ItemKind) -> ItemKindView {
    match k {
        ItemKind::Passive => ItemKindView::Passive,
        ItemKind::Active => ItemKindView::Active,
        ItemKind::Familiar => ItemKindView::Familiar,
        ItemKind::Trinket => ItemKindView::Trinket,
    }
}

fn origin_view(o: Origin) -> OriginView {
    match o {
        Origin::Rebirth => OriginView::Rebirth,
        Origin::Afterbirth => OriginView::Afterbirth,
        Origin::AfterbirthPlus => OriginView::AfterbirthPlus,
        Origin::Repentance => OriginView::Repentance,
    }
}

/// Without a graph: the first `STEPS` not done, in slot order. `basis: Stub` declares it.
pub fn next_steps(view: &UnlockView) -> NextSteps {
    NextSteps {
        steps: view.nodes.iter().filter(|n| !n.done).take(STEPS).cloned().collect(),
        basis: StepsBasis::Stub,
    }
}

/// The plan: the goals as they are, and an expansion M3 can't compute yet.
pub fn plan_view(goals: Vec<Goal>, store_available: bool) -> PlanView {
    PlanView { goals, expansion: PlanExpansion::Stub, store_available }
}
```

`kind_view` duplicates the `match` from `item_views` in `catalog_view.rs`: replace that one with a call to `kind_view` made `pub(crate)` in `catalog_view.rs` (or vice versa), a single implementation. `lib.rs`: `pub use graph::{next_steps, plan_view, target_of, unlock_view};`.

- [ ] **Step 4: green and commit**

Run: `cargo test -p ipc && cargo fmt --check && cargo clippy --all-targets -- -D warnings`

```bash
git add crates/ipc
git commit -m "ipc: unlock_view from the catalog and the save, next steps and plan with declared stubs"
```

---

### Task 6: real-data test — the junction pinned

**Files:**
- Create: `crates/ipc/tests/graph_real.rs`
- Modify: `crates/ipc/Cargo.toml` (`core-save` is already a dependency; so are `unpack` and `catalog`)

- [ ] **Step 1: the file**

```rust
//! Against the real profile `samples/live.rep+persistentgamedata1.dat` (2026-08-31)
//! and the real catalog via `samples/packed`. They skip with a note if something is
//! missing. The numbers are fixtures of known origin, measured on 2026-09-05.

use std::collections::BTreeSet;

use catalog::{Catalog, ItemKind};
use core_save::{Kind, Save};
use ipc::{next_steps, unlock_view, AchievementRef, UnlockTarget};
use unpack::ResourceSet;

fn samples() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../samples")
}

fn real() -> Option<(Catalog, ResourceSet, Save)> {
    let packed = samples().join("packed");
    let save = samples().join("live.rep+persistentgamedata1.dat");
    if !packed.is_dir() || !save.is_file() {
        eprintln!("skip: samples/packed or the live profile is missing");
        return None;
    }
    let rs = ResourceSet::open(&packed);
    let c = Catalog::build(|p| rs.read(p));
    let s = Save::open(&save).ok()?;
    Some((c, rs, s))
}

#[test]
fn the_real_profile_has_379_done_637_known_and_4_unknown_slots() {
    let Some((c, rs, s)) = real() else { return };
    let flags = s.flags(Kind::Achievements).expect("section 1");
    let v = unlock_view(Some(&c), &flags, |p| rs.read(p));
    assert_eq!((v.totals.slots, v.totals.done, v.totals.known, v.totals.unknown), (642, 379, 637, 4));
    let unknown_done: Vec<u32> = v.nodes.iter().filter_map(|n| match n.achievement {
        AchievementRef::Unknown { slot } if n.done => Some(slot),
        AchievementRef::Unknown { .. } | AchievementRef::Known { .. } => None,
    }).collect();
    assert_eq!(unknown_done, vec![638, 639, 641], "achievements newer than the catalog, done");
}

#[test]
fn the_slot_id_junction_is_pinned_by_the_items_seen_in_the_save() {
    // Among the items seen (section 4) that have an achievement, how many have the
    // `done` node? slot[id]: 169 of 171; slot[id-1]: 145; slot[id+1]: 142. If it drops,
    // it's the alignment that's broken, not a number to adjust.
    let Some((c, _, s)) = real() else { return };
    let flags = s.flags(Kind::Achievements).expect("section 1");
    let seen = s.flags(Kind::Items).expect("section 4");
    let v = unlock_view(Some(&c), &flags, |_| None);
    let done: BTreeSet<u32> = v.nodes.iter().filter(|n| n.done).filter_map(|n| match n.achievement { AchievementRef::Known { id, .. } => Some(id), AchievementRef::Unknown { .. } => None }).collect();
    let (mut agree, mut total) = (0, 0);
    for i in c.items().filter(|i| i.kind != ItemKind::Trinket) {
        let Some(a) = i.unlocked_by else { continue };
        if !seen.get(i.id.0 as usize).copied().unwrap_or(false) {
            continue;
        }
        total += 1;
        if done.contains(&a.0) {
            agree += 1;
        }
    }
    assert_eq!((agree, total), (169, 171));
}

#[test]
fn next_steps_on_the_real_profile_are_five_not_done_known_nodes_in_slot_order() {
    let Some((c, rs, s)) = real() else { return };
    let flags = s.flags(Kind::Achievements).expect("section 1");
    let v = unlock_view(Some(&c), &flags, |p| rs.read(p));
    let steps = next_steps(&v);
    assert_eq!(steps.steps.len(), 5);
    assert!(steps.steps.iter().all(|n| !n.done));
    let ids: Vec<u32> = steps.steps.iter().map(|n| match n.achievement { AchievementRef::Known { id, .. } => id, AchievementRef::Unknown { slot } => slot }).collect();
    assert!(ids.windows(2).all(|w| w[0] < w[1]), "increasing slot order: {ids:?}");
    // Every step has a readable name and, if it unlocks an item, its icon.
    for n in &steps.steps {
        for t in &n.unlocks {
            match t {
                UnlockTarget::Item { name, icon_url, .. } => { assert!(!name.is_empty()); assert!(icon_url.is_some()); }
                UnlockTarget::Character { name, .. } | UnlockTarget::Boss { name, .. } | UnlockTarget::Challenge { name, .. } => assert!(!name.is_empty()),
            }
        }
    }
}
```

- [ ] **Step 2: run it** — Run: `cargo test -p ipc --test graph_real -- --nocapture` → 3 green, none skipped.

- [ ] **Step 3: commit**

```bash
git add crates/ipc/tests/graph_real.rs
git commit -m "ipc: the achievement <-> slot junction pinned on the real profile"
```

---

### Task 7: `app` — `Store` in state, five commands

**Files:**
- Modify: `crates/app/Cargo.toml` (+ `store = { path = "../store" }`), `crates/app/src/error.rs`, `crates/app/src/lib.rs`

- [ ] **Step 1: `error.rs`** — two variants:

```rust
    /// A goal's target doesn't exist in the catalog.
    UnknownTarget,
    /// The app's database won't open: goals can neither be read nor written.
    StoreUnavailable {
        reason: String,
    },
```

- [ ] **Step 2: `lib.rs`** — state and commands. Above the commands:

```rust
use std::sync::Mutex;

use store::{Store, StoreError};

/// The app's database, opened once. `None` if it didn't open: the commands degrade
/// instead of retrying on every call.
struct StoreState(OnceLock<Option<Mutex<Store>>>);

impl StoreState {
    fn get_or_open(&self, app: &AppHandle) -> Option<&Mutex<Store>> {
        self.0
            .get_or_init(|| {
                let path = app.path().app_data_dir().ok()?.join("isaacdome.db");
                if let Some(dir) = path.parent() {
                    std::fs::create_dir_all(dir).ok()?;
                }
                Store::open(&path).ok().map(Mutex::new)
            })
            .as_ref()
    }
}

/// A `store` error mapped across the IPC boundary, without its `Debug` (which could
/// carry a path in SQLite's own message).
fn store_error(e: StoreError) -> IpcError {
    let reason = match e {
        StoreError::Unreadable { .. } => "unreadable database".to_string(),
        StoreError::NewerSchema { found, supported } => format!("database from a newer version ({found} > {supported})"),
        StoreError::Corrupt { .. } => "a database row doesn't read".to_string(),
    };
    IpcError::StoreUnavailable { reason }
}
```

The commands:

```rust
/// Section 1 of the active profile, already read. `NoActiveProfile` as with the others.
fn achievement_flags(app: &AppHandle) -> Result<Vec<bool>, IpcError> {
    let (_, save) = active_save(app)?;
    Ok(save.flags(Kind::Achievements).unwrap_or_default())
}

#[tauri::command]
fn unlock(app: AppHandle, state: tauri::State<'_, CatalogState>) -> Result<ipc::UnlockView, IpcError> {
    let flags = achievement_flags(&app)?;
    let d = discover(&Options::default());
    let resources = d.game.as_ref().map(|g| ResourceSet::open(&g.dir.join("resources").join("packed")));
    let catalog = resources.as_ref().and_then(|rs| state.get_or_build(rs));
    Ok(ipc::unlock_view(catalog, &flags, |p| resources.as_ref().and_then(|rs| rs.read(p))))
}

#[tauri::command]
fn next_steps(app: AppHandle, state: tauri::State<'_, CatalogState>) -> Result<ipc::NextSteps, IpcError> {
    let view = unlock(app, state)?;
    Ok(ipc::next_steps(&view))
}

#[tauri::command]
fn plan(app: AppHandle, store: tauri::State<'_, StoreState>) -> Result<ipc::PlanView, IpcError> {
    let Some(s) = store.get_or_open(&app) else {
        return Ok(ipc::plan_view(Vec::new(), false));
    };
    let goals = s.lock().map_err(|_| IpcError::StoreUnavailable { reason: "database state unavailable".into() })?.goals().map_err(store_error)?;
    Ok(ipc::plan_view(goals, true))
}

#[tauri::command]
fn add_goal(
    app: AppHandle,
    store: tauri::State<'_, StoreState>,
    catalog: tauri::State<'_, CatalogState>,
    target: ipc::UnlockTarget,
) -> Result<ipc::PlanView, IpcError> {
    // The target must exist in the catalog: a goal on a made-up id doesn't get saved.
    let d = discover(&Options::default());
    let resources = d.game.as_ref().map(|g| ResourceSet::open(&g.dir.join("resources").join("packed")));
    let known = resources.as_ref().and_then(|rs| catalog.get_or_build(rs)).is_some_and(|c| target_exists(c, &target));
    if !known {
        return Err(IpcError::UnknownTarget);
    }
    let Some(s) = store.get_or_open(&app) else {
        return Err(IpcError::StoreUnavailable { reason: "database not open".into() });
    };
    let created_unix = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
    let goal = ipc::Goal { id: ipc::GoalId::new(), target, created_unix, note: None };
    let guard = s.lock().map_err(|_| IpcError::StoreUnavailable { reason: "database state unavailable".into() })?;
    guard.add_goal(&goal).map_err(store_error)?;
    Ok(ipc::plan_view(guard.goals().map_err(store_error)?, true))
}

#[tauri::command]
fn remove_goal(app: AppHandle, store: tauri::State<'_, StoreState>, id: ipc::GoalId) -> Result<ipc::PlanView, IpcError> {
    let Some(s) = store.get_or_open(&app) else {
        return Err(IpcError::StoreUnavailable { reason: "database not open".into() });
    };
    let guard = s.lock().map_err(|_| IpcError::StoreUnavailable { reason: "database state unavailable".into() })?;
    guard.remove_goal(&id).map_err(store_error)?;
    Ok(ipc::plan_view(guard.goals().map_err(store_error)?, true))
}

fn target_exists(c: &catalog::Catalog, t: &ipc::UnlockTarget) -> bool {
    use catalog::{BossId, ChallengeId, CharacterId, ItemId, ItemKind};
    match t {
        ipc::UnlockTarget::Item { item_kind, id, .. } => {
            let kind = match item_kind {
                ipc::ItemKindView::Passive => ItemKind::Passive,
                ipc::ItemKindView::Active => ItemKind::Active,
                ipc::ItemKindView::Familiar => ItemKind::Familiar,
                ipc::ItemKindView::Trinket => ItemKind::Trinket,
            };
            c.item(kind, ItemId(*id)).is_some()
        }
        ipc::UnlockTarget::Character { id, .. } => c.character(CharacterId(*id)).is_some(),
        ipc::UnlockTarget::Boss { id, .. } => c.boss(BossId(*id)).is_some(),
        ipc::UnlockTarget::Challenge { id, .. } => c.challenge(ChallengeId(*id)).is_some(),
    }
}
```

`run()`: `.manage(StoreState(OnceLock::new()))` and the five commands in `generate_handler!`. `catalog` is already a dependency of `app`. **Note on the double `ResourceSet::open`**: `unlock` and `add_goal` open the archive set on every call (1.3 GB); it's the already-logged open block in `STATUS.md`, not to be fixed here — but `next_steps` must **not** open it twice: it calls `unlock` just once, as above.

- [ ] **Step 3: green and commit**

Run: `cargo build -p app && cargo test --workspace && cargo fmt --check && cargo clippy --all-targets -- -D warnings`

```bash
git add crates/app Cargo.lock
git commit -m "app: unlock, next steps and plan as commands, with the store opened once"
```

---

### Task 8: UI — types, wrapper, verification screen

**Files:**
- Modify: `ui/src/lib/ipc/types.ts`, `ui/src/lib/constants/commands.ts`, `ui/src/App.vue`
- Create: `ui/src/lib/ipc/graph.ts`

- [ ] **Step 1: `types.ts`** — appended:

```ts
// Mirrors crates/ipc/src/graph.rs and goals.rs. Every enum is a union on `kind`;
// `UnlockTarget.item` carries `itemKind`, not `kind`: `kind` is the tag.
export type AchievementRef =
  | { kind: 'known'; id: number; text: string; hint: string | null; iconUrl: string | null }
  | { kind: 'unknown'; slot: number }

export type UnlockTarget =
  | { kind: 'item'; itemKind: ItemKindView; id: number; name: string; iconUrl: string | null }
  | { kind: 'character'; id: number; name: string }
  | { kind: 'boss'; id: number; name: string }
  | { kind: 'challenge'; id: number; name: string }

// Mirrors ipc::ItemKindView, a tagged enum: an object with `kind`, not a string.
// (Didn't exist in TS yet: the verification screen didn't need it.)
export type ItemKindView =
  | { kind: 'passive' }
  | { kind: 'active' }
  | { kind: 'familiar' }
  | { kind: 'trinket' }

export type OriginView = 'rebirth' | 'afterbirth' | 'afterbirthPlus' | 'repentance'

export type GraphInfo =
  | { kind: 'stub' }
  | { kind: 'computed'; availableNow: boolean; blockedBy: number; fanOut: number; stepsMissing: number }

export interface UnlockNode {
  achievement: AchievementRef
  done: boolean
  unlocks: UnlockTarget[]
  origin: OriginView | null
  graph: GraphInfo
}

export interface UnlockTotals { slots: number; done: number; known: number; unknown: number }

export type UnlockDiagnostic =
  | { kind: 'slotsBeyondCatalog'; count: number }
  | { kind: 'catalogBeyondSlots'; count: number }
  | { kind: 'noCatalog' }

export interface UnlockView { nodes: UnlockNode[]; totals: UnlockTotals; diagnostics: UnlockDiagnostic[] }

export type StepsBasis = { kind: 'stub' } | { kind: 'fanOut' }
export interface NextSteps { steps: UnlockNode[]; basis: StepsBasis }

export type GoalId = string
export interface Goal { id: GoalId; target: UnlockTarget; createdUnix: number; note: string | null }
export interface PlanStep { goal: GoalId; node: UnlockNode; done: boolean }
export type PlanExpansion = { kind: 'stub' } | { kind: 'computed'; steps: PlanStep[] }
export interface PlanView { goals: Goal[]; expansion: PlanExpansion; storeAvailable: boolean }
```

Careful: `OriginView` as a string union violates the "no string unions as discriminators" convention **only if used as a discriminator**: here it's a value. If `pnpm scan` or the reviewer objects, convert it to `const OriginView = { Rebirth: 'rebirth', … } as const` with the derived type — the convention allows for it. `ItemKindView`, on the other hand, is a tagged enum (an object with `kind`), as in Rust: no string union. The existing TypeScript for `ItemView.kind` (`{ kind: { kind: 'passive' } }`) stays as it is.

Add to `IpcError`: `| { kind: 'unknownTarget' } | { kind: 'storeUnavailable'; reason: string }`, and in `App.vue` the exhaustive `switch` on `IpcError` must cover the two new cases (show them as an error).

- [ ] **Step 2: `commands.ts`** — `Unlock: 'unlock', NextSteps: 'next_steps', Plan: 'plan', AddGoal: 'add_goal', RemoveGoal: 'remove_goal'`.

- [ ] **Step 3: `graph.ts`**

```ts
import { invoke } from '@tauri-apps/api/core'
import { Command } from '../constants/commands'
import type { GoalId, NextSteps, PlanView, UnlockTarget, UnlockView } from './types'

export const unlock = (): Promise<UnlockView> => invoke(Command.Unlock)
export const nextSteps = (): Promise<NextSteps> => invoke(Command.NextSteps)
export const plan = (): Promise<PlanView> => invoke(Command.Plan)
export const addGoal = (target: UnlockTarget): Promise<PlanView> => invoke(Command.AddGoal, { target })
export const removeGoal = (id: GoalId): Promise<PlanView> => invoke(Command.RemoveGoal, { id })
```

- [ ] **Step 4: `App.vue`** — after the matrix section, when the profile is active: `steps = await nextSteps()` and `unlockView = await unlock()` in `load`, and:

```vue
    <section v-if="unlockView" class="flex flex-col gap-2">
      <h2 class="font-bold">
        Unlock: {{ unlockView.totals.done }} done out of {{ unlockView.totals.slots }}
        ({{ unlockView.totals.unknown }} unknown to the catalog)
      </h2>
      <p v-for="(d, i) in unlockView.diagnostics" :key="i" class="opacity-muted">
        {{ JSON.stringify(d) }}
      </p>
    </section>

    <section v-if="steps" class="flex flex-col gap-2">
      <h2 class="font-bold">
        Next steps ({{ steps.basis.kind === 'stub' ? 'slot order: there is no graph yet' : 'by fan-out' }})
      </h2>
      <ul class="flex flex-col gap-1">
        <li v-for="(n, i) in steps.steps" :key="i" class="flex flex-row items-center gap-2">
          <template v-if="n.achievement.kind === 'known'">
            <img v-if="n.achievement.iconUrl" :src="n.achievement.iconUrl" :alt="n.achievement.text" class="pixelated h-sprite" />
            <span>{{ n.achievement.text }}</span>
            <span v-if="n.achievement.hint" class="opacity-muted">— {{ n.achievement.hint }}</span>
          </template>
          <span v-else class="opacity-muted">slot {{ n.achievement.slot }}: done or not, the catalog doesn't know what it is</span>
        </li>
      </ul>
    </section>
```

`h-sprite` uses the `--spacing-sprite` token already in `@theme`; if the achievement icons need a different height (263×176, not square), add a `--spacing-achievement` token in `main.css`, never a hardcoded value. Don't add UI for goals: they're commands verified by tests, the screen arrives with the design system.

- [ ] **Step 5: checks and a look** — `pnpm typecheck && pnpm lint && pnpm scan && pnpm format:check` (format with `pnpm --filter ui format` if needed); `pnpm dev`, look at: "Unlock: 379 done out of 642 (4 unknown to the catalog)" and five steps with text and icon; screenshot kept outside the repo; close the app.

- [ ] **Step 6: commit**

```bash
git add ui/src
git commit -m "ui: types and wrappers for the graph contracts, and the Unlock and Next steps verification"
```

---

### Task 9: documents and report

**Files:**
- Create: `docs/superpowers/plans/2026-09-05-graph-contracts-report.md`
- Modify: `docs/STATUS.md`, `DESIGN-BRIEF.md`, `CLAUDE.md` (module table: `store` is no longer future-tense "SQLite: run archive, snapshots, catalog, plans" — it's born with the goals), `README.md` if it mentions `store`

- [ ] **Step 1: `STATUS.md`** — "Delivery to design": step 2 checked off with the three decisions checked off; table: Unlock 🟡 → "🟢 designable on the contract: real nodes, `graph` stub", Next Steps and Plan 🔴 → "🟢 on the contract"; new section `### store — app persistence ✅ (born with the goals)` with spec, migration 1, tests; session log with commit and test count.
- [ ] **Step 2: `DESIGN-BRIEF.md`** — §4 table updated as above; new §6 "The graph contracts" with the TypeScript types and the spec's real/stub table; §11 with a fourth question: "how do you draw a node whose graph is `stub`, without it looking like missing data?".
- [ ] **Step 3: report** with the same structure as previous ones: built, caught, decisions (in particular `store_available` in place of the diagnostic, unified `kind_view`, the double `ResourceSet::open` left at the open block), what's missing (M2, M3, `store`'s migration 2).
- [ ] **Step 4: commit**

```bash
git add docs DESIGN-BRIEF.md CLAUDE.md README.md
git commit -m "docs: graph contracts delivered, store born, status and brief aligned"
```

---

## Self-review (done while writing)

- **Spec coverage:** reverse index (T1); `Goal`/`GoalId`/`UnlockTarget` (T2); `store` with migration 1, `NewerSchema`, `Unreadable` (T3); contract types with pinned shape, including `unknown`, `stub`, `itemKind` (T4); `unlock_view`/`next_steps`/`plan_view` with the `slot[id]` junction, diagnostics, origin, icons (T5); real tests 642/379/637/4, 169/171, slots 638-639-641 (T6); commands with `Store` in state and degradation (T7); TS and screen (T8); documents (T9).
- **Deviations from the spec, declared:** `PlanView.store_available: bool` in place of a `StoreUnavailable` diagnostic in `plan()` (the frontend consumes a boolean; the typed error stays for `add_goal`/`remove_goal`); `Unlock` derives `Ord` for deterministic ordering instead of a comparator; `GoalId::new` with no dependency on a UUID crate (entropy from time + address + pid: enough to distinguish goals on one machine, not a security token — the doc says so).
- **Types consistent across tasks:** `UnlockTarget` defined in `goals.rs` (T2) and used by `graph.rs` (T4/T5), `store` (T3), and `app` (T7); `ItemKindView` gains `Deserialize, Hash` in T2 because `Goal` deserializes it; `Catalog::unlocks` (T1) consumed by T5; `catalog::{boss, challenge}` (plan B) consumed by T5 and T7; `data_url` reused; `STEPS` in T4 used in T5/T6.
- **Known risk:** the 169/171 test (T6) depends on the `live.rep+…1.dat` profile as it stands today; if the profile gets updated by playing, the number changes and the test will say so — that's intended: the file should be copied into `samples/` with a date, not kept as "live" (to note in `STATUS.md` at T9 as something to do: rename `live.*` with its date, as done for the other samples).
