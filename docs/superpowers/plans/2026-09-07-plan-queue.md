# Plan queue Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** One ordered list of achievements — what you want to unlock, in the order you intend to do it — where your order can never contradict the graph, and a move that would break it repairs the list instead of refusing.

**Architecture:** A new pure crate `crates/plan` owns the queue and the repair algorithm, and knows nothing about SQL, Tauri or the UI. `store` persists the queue as **one JSON document** (migration 2), because an order is a position, not a set of sequence numbers that can contradict each other. `ipc` turns it into resolved view-models, `app` wires four commands plus a one-off import of the old goals.

**Tech Stack:** Rust 2021, `serde`/`serde_json`, `rusqlite` (bundled), existing crates `graph`, `catalog`, `store`, `ipc`.

**Spec:** `docs/superpowers/specs/2026-09-07-plan-queue-design.md` — read it first; this plan argues from it.

## Global Constraints

- **Read-only on saves.** The queue lives in `isaacdome.db`; the game's files are never written.
- **Degrade, never fail.** An unreadable queue is a *declared* empty queue: never silent, never a panic.
- **A read never writes.** No command that returns the queue may modify it — the goals import is its own explicit command.
- **No `panic!` / `unwrap()`** outside tests on data read from disk.
- **Every struct crossing the IPC** carries `#[serde(rename_all = "camelCase")]`; every enum with struct variants also carries `rename_all_fields = "camelCase"`.
- **Fieldless enums on the IPC are bare camelCase strings**, never tagged objects.
- **Exhaustiveness is mandatory:** no `_ =>` arm on a closed enum.
- **Comments, doc-comments and `assert!` messages in English.**
- **Commits:** Conventional Commits, `type(scope): subject`, scope `plan` / `store` / `ipc` / `app`. **Never** a `Co-Authored-By` trailer or any reference to Claude.
- **Before declaring done:** `pnpm check`.
- Real-data tests go through `test-support` and **skip with a note**; run with `cargo test --workspace -- --nocapture` so the skips are visible.

---

### Task 1: The queue model

**Files:**
- Create: `crates/plan/Cargo.toml`, `crates/plan/src/lib.rs`, `crates/plan/src/model.rs`
- Test: `crates/plan/tests/model.rs`

**Interfaces:**
- Consumes: nothing.
- Produces: `plan::{Queue, Row}`; `Queue::from_rows(Vec<Row>)`, `Queue::rows() -> &[Row]`, `Queue::position(u32) -> Option<usize>`, `Queue::to_json() -> String`, `Queue::from_json(&str) -> Result<Queue, QueueError>`, `plan::QueueError`.

- [ ] **Step 1: Write the failing test**

`crates/plan/tests/model.rs`:

```rust
use plan::{Queue, Row};

fn row(achievement: u32, wanted: bool, origins: &[u32]) -> Row {
    Row {
        achievement,
        wanted,
        origins: origins.to_vec(),
    }
}

#[test]
fn the_document_round_trips_and_the_order_is_the_position() {
    let q = Queue::from_rows(vec![
        row(89, false, &[41]),
        row(41, true, &[]),
        row(512, true, &[]),
    ]);
    let json = q.to_json();
    assert!(
        json.starts_with('['),
        "the document is an array: the order is the position, not a column — got {json}"
    );
    let back = Queue::from_json(&json).expect("round trip");
    assert_eq!(back, q);
    assert_eq!(back.position(41), Some(1));
    assert_eq!(back.position(999), None);
}

#[test]
fn the_json_field_names_are_the_ones_written_in_the_spec() {
    let q = Queue::from_rows(vec![row(89, true, &[41, 512])]);
    let v: serde_json::Value = serde_json::from_str(&q.to_json()).expect("parses");
    assert_eq!(
        v[0],
        serde_json::json!({ "achievement": 89, "wanted": true, "origins": [41, 512] })
    );
}

#[test]
fn a_document_that_does_not_parse_is_an_error_not_an_empty_queue() {
    let err = Queue::from_json("{ not json").expect_err("must not read as empty");
    assert!(
        format!("{err:?}").contains("Unreadable"),
        "an unreadable queue and an empty queue are different things, got {err:?}"
    );
}

#[test]
fn an_empty_document_is_an_empty_queue() {
    assert_eq!(Queue::from_json("[]").expect("parses").rows().len(), 0);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p plan --test model`
Expected: FAIL — the package `plan` does not exist.

- [ ] **Step 3: Write minimal implementation**

`crates/plan/Cargo.toml`:

```toml
[package]
name = "plan"
version = "0.1.0"
edition = "2021"
description = "The plan queue: an ordered series of achievements, kept consistent with the graph"

[dependencies]
graph = { path = "../graph" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

`crates/plan/src/model.rs`:

```rust
//! The queue and its rows. The order **is** the position in the array: there is no
//! sequence column, and therefore no way to write an order that contradicts itself.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    pub achievement: u32,
    /// You asked for this one, for itself.
    pub wanted: bool,
    /// Every wanted achievement whose chain passes through this row. A list, not a single
    /// value: two wishes can need the same step, and with one slot the second would be
    /// lost — visibly, on removal, when a step another wish still needs looks orphaned.
    pub origins: Vec<u32>,
}

impl Row {
    /// A row with neither of the two reasons to exist is an orphan, and it goes.
    pub fn is_orphan(&self) -> bool {
        !self.wanted && self.origins.is_empty()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Queue(Vec<Row>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueueError {
    Unreadable { reason: String },
}

impl std::fmt::Display for QueueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueueError::Unreadable { reason } => write!(f, "unreadable queue: {reason}"),
        }
    }
}

impl std::error::Error for QueueError {}

impl Queue {
    pub fn from_rows(rows: Vec<Row>) -> Queue {
        Queue(rows)
    }

    pub fn rows(&self) -> &[Row] {
        &self.0
    }

    pub fn position(&self, achievement: u32) -> Option<usize> {
        self.0.iter().position(|r| r.achievement == achievement)
    }

    pub fn to_json(&self) -> String {
        // A `Vec<Row>` of plain integers and bools: serialization cannot fail. The
        // fallback is an empty document rather than an `unwrap`, and it is unreachable.
        serde_json::to_string(self).unwrap_or_else(|_| "[]".to_string())
    }

    pub fn from_json(s: &str) -> Result<Queue, QueueError> {
        serde_json::from_str(s).map_err(|e| QueueError::Unreadable {
            reason: e.to_string(),
        })
    }
}
```

`crates/plan/src/lib.rs`:

```rust
//! The plan queue: what you want to unlock, in the order you mean to do it.
//!
//! Pure — no SQL, no Tauri, no view-models. The order is yours; the only thing this crate
//! insists on is that it never contradicts the graph, and it repairs rather than refuses.
//!
//! Design: `docs/superpowers/specs/2026-09-07-plan-queue-design.md`.

pub mod model;

pub use model::{Queue, QueueError, Row};
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p plan --test model`
Expected: PASS, 4 tests.

- [ ] **Step 5: Commit**

```bash
git add crates/plan
git commit -m "feat(plan): the queue model, one ordered document"
```

---

### Task 2: The repair algorithm

**Files:**
- Create: `crates/plan/src/order.rs`
- Modify: `crates/plan/src/lib.rs`
- Test: `crates/plan/tests/order.rs`

**Interfaces:**
- Consumes: `plan::{Queue, Row}` from Task 1.
- Produces: `plan::order::Dependencies` (trait, one method `fn requires(&self, a: u32, b: u32) -> bool` — "a needs b, transitively"), and `Queue::move_row(&mut self, achievement: u32, to: usize, deps: &impl Dependencies) -> usize` returning the index the row actually landed at.

The rule, from the spec: rows that depend on the moved one gather **immediately below** it; rows it depends on gather **immediately above** it; everything else keeps its relative order. The landing index is `to.clamp(prerequisites_in_queue, rows - 1 - dependents_in_queue)` — dropping a row at the top when three of its prerequisites are queued asks for those three to sit above position zero, which is not a position.

A row the graph can't compute has no prerequisites and no dependents (`requires` answers false both ways), so it is never dragged and never drags.

- [ ] **Step 1: Write the failing test**

`crates/plan/tests/order.rs`:

```rust
use plan::order::Dependencies;
use plan::{Queue, Row};

/// "a requires b" from an explicit list of pairs, already transitive: the tests state the
/// relation they mean instead of deriving it, so a bug in the walk can't hide one here.
struct Deps(&'static [(u32, u32)]);

impl Dependencies for Deps {
    fn requires(&self, a: u32, b: u32) -> bool {
        self.0.contains(&(a, b))
    }
}

fn queue(ids: &[u32]) -> Queue {
    Queue::from_rows(
        ids.iter()
            .map(|a| Row {
                achievement: *a,
                wanted: true,
                origins: Vec::new(),
            })
            .collect(),
    )
}

fn ids(q: &Queue) -> Vec<u32> {
    q.rows().iter().map(|r| r.achievement).collect()
}

#[test]
fn with_no_dependencies_a_row_lands_exactly_where_it_was_dropped() {
    let deps = Deps(&[]);
    let mut q = queue(&[1, 2, 3, 4]);
    assert_eq!(q.move_row(4, 1, &deps), 1);
    assert_eq!(ids(&q), vec![1, 4, 2, 3]);
}

#[test]
fn moving_a_prerequisite_down_drags_what_needs_it() {
    // 3 requires 1. Dropping 1 at the bottom must not leave 3 above it.
    let deps = Deps(&[(3, 1)]);
    let mut q = queue(&[1, 2, 3]);
    let landed = q.move_row(1, 2, &deps);
    assert_eq!(landed, 1, "the clamp is symmetric: 3 has to stay below 1");
    assert_eq!(
        ids(&q),
        vec![2, 1, 3],
        "3 needs 1, so it follows it down instead of the move being refused"
    );
}

#[test]
fn moving_a_dependent_up_pulls_its_prerequisites_with_it_and_clamps() {
    // 4 requires 1 and 2. Dropping 4 at the top is impossible: two rows must precede it.
    let deps = Deps(&[(4, 1), (4, 2)]);
    let mut q = queue(&[1, 2, 3, 4]);
    let landed = q.move_row(4, 0, &deps);
    assert_eq!(landed, 2, "clamped to the number of prerequisites in the queue");
    assert_eq!(
        ids(&q),
        vec![1, 2, 4, 3],
        "the prerequisites sit contiguously above: what stopped the row is visible"
    );
}

#[test]
fn the_repair_is_transitive() {
    // 3 requires 2, 2 requires 1 — and the relation given here is already transitive.
    let deps = Deps(&[(2, 1), (3, 2), (3, 1)]);
    let mut q = queue(&[1, 2, 3, 4]);
    q.move_row(1, 3, &deps);
    assert_eq!(
        ids(&q),
        vec![4, 1, 2, 3],
        "moving 1 to the end drags the whole chain that hangs off it"
    );
}

#[test]
fn rows_with_no_relation_keep_their_relative_order() {
    let deps = Deps(&[(5, 1)]);
    let mut q = queue(&[1, 2, 3, 4, 5]);
    q.move_row(1, 4, &deps);
    assert_eq!(
        ids(&q),
        vec![2, 3, 4, 1, 5],
        "2, 3 and 4 are untouched by a constraint they are not part of"
    );
}

#[test]
fn a_row_the_graph_cannot_compute_is_never_dragged() {
    // 9 has no relation to anything: the graph doesn't know its prerequisites.
    let deps = Deps(&[(3, 1)]);
    let mut q = queue(&[1, 9, 3]);
    q.move_row(1, 2, &deps);
    assert_eq!(
        ids(&q),
        vec![9, 1, 3],
        "9 stays put: an unknown row carries no constraint in either direction"
    );
}

#[test]
fn moving_a_row_to_where_it_already_is_changes_nothing() {
    let deps = Deps(&[(3, 1)]);
    let mut q = queue(&[1, 2, 3]);
    let before = ids(&q);
    assert_eq!(q.move_row(2, 1, &deps), 1);
    assert_eq!(ids(&q), before);
}

#[test]
fn moving_a_row_that_is_not_in_the_queue_does_nothing() {
    let deps = Deps(&[]);
    let mut q = queue(&[1, 2]);
    assert_eq!(q.move_row(99, 0, &deps), 0);
    assert_eq!(ids(&q), vec![1, 2], "an absent row is not an error, it is a no-op");
}

#[test]
fn an_index_past_the_end_lands_on_the_last_position() {
    let deps = Deps(&[]);
    let mut q = queue(&[1, 2, 3]);
    assert_eq!(q.move_row(1, 99, &deps), 2);
    assert_eq!(ids(&q), vec![2, 3, 1]);
}

/// The property the readable cases above are examples of. Deterministic pseudo-random:
/// a failure has to be reproducible from the seed printed in the message.
#[test]
fn after_any_move_the_queue_never_contradicts_the_graph() {
    // 1 <- 2 <- 3 (transitive closure written out), 4 and 5 unconstrained.
    const PAIRS: &[(u32, u32)] = &[(2, 1), (3, 2), (3, 1)];
    let deps = Deps(PAIRS);
    let mut seed = 12345u64;
    let mut next = move || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (seed >> 33) as usize
    };
    for round in 0..500 {
        let mut q = queue(&[1, 2, 3, 4, 5]);
        for _ in 0..4 {
            let who = [1u32, 2, 3, 4, 5][next() % 5];
            let to = next() % 5;
            q.move_row(who, to, &deps);
        }
        let order = ids(&q);
        for (a, b) in PAIRS {
            let ia = order.iter().position(|x| x == a).expect("present");
            let ib = order.iter().position(|x| x == b).expect("present");
            assert!(
                ib < ia,
                "round {round}: {a} requires {b}, and {b} ended up below it: {order:?}"
            );
        }
        assert_eq!(order.len(), 5, "round {round}: a move lost or duplicated a row");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p plan --test order`
Expected: FAIL — `plan::order` does not exist.

- [ ] **Step 3: Write minimal implementation**

`crates/plan/src/order.rs`:

```rust
//! Moving a row, and repairing the order around it.
//!
//! The queue is yours to order, and the graph is the one thing it may not contradict. A
//! move is never refused: the rows that must yield are moved, and the moved row lands
//! where you asked whenever that is a position at all.

use crate::model::{Queue, Row};

/// "a requires b", transitively. The queue asks this and nothing else, which keeps this
/// crate independent of how the graph computes it — and lets the tests state the relation
/// they mean instead of deriving it.
pub trait Dependencies {
    fn requires(&self, a: u32, b: u32) -> bool;
}

impl Queue {
    /// Moves a row and returns the index it actually landed at.
    ///
    /// Rows that depend on it gather immediately below; rows it depends on gather
    /// immediately above; everything else keeps its relative order. The landing index is
    /// clamped so that the prerequisites have somewhere to be: dropping a row at the top
    /// with three prerequisites queued asks for three rows above position zero.
    pub fn move_row(&mut self, achievement: u32, to: usize, deps: &impl Dependencies) -> usize {
        let Some(from) = self.position(achievement) else {
            // Not in the queue: nothing to move, and not an error.
            return to.min(self.rows().len().saturating_sub(1));
        };
        let mut rows: Vec<Row> = self.rows().to_vec();
        let moved = rows.remove(from);

        let (mut above, mut below, mut free) = (Vec::new(), Vec::new(), Vec::new());
        for r in rows {
            if deps.requires(achievement, r.achievement) {
                above.push(r);
            } else if deps.requires(r.achievement, achievement) {
                below.push(r);
            } else {
                free.push(r);
            }
        }

        // `above` must all precede the row and `below` must all follow it, so the landing
        // index has a floor and a ceiling. Between them, the free rows decide.
        let total = above.len() + below.len() + free.len();
        let landed = to.clamp(above.len(), total - below.len());
        let free_above = landed - above.len();

        let mut out = Vec::with_capacity(total + 1);
        out.append(&mut above);
        out.extend(free.drain(..free_above));
        out.push(moved);
        out.append(&mut free);
        out.append(&mut below);
        *self = Queue::from_rows(out);
        landed
    }
}
```

Add `pub mod order;` and `pub use order::Dependencies;` to `crates/plan/src/lib.rs`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p plan --test order`
Expected: PASS, 10 tests including the property.

- [ ] **Step 5: Commit**

```bash
git add crates/plan
git commit -m "feat(plan): move a row and repair the order around it"
```

---

### Task 3: Adding and removing

**Files:**
- Create: `crates/plan/src/edit.rs`
- Modify: `crates/plan/src/lib.rs`
- Test: `crates/plan/tests/edit.rs`

**Interfaces:**
- Consumes: `plan::{Queue, Row, Dependencies}` from Tasks 1-2.
- Produces: `Queue::enqueue(&mut self, achievement: u32, chain: &[u32], deps: &impl Dependencies)` and `Queue::remove(&mut self, achievement: u32)`.

`chain` is the achievement's **missing** transitive prerequisites, in dependency order, as the caller read them from the graph. This crate does not talk to the graph: it is handed the chain, which is what keeps it testable without a catalog.

- [ ] **Step 1: Write the failing test**

`crates/plan/tests/edit.rs`:

```rust
use plan::order::Dependencies;
use plan::{Queue, Row};

struct Deps(&'static [(u32, u32)]);

impl Dependencies for Deps {
    fn requires(&self, a: u32, b: u32) -> bool {
        self.0.contains(&(a, b))
    }
}

fn ids(q: &Queue) -> Vec<u32> {
    q.rows().iter().map(|r| r.achievement).collect()
}

fn row_of(q: &Queue, achievement: u32) -> &Row {
    q.rows()
        .iter()
        .find(|r| r.achievement == achievement)
        .expect("row present")
}

#[test]
fn a_wish_goes_last_with_its_missing_steps_immediately_before_it() {
    let mut q = Queue::default();
    q.enqueue(41, &[7, 12], &Deps(&[(41, 7), (41, 12)]));
    assert_eq!(ids(&q), vec![7, 12, 41]);
    assert!(row_of(&q, 41).wanted, "you asked for it");
    assert!(!row_of(&q, 7).wanted, "it arrived as a step");
    assert_eq!(row_of(&q, 7).origins, vec![41]);
}

#[test]
fn a_second_wish_appends_after_the_first() {
    let deps = Deps(&[(41, 7), (512, 9)]);
    let mut q = Queue::default();
    q.enqueue(41, &[7], &deps);
    q.enqueue(512, &[9], &deps);
    assert_eq!(
        ids(&q),
        vec![7, 41, 9, 512],
        "a new wish is the lowest priority until you say otherwise"
    );
}

#[test]
fn a_step_already_queued_is_not_duplicated_and_gains_an_origin() {
    let deps = Deps(&[(41, 7), (512, 7)]);
    let mut q = Queue::default();
    q.enqueue(41, &[7], &deps);
    q.enqueue(512, &[7], &deps);
    assert_eq!(ids(&q), vec![7, 41, 512]);
    assert_eq!(
        row_of(&q, 7).origins,
        vec![41, 512],
        "one row, two reasons to be there"
    );
}

#[test]
fn enqueueing_something_already_wanted_leaves_it_where_it_is() {
    let deps = Deps(&[]);
    let mut q = Queue::default();
    q.enqueue(41, &[], &deps);
    q.enqueue(512, &[], &deps);
    q.enqueue(41, &[], &deps);
    assert_eq!(
        ids(&q),
        vec![41, 512],
        "asking twice is not a reason to reorder what you already arranged"
    );
}

#[test]
fn a_step_that_was_below_its_wish_is_repaired_into_place() {
    let deps = Deps(&[(512, 7)]);
    let mut q = Queue::from_rows(vec![Row {
        achievement: 7,
        wanted: true,
        origins: Vec::new(),
    }]);
    // 512 needs 7, and 7 is already in the queue: enqueueing 512 must not leave it above.
    q.enqueue(512, &[7], &deps);
    assert_eq!(ids(&q), vec![7, 512]);
    assert!(row_of(&q, 7).wanted, "it stays wanted, and gains an origin");
    assert_eq!(row_of(&q, 7).origins, vec![512]);
}

#[test]
fn removing_a_wish_takes_only_the_rows_left_with_no_reason_to_be_there() {
    let deps = Deps(&[(41, 7), (41, 12)]);
    let mut q = Queue::default();
    q.enqueue(41, &[7, 12], &deps);
    q.remove(41);
    assert_eq!(ids(&q), Vec::<u32>::new(), "its steps served nothing else");
}

#[test]
fn a_step_two_wishes_need_survives_the_removal_of_one() {
    let deps = Deps(&[(41, 7), (512, 7)]);
    let mut q = Queue::default();
    q.enqueue(41, &[7], &deps);
    q.enqueue(512, &[7], &deps);
    q.remove(41);
    assert_eq!(ids(&q), vec![7, 512]);
    assert_eq!(row_of(&q, 7).origins, vec![512]);
}

#[test]
fn a_step_you_also_asked_for_survives_the_removal_of_its_wish() {
    let deps = Deps(&[(41, 7)]);
    let mut q = Queue::default();
    q.enqueue(41, &[7], &deps);
    q.enqueue(7, &[], &deps); // you decide you want the step for itself too
    q.remove(41);
    assert_eq!(ids(&q), vec![7]);
    assert!(row_of(&q, 7).wanted);
    assert!(row_of(&q, 7).origins.is_empty());
}

#[test]
fn removing_something_absent_is_a_no_op() {
    let mut q = Queue::default();
    q.enqueue(41, &[], &Deps(&[]));
    q.remove(999);
    assert_eq!(ids(&q), vec![41]);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p plan --test edit`
Expected: FAIL — `enqueue` and `remove` do not exist.

- [ ] **Step 3: Write minimal implementation**

`crates/plan/src/edit.rs`:

```rust
//! Adding a wish and removing one. The chain arrives from the caller, already read from
//! the graph: this crate never asks the graph anything, which is what lets it be tested
//! without a catalog.

use crate::model::{Queue, Row};
use crate::order::Dependencies;

impl Queue {
    /// Adds `achievement` as a wish, with its missing prerequisites before it.
    ///
    /// A step already queued keeps its place and gains an origin; a wish already queued is
    /// left exactly where it is — asking twice is not a reason to undo an arrangement you
    /// made by hand.
    pub fn enqueue(&mut self, achievement: u32, chain: &[u32], deps: &impl Dependencies) {
        let mut rows = self.rows().to_vec();
        for step in chain {
            match rows.iter_mut().find(|r| r.achievement == *step) {
                Some(r) => {
                    if !r.origins.contains(&achievement) {
                        r.origins.push(achievement);
                    }
                }
                None => rows.push(Row {
                    achievement: *step,
                    wanted: false,
                    origins: vec![achievement],
                }),
            }
        }
        match rows.iter_mut().find(|r| r.achievement == achievement) {
            Some(r) => r.wanted = true,
            None => rows.push(Row {
                achievement,
                wanted: true,
                origins: Vec::new(),
            }),
        }
        *self = Queue::from_rows(rows);
        // The rows just appended are at the end in chain order; moving the wish onto its
        // own position runs the repair, which pulls its prerequisites above it and leaves
        // everything else alone.
        let at = self.position(achievement).unwrap_or(0);
        self.move_row(achievement, at, deps);
    }

    /// Removes a wish. Its steps go only if nothing else keeps them: another wish that
    /// needs them, or your having asked for them yourself.
    pub fn remove(&mut self, achievement: u32) {
        let mut rows = self.rows().to_vec();
        for r in rows.iter_mut() {
            r.origins.retain(|o| *o != achievement);
        }
        rows.retain(|r| r.achievement != achievement);
        rows.retain(|r| !r.is_orphan());
        *self = Queue::from_rows(rows);
    }
}
```

Add `pub mod edit;` to `crates/plan/src/lib.rs`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p plan`
Expected: PASS, the whole `plan` suite.

- [ ] **Step 5: Commit**

```bash
git add crates/plan
git commit -m "feat(plan): add a wish with its chain, and remove it without orphaning"
```

---

### Task 4: Persistence

**Files:**
- Modify: `crates/store/src/migrations.rs`, `crates/store/src/lib.rs`, `crates/store/Cargo.toml`
- Test: `crates/store/tests/queue.rs`

**Interfaces:**
- Consumes: `plan::{Queue, QueueError}` from Task 1.
- Produces: `Store::queue() -> Result<Result<Queue, QueueError>, StoreError>` and `Store::set_queue(&Queue) -> Result<(), StoreError>`.

The nested `Result` is deliberate and mirrors `goals()`: the outer one is "the database failed", the inner is "the document didn't parse". They are different situations and the Plan screen says different things about them.

Migration 2 creates a one-row table holding the document. It **seeds nothing**: seeding means resolving a `TargetKey` to the achievement that unlocks it, and `store` has no catalog — it does not even depend on the crate. The import is Task 6's explicit command.

- [ ] **Step 1: Write the failing test**

`crates/store/tests/queue.rs`:

```rust
use plan::{Queue, Row};
use store::Store;
use tempfile::tempdir;

fn row(achievement: u32, wanted: bool, origins: &[u32]) -> Row {
    Row {
        achievement,
        wanted,
        origins: origins.to_vec(),
    }
}

#[test]
fn a_queue_survives_reopening() {
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("isaacdome.db");
    {
        let s = Store::open(&path).expect("opens");
        s.set_queue(&Queue::from_rows(vec![row(7, false, &[41]), row(41, true, &[])]))
            .expect("writes");
    }
    let s = Store::open(&path).expect("reopens");
    let q = s.queue().expect("query").expect("document parses");
    assert_eq!(q.rows().len(), 2);
    assert_eq!(q.position(41), Some(1), "the order is what was written");
}

#[test]
fn a_fresh_database_has_an_empty_queue_and_that_is_not_an_error() {
    let dir = tempdir().expect("temp dir");
    let s = Store::open(&dir.path().join("isaacdome.db")).expect("opens");
    let q = s.queue().expect("query").expect("parses");
    assert_eq!(q.rows().len(), 0);
}

#[test]
fn a_document_that_does_not_parse_is_declared_not_flattened_to_empty() {
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("isaacdome.db");
    let s = Store::open(&path).expect("opens");
    s.set_queue(&Queue::default()).expect("writes");
    s.__corrupt_queue_for_tests("{ not json").expect("writes garbage");
    let inner = s.queue().expect("the query itself works");
    assert!(
        inner.is_err(),
        "an unreadable queue must not read as an empty one"
    );
}

#[test]
fn writing_twice_replaces_the_document_instead_of_appending_a_second_one() {
    let dir = tempdir().expect("temp dir");
    let s = Store::open(&dir.path().join("isaacdome.db")).expect("opens");
    s.set_queue(&Queue::from_rows(vec![row(1, true, &[])]))
        .expect("writes");
    s.set_queue(&Queue::from_rows(vec![row(2, true, &[])]))
        .expect("writes again");
    let q = s.queue().expect("query").expect("parses");
    assert_eq!(q.rows().len(), 1);
    assert_eq!(q.rows()[0].achievement, 2);
}

#[test]
fn the_schema_version_moved_to_two() {
    let dir = tempdir().expect("temp dir");
    let s = Store::open(&dir.path().join("isaacdome.db")).expect("opens");
    assert_eq!(s.schema_version().expect("reads"), 2);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p store --test queue`
Expected: FAIL — `Store::queue`, `Store::set_queue` and the test helper do not exist.

- [ ] **Step 3: Write minimal implementation**

In `crates/store/Cargo.toml`, add `plan = { path = "../plan" }` to `[dependencies]`.

In `crates/store/src/migrations.rs`, raise `SCHEMA_VERSION` to `2` and append to `MIGRATIONS`:

```rust
    // 2: the plan queue, as one JSON document rather than a row per achievement. The
    // order is the position in the array, so there is no sequence column that could
    // contradict itself. One row, pinned by the CHECK: a second document would be a
    // second answer to "what is the order".
    "CREATE TABLE plan_queue (
        id INTEGER PRIMARY KEY CHECK (id = 1),
        rows_json TEXT NOT NULL
    );",
```

The array in `MIGRATIONS` becomes `[&str; 2]`.

In `crates/store/src/lib.rs`:

```rust
    /// The queue, as it was written. The outer `Result` is "the database failed"; the
    /// inner one is "the document didn't parse". They are different situations — one is a
    /// broken file, the other a plan written by a version that knew more — and the screen
    /// says different things about them.
    pub fn queue(&self) -> Result<Result<plan::Queue, plan::QueueError>, StoreError> {
        let found: Option<String> = self
            .conn
            .query_row("SELECT rows_json FROM plan_queue WHERE id = 1", [], |r| {
                r.get(0)
            })
            .optional()
            .map_err(StoreError::from_sqlite)?;
        // No row yet is an empty queue, not a failure: a fresh database has no plan.
        Ok(match found {
            Some(json) => plan::Queue::from_json(&json),
            None => Ok(plan::Queue::default()),
        })
    }

    /// Replaces the document. The whole order is one value, so a write is one statement
    /// and there is no half-applied reorder to recover from.
    pub fn set_queue(&self, q: &plan::Queue) -> Result<(), StoreError> {
        self.conn
            .execute(
                "INSERT INTO plan_queue (id, rows_json) VALUES (1, ?1)
                 ON CONFLICT(id) DO UPDATE SET rows_json = excluded.rows_json",
                params![q.to_json()],
            )
            .map(|_| ())
            .map_err(StoreError::from_sqlite)
    }

    /// Writes a document straight in, for the test that an unreadable queue is declared
    /// rather than flattened to an empty one. There is no other way to produce that state
    /// through the public API, which is the point.
    pub fn __corrupt_queue_for_tests(&self, raw: &str) -> Result<(), StoreError> {
        self.conn
            .execute(
                "INSERT INTO plan_queue (id, rows_json) VALUES (1, ?1)
                 ON CONFLICT(id) DO UPDATE SET rows_json = excluded.rows_json",
                params![raw],
            )
            .map(|_| ())
            .map_err(StoreError::from_sqlite)
    }
```

`optional()` comes from `rusqlite::OptionalExtension`; add the import.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p store`
Expected: PASS — the new file plus the existing `goals` tests, which migration 2 must not disturb.

- [ ] **Step 5: Commit**

```bash
git add crates/store
git commit -m "feat(store): migration 2, the plan queue as one document"
```

---

### Task 5: The view-models

**Files:**
- Create: `crates/ipc/src/queue.rs`
- Modify: `crates/ipc/src/lib.rs`, `crates/ipc/Cargo.toml`
- Test: `crates/ipc/tests/queue.rs`

**Interfaces:**
- Consumes: `plan::Queue`, `graph::{Graph, evaluate::Eval}`, `ipc::UnlockNode`.
- Produces: `ipc::{QueueView, QueueRow, QueueDiagnostic}` and `ipc::queue_view(...) -> QueueView`.

Eight parameters would trip `clippy::too_many_arguments` (the threshold is seven), and a
list that long is hard to call correctly anyway. They travel as one struct:

```rust
pub struct QueueInputs<'a> {
    pub catalog: Option<&'a Catalog>,
    pub flags: Option<&'a [bool]>,
    pub graph: Option<&'a graph::Graph>,
    pub eval: Option<&'a graph::evaluate::Eval>,
    pub queue: Result<&'a plan::Queue, &'a plan::QueueError>,
    pub goals_pending: u32,
    /// `Some` when the database itself failed: the text is ours, never SQLite's.
    pub store_reason: Option<String>,
}

pub fn queue_view(inputs: QueueInputs<'_>, icon: impl FnMut(&str) -> Option<Vec<u8>>) -> QueueView
```

- [ ] **Step 1: Write the failing test**

`crates/ipc/tests/queue.rs`:

```rust
use ipc::{QueueDiagnostic, QueueView};
use serde_json::{json, to_value};

#[test]
fn queue_row_and_diagnostic_shapes_are_pinned() {
    assert_eq!(
        to_value(QueueDiagnostic::Completed {
            count: 2,
            wanted: vec![41]
        })
        .expect("serializes"),
        json!({ "kind": "completed", "count": 2, "wanted": [41] })
    );
    assert_eq!(
        to_value(QueueDiagnostic::Unreadable).expect("serializes"),
        json!({ "kind": "unreadable" }),
        "unreadable and empty are different things, and the wire says which"
    );
    assert_eq!(
        to_value(QueueDiagnostic::GoalsPending { count: 3 }).expect("serializes"),
        json!({ "kind": "goalsPending", "count": 3 })
    );
    assert_eq!(
        to_value(QueueDiagnostic::Unresolved { achievement: 900 }).expect("serializes"),
        json!({ "kind": "unresolved", "achievement": 900 })
    );
}

#[test]
fn an_unreadable_queue_is_empty_and_says_so() {
    let err = plan::QueueError::Unreadable {
        reason: "expected value".into(),
    };
    let v: QueueView = ipc::queue_view(None, None, None, None, Err(&err), 0, None, |_| None);
    assert!(v.rows.is_empty());
    assert!(v.diagnostics.contains(&QueueDiagnostic::Unreadable));
    assert!(
        v.store_available,
        "the database opened fine: it is the document that didn't parse"
    );
}

#[test]
fn a_store_that_will_not_open_is_a_different_case_from_an_unreadable_document() {
    let q = plan::Queue::default();
    let v = ipc::queue_view(
        None,
        None,
        None,
        None,
        Ok(&q),
        0,
        Some("database from a newer version (7 > 2)".into()),
        |_| None,
    );
    assert!(!v.store_available);
    assert!(v
        .diagnostics
        .iter()
        .any(|d| matches!(d, QueueDiagnostic::StoreUnavailable { .. })));
}

#[test]
fn a_completed_row_leaves_the_view_and_is_reported() {
    let c = catalog_with_achievements();
    let q = plan::Queue::from_rows(vec![
        plan::Row { achievement: 1, wanted: true, origins: vec![] },
        plan::Row { achievement: 2, wanted: false, origins: vec![1] },
    ]);
    // Slot 1 is done, slot 2 is not.
    let flags = [false, true, false];
    let v = ipc::queue_view(Some(&c), Some(&flags), None, None, Ok(&q), 0, None, |_| None);
    assert_eq!(
        v.rows.iter().map(|r| r.wanted).collect::<Vec<_>>(),
        vec![false],
        "the done row is gone from the view"
    );
    assert!(
        v.diagnostics.contains(&QueueDiagnostic::Completed {
            count: 1,
            wanted: vec![1]
        }),
        "a row that vanishes without a word is a bug: {:?}",
        v.diagnostics
    );
}

#[test]
fn a_row_the_catalog_no_longer_knows_stays_and_is_declared() {
    let c = catalog_with_achievements();
    let q = plan::Queue::from_rows(vec![plan::Row {
        achievement: 900,
        wanted: true,
        origins: vec![],
    }]);
    let flags = [false, false, false];
    let v = ipc::queue_view(Some(&c), Some(&flags), None, None, Ok(&q), 0, None, |_| None);
    assert!(v
        .diagnostics
        .contains(&QueueDiagnostic::Unresolved { achievement: 900 }));
}

use catalog::Catalog;

const ITEMS: &[u8] = b"<items gfxroot=\"gfx/items/\"><passive id=\"2\" gfx=\"a.png\" name=\"A\" achievement=\"1\" /></items>";
const ACH: &[u8] = b"<achievements gfxroot=\"gfx/ui/achievement/\"><achievement id=\"1\" text=\"t1\" gfx=\"1.png\" /><achievement id=\"2\" text=\"t2\" gfx=\"2.png\" /></achievements>";

fn catalog_with_achievements() -> Catalog {
    Catalog::build(|p| match p {
        "items.xml" => Some(ITEMS.to_vec()),
        "achievements.xml" => Some(ACH.to_vec()),
        _ => None,
    })
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p ipc --test queue`
Expected: FAIL — `ipc::queue_view` and the types do not exist.

- [ ] **Step 3: Write minimal implementation**

Add `plan = { path = "../plan" }` to `crates/ipc/Cargo.toml`.

`crates/ipc/src/queue.rs` defines:

```rust
//! The plan queue as the UI sees it: rows already resolved to nodes, and every reason a
//! row is missing said out loud.

use catalog::{AchievementId, Catalog};
use serde::Serialize;

use crate::graph::{unlock_view, UnlockNode};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueView {
    pub rows: Vec<QueueRow>,
    pub diagnostics: Vec<QueueDiagnostic>,
    /// `false` when `store` won't open: the queue can't be seen or changed, and the UI
    /// says so instead of showing an empty list.
    pub store_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueRow {
    /// The same node the Unlock screen draws, so the two can never disagree.
    pub node: UnlockNode,
    pub wanted: bool,
    /// The wanted achievements whose chain passes through this row.
    pub origins: Vec<u32>,
    /// Prerequisites this row still needs that are **not** in the queue.
    pub steps_not_queued: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum QueueDiagnostic {
    StoreUnavailable { reason: String },
    /// The saved document didn't parse: the queue is empty because it couldn't be read,
    /// which is not the same as being empty.
    Unreadable,
    /// Rows left out of this read because the profile has completed them.
    Completed { count: u32, wanted: Vec<u32> },
    /// A row whose achievement this catalog no longer knows. It stays, by id.
    Unresolved { achievement: u32 },
    /// Goals saved before the queue existed, not yet imported. One action moves them in;
    /// nothing happens on its own, because a read never writes.
    GoalsPending { count: u32 },
    NoCatalog,
}
```

```rust
pub fn queue_view(inputs: QueueInputs<'_>, icon: impl FnMut(&str) -> Option<Vec<u8>>) -> QueueView {
    let QueueInputs {
        catalog,
        flags,
        graph,
        eval,
        queue,
        goals_pending,
        store_reason,
    } = inputs;
    let mut diagnostics = Vec::new();
    let store_available = store_reason.is_none();
    if let Some(reason) = store_reason {
        diagnostics.push(QueueDiagnostic::StoreUnavailable { reason });
    }
    if goals_pending > 0 {
        diagnostics.push(QueueDiagnostic::GoalsPending {
            count: goals_pending,
        });
    }
    let queue = match queue {
        Ok(q) => q,
        Err(_) => {
            // Unreadable is not empty, and the difference is the whole point of saying it.
            diagnostics.push(QueueDiagnostic::Unreadable);
            return QueueView {
                rows: Vec::new(),
                diagnostics,
                store_available,
            };
        }
    };
    let Some(c) = catalog else {
        diagnostics.push(QueueDiagnostic::NoCatalog);
        return QueueView {
            rows: Vec::new(),
            diagnostics,
            store_available,
        };
    };

    // One `unlock_view`, indexed by achievement: a queue row shows **the same node** the
    // Unlock screen shows, so the two can never drift apart.
    let view = unlock_view(Some(c), flags, graph, eval, icon);
    let by_id: std::collections::BTreeMap<u32, &UnlockNode> = view
        .nodes
        .iter()
        .filter_map(|n| match &n.achievement {
            crate::graph::AchievementRef::Known { id, .. } => Some((*id, n)),
            crate::graph::AchievementRef::Unknown { .. } => None,
        })
        .collect();
    let queued: std::collections::BTreeSet<u32> =
        queue.rows().iter().map(|r| r.achievement).collect();

    let mut rows = Vec::new();
    let mut completed = Vec::new();
    let mut completed_wanted = Vec::new();
    for r in queue.rows() {
        let Some(node) = by_id.get(&r.achievement) else {
            // The catalog no longer knows it — an older edition, or a patch that removed
            // it. The row stays in the file and gets named; it never turns into a
            // different row and never vanishes without a word.
            diagnostics.push(QueueDiagnostic::Unresolved {
                achievement: r.achievement,
            });
            continue;
        };
        if node.done {
            completed.push(r.achievement);
            if r.wanted {
                completed_wanted.push(r.achievement);
            }
            continue;
        }
        let steps_not_queued = graph
            .and_then(|g| flags.map(|f| g.missing_chain(r.achievement, Some(f))))
            .map(|chain| chain.iter().filter(|id| !queued.contains(id)).count() as u32)
            .unwrap_or(0);
        rows.push(QueueRow {
            node: (*node).clone(),
            wanted: r.wanted,
            origins: r.origins.clone(),
            steps_not_queued,
        });
    }
    if !completed.is_empty() {
        diagnostics.push(QueueDiagnostic::Completed {
            count: completed.len() as u32,
            wanted: completed_wanted,
        });
    }
    QueueView {
        rows,
        diagnostics,
        store_available,
    }
}
```

Note the two `continue` branches: a row never disappears silently. One leaves through
`Unresolved`, the other through `Completed`, and both are on the wire.

Export the three types and `queue_view` from `crates/ipc/src/lib.rs`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p ipc`
Expected: PASS, the whole `ipc` suite including the existing graph tests.

- [ ] **Step 5: Commit**

```bash
git add crates/ipc
git commit -m "feat(ipc): the plan queue view-model"
```

---

### Task 6: The commands

**Files:**
- Modify: `crates/app/src/lib.rs`, `crates/app/Cargo.toml`

**Interfaces:**
- Consumes: everything above.
- Produces: `queue`, `queue_add`, `queue_remove`, `queue_move`, `queue_import_goals`, all returning `Result<ipc::QueueView, IpcError>`.

Every mutation returns the whole view: the queue is small, a move can reorder much of it, and handing back the new truth is cheaper to reason about than teaching the frontend to replay the repair.

The `Dependencies` implementation lives here, over the graph:

```rust
/// "a requires b" for the queue, read from the graph's transitive prerequisites.
///
/// The chains are computed **once, for the rows in the queue**, and not per question: a
/// move asks `requires` twice per row, and each answer would otherwise be a fresh
/// transitive walk over 637 nodes.
///
/// A node the graph can't compute has an empty chain, so it is never dragged and never
/// drags — which is Decision 3 of the spec, expressed once, here.
struct GraphDeps {
    chains: std::collections::BTreeMap<u32, std::collections::BTreeSet<u32>>,
}

impl GraphDeps {
    fn new(g: &graph::Graph, flags: Option<&[bool]>, rows: &[u32]) -> GraphDeps {
        GraphDeps {
            chains: rows
                .iter()
                .map(|a| (*a, g.missing_chain(*a, flags).into_iter().collect()))
                .collect(),
        }
    }
}

impl plan::Dependencies for GraphDeps {
    fn requires(&self, a: u32, b: u32) -> bool {
        self.chains.get(&a).is_some_and(|c| c.contains(&b))
    }
}
```

Build it from the ids currently in the queue, plus the one being added:
`GraphDeps::new(g, flags.as_deref(), &ids)`.

- [ ] **Step 1: Add the graph API this needs, with its test**

The commands need "the missing transitive prerequisites of X", which `evaluate` computes
internally today and does not expose. Add it to `crates/graph/src/evaluate.rs`:

```rust
impl Graph {
    /// The not-done achievements standing between the profile and this node, transitively.
    /// Empty when the node is done, when it is available now, or when the graph can't say —
    /// and the caller can tell those apart from `NodeInfo`.
    pub fn missing_chain(&self, achievement: u32, flags: Option<&[bool]>) -> Vec<u32> {
        let Some(flags) = flags else {
            return Vec::new();
        };
        let done = |id: u32| flags.get(id as usize).copied().unwrap_or(false);
        let mut memo = BTreeMap::new();
        let mut stack = Vec::new();
        let mut cycles = Vec::new();
        transitive(self, achievement, &done, &mut memo, &mut stack, &mut cycles)
            .map(|set| set.into_iter().collect())
            .unwrap_or_default()
    }
}
```

Test in `crates/graph/tests/evaluate.rs`:

```rust
#[test]
fn the_missing_chain_is_what_still_stands_between_you_and_a_node() {
    let g = graph(&[(1, &[]), (2, &[1]), (3, &[2])], &[]);
    assert_eq!(g.missing_chain(3, Some(&flags(&[], 4))), vec![1, 2]);
    assert_eq!(
        g.missing_chain(3, Some(&flags(&[1], 4))),
        vec![2],
        "what is done is not owed again"
    );
    assert!(g.missing_chain(1, Some(&flags(&[], 4))).is_empty());
    assert!(
        g.missing_chain(3, None).is_empty(),
        "without section 1 there is nothing to compute, and nothing is claimed"
    );
}
```

Run: `cargo test -p graph --test evaluate` — red, then green.

- [ ] **Step 2: Write the commands**

Add `plan = { path = "../plan" }` to `crates/app/Cargo.toml`. Each command follows the
shape the existing `plan` command already uses: open the store through `StoreState`, map a
failure to a reason with `store_reason`, and never turn an expected failure into `Err`.

```rust
#[tauri::command]
fn queue_move(
    app: AppHandle,
    achievement: u32,
    to: usize,
    store: tauri::State<'_, StoreState>,
    catalog: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    graph: tauri::State<'_, GraphState>,
) -> Result<ipc::QueueView, IpcError> {
    let Some(store) = store.get() else {
        return Err(IpcError::StoreUnavailable);
    };
    let mut q = match store.queue() {
        Ok(Ok(q)) => q,
        // A document that won't parse must not be silently replaced by a moved version of
        // an empty one: the move is refused as a store failure, and the read explains.
        Ok(Err(_)) | Err(_) => return Err(IpcError::StoreUnavailable),
    };
    let flags = achievement_flags(&app)?;
    let rs = resources.get();
    let cat = rs.and_then(|rs| catalog.get_or_build(rs));
    if let Some(g) = cat.and_then(|c| graph.get(c)) {
        let ids: Vec<u32> = q.rows().iter().map(|r| r.achievement).collect();
        let deps = GraphDeps::new(g, flags.as_deref(), &ids);
        q.move_row(achievement, to, &deps);
        store.set_queue(&q).map_err(|_| IpcError::StoreUnavailable)?;
    }
    queue_command(app, store, catalog, resources, graph)
}
```

`queue_remove` is `q.remove(achievement)` between the same read and write. `queue_add` and
`queue_import_goals` carry the only real logic:

```rust
// queue_add, once the store, catalog and graph are in hand:
let chain = g.missing_chain(achievement, flags.as_deref());
let mut ids: Vec<u32> = q.rows().iter().map(|r| r.achievement).collect();
ids.push(achievement);
ids.extend(chain.iter().copied());
let deps = GraphDeps::new(g, flags.as_deref(), &ids);
q.enqueue(achievement, &chain, &deps);
store.set_queue(&q).map_err(|_| IpcError::StoreUnavailable)?;
```

```rust
// queue_import_goals: the one-off move from the old goals table. A goal is a target; the
// queue holds achievements, so each target is resolved through the catalog's reverse
// index to the achievement that unlocks it. A target nothing unlocks is skipped rather
// than guessed at — and the goals table is left untouched, so the step is repeatable and
// reversible.
let goals = match store.goals() {
    Ok(g) => g.goals,
    Err(_) => return Err(IpcError::StoreUnavailable),
};
for goal in &goals {
    let Some(achievement) = achievement_unlocking(c, &goal.target) else {
        continue;
    };
    let chain = g.missing_chain(achievement, flags.as_deref());
    let mut ids: Vec<u32> = q.rows().iter().map(|r| r.achievement).collect();
    ids.push(achievement);
    ids.extend(chain.iter().copied());
    let deps = GraphDeps::new(g, flags.as_deref(), &ids);
    q.enqueue(achievement, &chain, &deps);
}
store.set_queue(&q).map_err(|_| IpcError::StoreUnavailable)?;
```

`achievement_unlocking(catalog, &TargetKey) -> Option<u32>` is the inverse of the reverse
index: it walks `Catalog::achievements` and returns the first whose `unlocks` contains
that target. It belongs in `ipc` next to `resolve_target`, which already turns a
`TargetKey` into what the catalog knows about it, and it gets a test there against the
fixture catalog.

`queue` reads and returns without writing. `goals_pending` for the view is
`store.goals()`'s count while the queue holds no row for any of them.

- [ ] **Step 3: Verify by hand**

Run: `pnpm dev`
Expected: the window opens. This crate is wiring and is not unit-tested (`CLAUDE.md`); the
logic it calls is covered in `plan`, `store`, `graph` and `ipc`.

- [ ] **Step 4: Commit**

```bash
git add crates/app crates/graph
git commit -m "feat(app): the plan queue commands"
```

---

### Task 7: Real data, the TypeScript mirror, and the documents

**Files:**
- Create: `crates/plan/tests/real_data.rs`
- Modify: `ui/src/lib/ipc/types.ts`, `docs/STATO.md`, `CLAUDE.md`
- Create: `docs/superpowers/plans/2026-09-07-plan-queue-report.md`

- [ ] **Step 1: Write the real-data test**

`crates/plan/tests/real_data.rs` builds the real graph and the most recent dated save
through `test-support` (copy the `support` module from `crates/graph/tests/support/mod.rs`,
which already declares its sample or its skip), then:

```rust
#[test]
fn enqueueing_a_real_achievement_queues_exactly_its_missing_chain() {
    let Some((g, flags)) = support::real_graph_and_flags() else {
        return;
    };
    // The node with the deepest chain on this profile: the case that exercises the walk.
    let deepest = g
        .nodes()
        .iter()
        .map(|n| (g.missing_chain(n.achievement, Some(&flags)).len(), n.achievement))
        .max()
        .expect("the catalog is not empty");
    eprintln!("deepest chain on this profile: {} steps for node {}", deepest.0, deepest.1);
    let chain = g.missing_chain(deepest.1, Some(&flags));
    let mut q = plan::Queue::default();
    q.enqueue(deepest.1, &chain, &GraphDeps { graph: &g, flags: Some(&flags) });
    assert_eq!(
        q.rows().len(),
        chain.len() + 1,
        "the wish and its chain, nothing else"
    );
    assert!(
        q.rows().iter().all(|r| !flags.get(r.achievement as usize).copied().unwrap_or(false)),
        "a chain must never contain something already done"
    );
    let order: Vec<u32> = q.rows().iter().map(|r| r.achievement).collect();
    for (i, id) in order.iter().enumerate() {
        for prereq in g.missing_chain(*id, Some(&flags)) {
            if let Some(j) = order.iter().position(|x| *x == prereq) {
                assert!(j < i, "{prereq} must come before {id}, got {order:?}");
            }
        }
    }
}
```

`GraphDeps` is duplicated in this test file, three lines, rather than exported from `app`,
which is the Tauri crate and not a dependency of anything.

Run: `cargo test -p plan -- --nocapture` — the sample line has to appear.

- [ ] **Step 2: Mirror the types in TypeScript**

In `ui/src/lib/ipc/types.ts`, add `QueueView`, `QueueRow` and `QueueDiagnostic`, mirroring
the Rust exactly — `QueueDiagnostic` is a tagged union with `kind`, and `steps_not_queued`
becomes `stepsNotQueued`. Add typed wrappers in `ui/src/lib/ipc/queue.ts` for the five
commands: no component ever calls `invoke` (rule 3 of the frontend conventions).

Run: `pnpm typecheck && pnpm lint && pnpm scan`.

- [ ] **Step 3: Write the report and update the state**

The report says what execution found that this plan didn't know, with numbers: how deep the
deepest chain on a real profile is, how many rows a typical wish adds, and anything the
repair algorithm did that surprised its author. In `docs/STATO.md`: the M3 line, the session
entry. In `CLAUDE.md`: the `plan` row in the modules table, and `store` gains "migration 2:
the plan queue".

- [ ] **Step 4: Verify**

Run: `pnpm check`
Expected: green.

- [ ] **Step 5: Commit**

```bash
git add crates docs ui CLAUDE.md
git commit -m "docs: close the plan queue, with its real-data test and the TypeScript mirror"
```
