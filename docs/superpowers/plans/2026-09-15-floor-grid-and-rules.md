# Floor F1 — the painted grid and the cited rules

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** a screen where you paint the floor's minimap on a 13x13 grid and it lights the cells the game's own documented placement rules allow for the Secret, Super Secret and Ultra Secret rooms, each lit cell able to name the rule and quote its source.

**Architecture:** a new pure crate `crates/floor` holds the grid, the rules (a JSON file embedded with `include_str!`, one quotation and one URL per rule) and the solver; `crates/ipc` turns its answer into a view-model; one Tauri command is wiring; the screen keeps the painted grid in a Pinia store and asks the command for the ranking. Nothing here reads a file at runtime and nothing here touches the run archive, so F1 cannot break M4.

**Tech Stack:** Rust 2021 (workspace crates, `serde`, `serde_json`, `ts-rs`), Tauri 2, Vue 3 + TypeScript, Tailwind v4 tokens, Pinia, vue-i18n, Vitest.

**Spec:** `docs/superpowers/specs/2026-09-15-floor-secret-rooms-design.md` — read it first. This plan argues from it and does not repeat its reasoning.

## Global Constraints

- **Read-only, no network, no keys.** Nothing in F1 opens a save or talks to the network at runtime. The wiki is read **once, by a human or an agent, during Task 1**, and what it says lands in a committed JSON file.
- **No `panic!`, no `unwrap()`, no `expect()` outside tests.**
- **No `_ =>` arm on a closed enum.** Adding a variant must break the build.
- **Every struct crossing the IPC** carries `#[serde(rename_all = "camelCase")]`. Every enum with struct variants carries `#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]`. **Fieldless enums are bare camelCase strings** — no tag — exactly like `ItemKindView` and `StepsBasis`.
- **Tauri commands return `Result<T, IpcError>`**, never `Result<T, String>`.
- **`ui/src/lib/ipc/types.ts` is generated.** Never edit it by hand; run `pnpm ipc:types`. `scripts/check` fails when it disagrees with the Rust types.
- **A doc comment on a wire type becomes JSDoc in that generated file**, so it is under `pnpm scan`: no arrows, no check marks, no glyph the app's font lacks.
- **Frontend, the five rules:** no `<style>` in SFCs; no hardcoded visual constant (no `w-[48px]`, no `opacity-50`, no `:size="16"`) — every value is a token in `@theme`; no `invoke()` in components, only wrappers under `ui/src/lib/ipc/`; no raw `<button>`/`<input>`, use `ui/src/components/ui/`; no string unions — `const X = { … } as const`.
- **No visible string in a template.** Everything goes through `t()`, in **both** `ui/src/i18n/messages/it.ts` and `en.ts`.
- **Test-first.** The expected value comes from the spec or from Task 1's report, **never** from the code's current output.
- **Commits:** Conventional Commits, `type(scope): subject`, English, atomic. Scope is the crate or package (`floor`, `ipc`, `app`, `ui`); drop it for repo-wide `docs:`. **Never** a `Co-Authored-By` trailer or any mention of Claude.
- **Stage by explicit path.** Other sessions edit this tree in parallel; `git add -A` is forbidden here.
- **Before calling anything done:** `pnpm check`. To see what skipped: `cargo test --workspace -- --nocapture`.
- **Attribution.** Task 1 quotes a CC BY-SA 4.0 wiki. Every quotation that lands in `placement.json` carries its page URL and the date it was read, and the file carries the licence line. This is a licence obligation, not a nicety.

### One refinement of the spec, decided here

§3 of the spec names a `Tier` on a candidate. This plan does **not** create a `Tier` enum, because its variants could only be invented names for probability bands ("likely", "rare") and the repo's own rule is not to name something from a guess. A candidate carries instead:

- `neighbours: u8` — how many painted rooms touch the cell, a count, not a word;
- `rank: u8` — its position in the preference order the cited rule states (`0` is that rule's first preference).

The screen colours by `rank` and can always show the count. Amend §3 of the spec in Task 1's commit so the two documents do not disagree.

---

### Task 1: The rules, read from the wiki and written down with their sources

No code. The output is a report, and it is what every later test's expected value is read from. **What this pass cannot quote does not become a rule** — it becomes a line in the report's "could not source" section and, later, an `Unmodelled` entry.

**Files:**
- Create: `docs/superpowers/reports/2026-09-15-secret-room-rules.md`
- Modify: `docs/superpowers/specs/2026-09-15-floor-secret-rooms-design.md` (§3, the `Tier` amendment above)

**Interfaces:**
- Consumes: nothing.
- Produces: a table of rules, each with `id`, `target`, the constraint in one sentence, the **verbatim quotation**, the page URL, and the date read. Task 4 transcribes this table into JSON; Tasks 5 and 9 read their expected values from it.

- [ ] **Step 1: Read the three wiki pages**

Read, in this order, and keep the raw wording:

```
https://bindingofisaacrebirth.wiki.gg/wiki/Secret_Room
https://bindingofisaacrebirth.wiki.gg/wiki/Super_Secret_Room
https://bindingofisaacrebirth.wiki.gg/wiki/Ultra_Secret_Room
```

This is the same wiki `dataset/ATTRIBUTION.md` already names (bindingofisaacrebirth.wiki.gg, CC BY-SA 4.0), so the licence question is already settled for the repo — follow it, do not invent a second answer.

A fetch on 2026-09-15 while writing this plan returned these sentences from the first page. **Re-read them rather than trusting this list**; it is here so you can tell a page that changed from a page you misread:

- *"Regular Secret Rooms are usually located next to 3 or 4 rooms, while Super Secret Rooms can only be next to one room."*
- *"Secret Rooms are equally as likely to be in a valid location with 3 neighbors, as it is with 4 neighbors. 2 neighbor locations are rare but possible, even when there are locations with 3+ neighbors available."*
- *"Secret Rooms can exist next to all types of rooms except Boss Rooms, Super Secret Rooms, and other Secret Rooms."*
- *"Entrances to Secret Rooms will never have rocks or gaps in the way."*
- *"Super Secret Rooms are only located next to one other room, and this room can't be a Special Room; in other words, it is placed on one of the floor's dead ends."*
- *"Super Secret Rooms replace the dead-end room that would require the 2nd most rooms walked through from the start room to access."*
- *"Ultra Secret rooms are most likely generated in spots that connect to 3+ non-red rooms through its adjacent red rooms."*
- *"a specific 3+ room location is 11.5x more likely than a specific 2 room location."*

- [ ] **Step 2: Write the report**

Create `docs/superpowers/reports/2026-09-15-secret-room-rules.md` with exactly these four sections.

**§1 What was read** — the three URLs, the date, and the licence line (CC BY-SA 4.0, attribution to The Binding of Isaac: Rebirth Wiki), phrased the way `dataset/ATTRIBUTION.md` phrases it.

**§2 The rules** — a table, one row per constraint, with these columns and no others:

| id | target | constraint, in one sentence | quotation | url |
|---|---|---|---|---|

Give every row an id of the form `<target>-<what>`: `secret-neighbours`, `secret-forbidden-neighbours`, `super-secret-dead-end`, `super-secret-neighbour-not-special`, `super-secret-second-longest`, `ultra-secret-connections`. **A row with an empty quotation column is not a rule** — move it to §3.

**§3 What could not be sourced** — every claim you expected to find and did not, and every sentence you found but cannot evaluate on a grid of painted cells. From the fetch above, at least these two belong here, and say why:

- *"Entrances to Secret Rooms will never have rocks or gaps in the way"* — a fact about the room's contents, which a painted minimap does not carry.
- *"3+ non-red rooms through its adjacent red rooms"* — "red rooms" is the Red Key mechanic; a grid of painted rooms does not model it. Whether any part of the Ultra Secret rule survives on a plain grid is a question for this section, not an assumption for the code.

**§4 What a Special Room is** — `super-secret-neighbour-not-special` needs the list, and the list needs its own citation. Quote the wiki's own enumeration. **If the Start Room's membership is not stated, say so here**; it then becomes an `Unmodelled` entry in Task 4, not a decision made in silence.

- [ ] **Step 3: Amend §3 of the spec**

In `docs/superpowers/specs/2026-09-15-floor-secret-rooms-design.md`, replace the `Candidate` line

```rust
pub struct Candidate { pub cell: u16, pub tier: Tier, pub applied: Vec<RuleId> }
```

with

```rust
pub struct Candidate { pub cell: u16, pub neighbours: u8, pub rank: u8, pub applied: Vec<RuleId> }
```

and add, immediately under the code block:

```markdown
**No `Tier`.** Its variants could only be invented names for probability bands, and the repo's
rule is not to name a thing from a guess. A candidate carries a count and a rank — the rank
being its position in the preference order the cited rule states — and the screen colours by
the rank. Decided while writing the plan, 2026-09-15.
```

- [ ] **Step 4: Commit**

```bash
git add docs/superpowers/reports/2026-09-15-secret-room-rules.md docs/superpowers/specs/2026-09-15-floor-secret-rooms-design.md
git commit -m "docs: the secret room placement rules, quoted with their sources"
```

---

### Task 2: The crate, the grid, and neighbours that do not wrap the row

**Files:**
- Create: `crates/floor/Cargo.toml`
- Create: `crates/floor/src/lib.rs`
- Create: `crates/floor/src/grid.rs`
- Test: `crates/floor/tests/grid.rs`

`crates/*` is already the workspace's member glob (`Cargo.toml` at the root), so a new directory needs no edit there.

**Interfaces:**
- Consumes: nothing.
- Produces: `floor::{WIDTH, HEIGHT, CELLS, START}`, `floor::neighbours(cell: u16) -> Vec<u16>`.

- [ ] **Step 1: Write the failing test**

Create `crates/floor/tests/grid.rs`:

```rust
//! The grid, before any rule. A flat array of 169 cells makes two mistakes silently — the row
//! that wraps and the edge that does not — so both are properties here, not review comments.

use floor::{neighbours, CELLS, HEIGHT, START, WIDTH};

fn sorted(cell: u16) -> Vec<u16> {
    let mut v = neighbours(cell);
    v.sort_unstable();
    v
}

#[test]
fn the_grid_is_thirteen_by_thirteen_and_the_start_room_is_its_centre() {
    assert_eq!(WIDTH, 13);
    assert_eq!(HEIGHT, 13);
    assert_eq!(CELLS, 169);
    // 84 is what the game prints (CURRENT ROOM INDEX 84, 42 times across the five real logs
    // in samples/logs/ and never another value). That it is the centre is 6 * 13 + 6.
    assert_eq!(START, 84);
    assert_eq!(START, 6 * WIDTH + 6);
}

#[test]
fn a_cell_in_the_middle_has_four_neighbours() {
    assert_eq!(sorted(START), vec![71, 83, 85, 97]);
}

#[test]
fn a_cell_on_the_left_edge_has_no_neighbour_on_the_row_above() {
    // index 13 is (x = 0, y = 1). 12 is (x = 12, y = 0): the wrap this test exists for.
    assert_eq!(sorted(13), vec![0, 14, 26]);
}

#[test]
fn a_cell_on_the_right_edge_has_no_neighbour_on_the_row_below() {
    // index 25 is (x = 12, y = 1). 26 is (x = 0, y = 2).
    assert_eq!(sorted(25), vec![12, 24, 38]);
}

#[test]
fn the_four_corners_have_two_neighbours_each() {
    assert_eq!(sorted(0), vec![1, 13]);
    assert_eq!(sorted(12), vec![11, 25]);
    assert_eq!(sorted(156), vec![143, 157]);
    assert_eq!(sorted(168), vec![155, 167]);
}

#[test]
fn every_neighbour_is_one_step_away_and_never_across_a_row() {
    for cell in 0..CELLS as u16 {
        for n in neighbours(cell) {
            let (x, y) = (cell % WIDTH, cell / WIDTH);
            let (nx, ny) = (n % WIDTH, n / WIDTH);
            let dx = x.abs_diff(nx);
            let dy = y.abs_diff(ny);
            assert_eq!(dx + dy, 1, "{cell} and {n} are not adjacent");
        }
    }
}

#[test]
fn a_cell_outside_the_grid_has_no_neighbours() {
    assert!(neighbours(CELLS as u16).is_empty());
    assert!(neighbours(u16::MAX).is_empty());
}
```

- [ ] **Step 2: Create the crate so the test can fail for the right reason**

Create `crates/floor/Cargo.toml`:

```toml
[package]
name = "floor"
version = "0.1.0"
edition = "2021"
license.workspace = true
description = "The floor's grid and the game's documented secret-room placement rules"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

Create `crates/floor/src/lib.rs`:

```rust
//! floor — the grid a player paints, and the game's own rules about where a secret room can be.
//!
//! Pure: no I/O, no clock, no network. The rules are a JSON file embedded at build time, and
//! every one of them carries the sentence it was read from — see
//! `docs/superpowers/reports/2026-09-15-secret-room-rules.md`.

mod grid;

pub use grid::{neighbours, CELLS, HEIGHT, START, WIDTH};
```

Create `crates/floor/src/grid.rs` with the constants and a `neighbours` that returns an empty
vector, so the shape compiles and the behaviour is still missing:

```rust
/// The level grid is 13 wide and 13 tall; a cell's index is `y * WIDTH + x`.
pub const WIDTH: u16 = 13;
pub const HEIGHT: u16 = 13;
pub const CELLS: usize = (WIDTH * HEIGHT) as usize;

/// Where the game says a run starts: `CURRENT ROOM INDEX 84`, printed once per floor.
pub const START: u16 = 84;

/// The cells that touch `cell`, orthogonally. Never wraps a row, and answers nothing for an
/// index outside the grid rather than clamping it into a plausible wrong cell.
pub fn neighbours(_cell: u16) -> Vec<u16> {
    Vec::new()
}
```

- [ ] **Step 3: Run the test to verify it fails**

Run: `cargo test -p floor --test grid`
Expected: the constants tests pass, every `neighbours` test FAILS with a left/right mismatch against an empty vector.

- [ ] **Step 4: Write the implementation**

Replace the body of `neighbours` in `crates/floor/src/grid.rs`:

```rust
pub fn neighbours(cell: u16) -> Vec<u16> {
    if cell as usize >= CELLS {
        return Vec::new();
    }
    let (x, y) = (cell % WIDTH, cell / WIDTH);
    let mut out = Vec::with_capacity(4);
    if x > 0 {
        out.push(cell - 1);
    }
    if x + 1 < WIDTH {
        out.push(cell + 1);
    }
    if y > 0 {
        out.push(cell - WIDTH);
    }
    if y + 1 < HEIGHT {
        out.push(cell + WIDTH);
    }
    out
}
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p floor`
Expected: 7 passed.

- [ ] **Step 6: Commit**

```bash
git add crates/floor/Cargo.toml crates/floor/src/lib.rs crates/floor/src/grid.rs crates/floor/tests/grid.rs Cargo.lock
git commit -m "feat(floor): the level grid, with neighbours that do not wrap the row"
```

---

### Task 3: What a painted cell is — `RoomKind`, `Cell`, `Shape`, `Grid`

**Files:**
- Create: `crates/floor/src/room.rs`
- Modify: `crates/floor/src/grid.rs` (add `Grid`)
- Modify: `crates/floor/src/lib.rs` (re-exports)
- Test: `crates/floor/tests/room.rs`

**Interfaces:**
- Consumes: `neighbours`, `CELLS` from Task 2.
- Produces:
  - `floor::RoomKind` — `Start | Normal | Boss | Treasure | Shop | Curse | Challenge | Sacrifice | Arcade | Library | Miniboss | Secret | SuperSecret | UltraSecret`, `serde` `rename_all = "camelCase"`.
  - `floor::Shape` — `Single` (one variant, on purpose: see the spec's decision 4).
  - `floor::Cell` — `Empty | Room { kind: RoomKind, shape: Shape }`.
  - `floor::Grid` with `Grid::empty()`, `Grid::from_cells(cells: Vec<Cell>) -> Option<Grid>`, `Grid::at(cell: u16) -> Cell`, `Grid::set(&mut self, cell: u16, value: Cell)`, `Grid::painted(&self) -> usize`, `Grid::neighbour_kinds(cell: u16) -> Vec<RoomKind>`.

- [ ] **Step 1: Write the failing test**

Create `crates/floor/tests/room.rs`:

```rust
use floor::{Cell, Grid, RoomKind, Shape, CELLS, START};

fn one_room(cell: u16, kind: RoomKind) -> Grid {
    let mut g = Grid::empty();
    g.set(cell, Cell::Room { kind, shape: Shape::Single });
    g
}

#[test]
fn an_empty_grid_has_no_painted_cell() {
    let g = Grid::empty();
    assert_eq!(g.painted(), 0);
    assert_eq!(g.at(START), Cell::Empty);
}

#[test]
fn a_painted_cell_reads_back_as_what_was_painted() {
    let g = one_room(START, RoomKind::Start);
    assert_eq!(
        g.at(START),
        Cell::Room { kind: RoomKind::Start, shape: Shape::Single }
    );
    assert_eq!(g.painted(), 1);
}

#[test]
fn painting_outside_the_grid_changes_nothing() {
    let mut g = Grid::empty();
    g.set(CELLS as u16, Cell::Room { kind: RoomKind::Boss, shape: Shape::Single });
    assert_eq!(g.painted(), 0);
    assert_eq!(g.at(CELLS as u16), Cell::Empty);
}

#[test]
fn a_grid_of_the_wrong_length_is_refused_rather_than_padded() {
    assert!(Grid::from_cells(vec![Cell::Empty; CELLS]).is_some());
    assert!(Grid::from_cells(vec![Cell::Empty; CELLS - 1]).is_none());
    assert!(Grid::from_cells(vec![Cell::Empty; CELLS + 1]).is_none());
}

#[test]
fn the_kinds_next_to_a_cell_are_the_painted_ones_only() {
    let mut g = one_room(83, RoomKind::Normal);
    g.set(85, Cell::Room { kind: RoomKind::Boss, shape: Shape::Single });
    // 71 and 97 stay empty and contribute nothing.
    let mut kinds = g.neighbour_kinds(START);
    kinds.sort_by_key(|k| format!("{k:?}"));
    assert_eq!(kinds, vec![RoomKind::Boss, RoomKind::Normal]);
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p floor --test room`
Expected: FAIL to compile — `RoomKind`, `Cell`, `Shape`, `Grid` do not exist.

- [ ] **Step 3: Write the implementation**

Create `crates/floor/src/room.rs`:

```rust
use serde::{Deserialize, Serialize};

/// The room kinds a painted minimap can carry. Closed on purpose: a kind the grid cannot
/// name is a kind no rule may quietly treat as "normal".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RoomKind {
    Start,
    Normal,
    Boss,
    Treasure,
    Shop,
    Curse,
    Challenge,
    Sacrifice,
    Arcade,
    Library,
    Miniboss,
    Secret,
    SuperSecret,
    UltraSecret,
}

/// One variant, deliberately. The spec's decision 4 is "1x1 now, shapes later", and a
/// one-variant enum is what makes "later" a compile error instead of a rewrite.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Shape {
    Single,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cell {
    Empty,
    Room { kind: RoomKind, shape: Shape },
}
```

Append to `crates/floor/src/grid.rs`:

```rust
use crate::room::{Cell, RoomKind};

/// A painted floor: 169 cells, nothing else. It does not know which floor it is, and it does
/// not know whether it is finished.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    cells: Vec<Cell>,
}

impl Grid {
    pub fn empty() -> Self {
        Self { cells: vec![Cell::Empty; CELLS] }
    }

    /// `None` when the caller hands over something that is not a grid. Padding it would make
    /// a truncated payload look like a floor with empty cells at the end.
    pub fn from_cells(cells: Vec<Cell>) -> Option<Self> {
        (cells.len() == CELLS).then_some(Self { cells })
    }

    pub fn at(&self, cell: u16) -> Cell {
        self.cells.get(cell as usize).copied().unwrap_or(Cell::Empty)
    }

    pub fn set(&mut self, cell: u16, value: Cell) {
        if let Some(slot) = self.cells.get_mut(cell as usize) {
            *slot = value;
        }
    }

    pub fn painted(&self) -> usize {
        self.cells.iter().filter(|c| **c != Cell::Empty).count()
    }

    pub fn neighbour_kinds(&self, cell: u16) -> Vec<RoomKind> {
        neighbours(cell)
            .into_iter()
            .filter_map(|n| match self.at(n) {
                Cell::Empty => None,
                Cell::Room { kind, shape: _ } => Some(kind),
            })
            .collect()
    }
}
```

Update `crates/floor/src/lib.rs`:

```rust
mod grid;
mod room;

pub use grid::{neighbours, Grid, CELLS, HEIGHT, START, WIDTH};
pub use room::{Cell, RoomKind, Shape};
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p floor`
Expected: 12 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/floor/src/room.rs crates/floor/src/grid.rs crates/floor/src/lib.rs crates/floor/tests/room.rs
git commit -m "feat(floor): a painted cell, and a grid that refuses a payload of the wrong length"
```

---

### Task 4: The rules file and its loader

**Files:**
- Create: `crates/floor/rules/placement.json`
- Create: `crates/floor/src/rules.rs`
- Modify: `crates/floor/src/lib.rs`
- Test: `crates/floor/tests/rules.rs`

**Interfaces:**
- Consumes: `RoomKind` from Task 3.
- Produces:
  - `floor::Target` — `Secret | SuperSecret | UltraSecret`.
  - `floor::Constraint` — `NeighbourCount { allowed: Vec<u8>, rank: u8 } | ForbiddenNeighbour { kinds: Vec<RoomKind> } | NeighbourNotSpecial | DeadEndDistanceRank { rank: u8 } | Unmodelled { note: String }`.
  - `floor::Rule { id: String, target: Target, constraint: Constraint, quote: String, url: String }`.
  - `floor::Rules` with `Rules::embedded() -> Result<&'static Rules, RulesError>`, `Rules::parse(&str) -> Result<Rules, RulesError>`, `Rules::for_target(Target) -> impl Iterator<Item = &Rule>`.
  - `floor::SPECIAL_KINDS: &[RoomKind]`.

- [ ] **Step 1: Write the failing test**

Create `crates/floor/tests/rules.rs`. The parsing tests use a fixture written **in the test**, so they specify the loader and not the day's content of the wiki; the last test is the one that holds the real file to the report.

```rust
use floor::{Constraint, Rules, Target};

const FIXTURE: &str = r#"{
  "version": 1,
  "license": "CC BY-SA 4.0",
  "read": "2026-09-15",
  "rules": [
    {
      "id": "secret-neighbours",
      "target": "secret",
      "quote": "Regular Secret Rooms are usually located next to 3 or 4 rooms",
      "url": "https://example.invalid/Secret_Room",
      "constraint": { "kind": "neighbourCount", "allowed": [3, 4], "rank": 0 }
    },
    {
      "id": "secret-forbidden-neighbours",
      "target": "secret",
      "quote": "except Boss Rooms, Super Secret Rooms, and other Secret Rooms",
      "url": "https://example.invalid/Secret_Room",
      "constraint": { "kind": "forbiddenNeighbour", "kinds": ["boss", "superSecret", "secret"] }
    },
    {
      "id": "ultra-secret-connections",
      "target": "ultraSecret",
      "quote": "through its adjacent red rooms",
      "url": "https://example.invalid/Ultra_Secret_Room",
      "constraint": { "kind": "unmodelled", "note": "red rooms are not painted on this grid" }
    }
  ]
}"#;

#[test]
fn a_rule_carries_its_quotation_and_its_url() {
    let rules = Rules::parse(FIXTURE).expect("the fixture parses");
    let r = rules
        .for_target(Target::Secret)
        .find(|r| r.id == "secret-neighbours")
        .expect("the rule is there");
    assert_eq!(r.quote, "Regular Secret Rooms are usually located next to 3 or 4 rooms");
    assert_eq!(r.url, "https://example.invalid/Secret_Room");
}

#[test]
fn for_target_answers_only_that_targets_rules() {
    let rules = Rules::parse(FIXTURE).expect("the fixture parses");
    let ids: Vec<&str> = rules.for_target(Target::Secret).map(|r| r.id.as_str()).collect();
    assert_eq!(ids, vec!["secret-neighbours", "secret-forbidden-neighbours"]);
    assert_eq!(rules.for_target(Target::SuperSecret).count(), 0);
}

#[test]
fn a_constraint_the_grid_cannot_evaluate_parses_as_unmodelled_rather_than_being_dropped() {
    let rules = Rules::parse(FIXTURE).expect("the fixture parses");
    let r = rules.for_target(Target::UltraSecret).next().expect("one rule");
    match &r.constraint {
        Constraint::Unmodelled { note } => assert!(note.contains("red rooms")),
        other => panic!("expected Unmodelled, got {other:?}"),
    }
}

#[test]
fn a_malformed_rules_file_is_an_error_and_not_an_empty_set_of_rules() {
    assert!(Rules::parse("{").is_err());
    assert!(Rules::parse(r#"{"version":1,"rules":[{"id":"x"}]}"#).is_err());
}

#[test]
fn the_embedded_file_parses_and_every_rule_in_it_is_sourced() {
    let rules = Rules::embedded().expect("the embedded rules parse");
    assert!(
        rules.all().count() >= 4,
        "the report found at least four constraints worth encoding"
    );
    for r in rules.all() {
        assert!(!r.quote.trim().is_empty(), "{} has no quotation", r.id);
        assert!(
            r.url.starts_with("https://bindingofisaacrebirth.wiki.gg/wiki/"),
            "{} does not cite the wiki the repo already attributes",
            r.id
        );
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p floor --test rules`
Expected: FAIL to compile — `Rules`, `Target`, `Constraint` do not exist.

- [ ] **Step 3: Write the rules file from Task 1's report**

Create `crates/floor/rules/placement.json`. **Transcribe the report's §2 table — do not write a
rule the report does not have, and do not leave one out.** The shape is exactly the fixture's;
the content below is the transcription of the sentences quoted in Task 1, and the `url` of each
must be the real page. `super-secret-second-longest` uses `deadEndDistanceRank` with `rank: 1`
because the wiki says the **2nd** most rooms walked, and ranks count from zero.

**The order of the rules in the file is significant**, and Task 5's solver depends on it: a
`neighbourCount` rule *proposes* cells, every other kind only *narrows* what was proposed. So
every count rule for a target comes first, narrowing rules after. Putting
`secret-forbidden-neighbours` above `secret-neighbours` would filter an empty list and forbid
nothing, with no error anywhere — which is why Task 5's boss test checks both the grid that
should be rejected and the one that should not.

```json
{
  "version": 1,
  "license": "CC BY-SA 4.0, The Binding of Isaac: Rebirth Wiki",
  "read": "2026-09-15",
  "rules": [
    {
      "id": "secret-neighbours",
      "target": "secret",
      "quote": "Secret Rooms are equally as likely to be in a valid location with 3 neighbors, as it is with 4 neighbors.",
      "url": "https://bindingofisaacrebirth.wiki.gg/wiki/Secret_Room",
      "constraint": { "kind": "neighbourCount", "allowed": [3, 4], "rank": 0 }
    },
    {
      "id": "secret-neighbours-two",
      "target": "secret",
      "quote": "2 neighbor locations are rare but possible, even when there are locations with 3+ neighbors available.",
      "url": "https://bindingofisaacrebirth.wiki.gg/wiki/Secret_Room",
      "constraint": { "kind": "neighbourCount", "allowed": [2], "rank": 1 }
    },
    {
      "id": "secret-forbidden-neighbours",
      "target": "secret",
      "quote": "Secret Rooms can exist next to all types of rooms except Boss Rooms, Super Secret Rooms, and other Secret Rooms.",
      "url": "https://bindingofisaacrebirth.wiki.gg/wiki/Secret_Room",
      "constraint": { "kind": "forbiddenNeighbour", "kinds": ["boss", "superSecret", "secret"] }
    },
    {
      "id": "super-secret-dead-end",
      "target": "superSecret",
      "quote": "Super Secret Rooms are only located next to one other room, and this room can't be a Special Room; in other words, it is placed on one of the floor's dead ends.",
      "url": "https://bindingofisaacrebirth.wiki.gg/wiki/Secret_Room",
      "constraint": { "kind": "neighbourCount", "allowed": [1], "rank": 0 }
    },
    {
      "id": "super-secret-neighbour-not-special",
      "target": "superSecret",
      "quote": "this room can't be a Special Room",
      "url": "https://bindingofisaacrebirth.wiki.gg/wiki/Secret_Room",
      "constraint": { "kind": "neighbourNotSpecial" }
    },
    {
      "id": "super-secret-second-longest",
      "target": "superSecret",
      "quote": "Super Secret Rooms replace the dead-end room that would require the 2nd most rooms walked through from the start room to access.",
      "url": "https://bindingofisaacrebirth.wiki.gg/wiki/Secret_Room",
      "constraint": { "kind": "deadEndDistanceRank", "rank": 1 }
    },
    {
      "id": "ultra-secret-connections",
      "target": "ultraSecret",
      "quote": "Ultra Secret rooms are most likely generated in spots that connect to 3+ non-red rooms through its adjacent red rooms.",
      "url": "https://bindingofisaacrebirth.wiki.gg/wiki/Secret_Room",
      "constraint": { "kind": "unmodelled", "note": "red rooms are the Red Key mechanic and are not painted on this grid" }
    }
  ]
}
```

- [ ] **Step 4: Write the loader**

Create `crates/floor/src/rules.rs`:

```rust
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::room::RoomKind;

/// Which secret room a rule is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Target {
    Secret,
    SuperSecret,
    UltraSecret,
}

/// What a rule asks of a cell. `Unmodelled` is the honest variant: a sentence the wiki states
/// and this grid cannot evaluate. It is carried, never dropped, because a dropped constraint
/// reads as "nothing in the way".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum Constraint {
    NeighbourCount { allowed: Vec<u8>, rank: u8 },
    ForbiddenNeighbour { kinds: Vec<RoomKind> },
    NeighbourNotSpecial,
    DeadEndDistanceRank { rank: u8 },
    Unmodelled { note: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub target: Target,
    pub quote: String,
    pub url: String,
    pub constraint: Constraint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rules {
    pub version: u32,
    pub license: String,
    pub read: String,
    rules: Vec<Rule>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulesError {
    pub message: String,
}

impl Rules {
    pub fn parse(text: &str) -> Result<Self, RulesError> {
        serde_json::from_str(text).map_err(|e| RulesError { message: e.to_string() })
    }

    /// The file committed next to this crate, parsed once. An error here is a build-time
    /// mistake the tests catch, so it is reported rather than unwrapped.
    pub fn embedded() -> Result<&'static Rules, RulesError> {
        static ONCE: OnceLock<Result<Rules, RulesError>> = OnceLock::new();
        ONCE.get_or_init(|| Rules::parse(include_str!("../rules/placement.json")))
            .as_ref()
            .map_err(Clone::clone)
    }

    pub fn all(&self) -> impl Iterator<Item = &Rule> {
        self.rules.iter()
    }

    pub fn for_target(&self, target: Target) -> impl Iterator<Item = &Rule> {
        self.rules.iter().filter(move |r| r.target == target)
    }
}

/// The room kinds the wiki calls Special Rooms. Sourced in §4 of the rules report; a kind whose
/// membership that report could not state is **not** here.
pub const SPECIAL_KINDS: &[RoomKind] = &[
    RoomKind::Boss,
    RoomKind::Treasure,
    RoomKind::Shop,
    RoomKind::Curse,
    RoomKind::Challenge,
    RoomKind::Sacrifice,
    RoomKind::Arcade,
    RoomKind::Library,
    RoomKind::Miniboss,
    RoomKind::Secret,
    RoomKind::SuperSecret,
    RoomKind::UltraSecret,
];

pub fn is_special(kind: RoomKind) -> bool {
    SPECIAL_KINDS.contains(&kind)
}
```

If §4 of the report could not source a kind in that list, **remove it** and add a line to the
report saying so. The list is a transcription, not a recollection.

Update `crates/floor/src/lib.rs`:

```rust
mod grid;
mod room;
mod rules;

pub use grid::{neighbours, Grid, CELLS, HEIGHT, START, WIDTH};
pub use room::{Cell, RoomKind, Shape};
pub use rules::{is_special, Constraint, Rule, Rules, RulesError, Target, SPECIAL_KINDS};
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p floor`
Expected: 17 passed.

- [ ] **Step 6: Commit**

```bash
git add crates/floor/rules/placement.json crates/floor/src/rules.rs crates/floor/src/lib.rs crates/floor/tests/rules.rs
git commit -m "feat(floor): the placement rules, each with the sentence it was read from"
```

---

### Task 5: The solver

**Files:**
- Create: `crates/floor/src/solve.rs`
- Modify: `crates/floor/src/lib.rs`
- Test: `crates/floor/tests/solve.rs`

**Interfaces:**
- Consumes: `Grid`, `RoomKind`, `Cell`, `Shape`, `neighbours`, `Rules`, `Target`, `Constraint`, `is_special`.
- Produces:
  - `floor::Candidate { cell: u16, neighbours: u8, rank: u8, applied: Vec<String> }`
  - `floor::Unresolved { rule: String, note: String, quote: String, url: String }`
  - `floor::Solution { target: Target, candidates: Vec<Candidate>, unresolved: Vec<Unresolved> }`
  - `floor::solve(grid: &Grid, rules: &Rules, target: Target) -> Solution`
  - `floor::distance_from_start(grid: &Grid) -> Vec<Option<u16>>` — breadth-first over painted cells, index-aligned with the grid, `None` where a cell is empty or unreachable.

- [ ] **Step 1: Write the failing test**

Create `crates/floor/tests/solve.rs`. The grids are drawn as text so a reader can check the
expected value by eye; `parse_grid` maps one character per cell.

```rust
//! Every expected value here comes from a sentence in
//! `docs/superpowers/reports/2026-09-15-secret-room-rules.md`, named in the test.

use floor::{
    distance_from_start, solve, Cell, Grid, RoomKind, Rules, Shape, Target, WIDTH,
};

/// `.` empty, `S` start, `n` normal, `B` boss, `T` treasure. Rows are laid out from cell 0.
fn parse_grid(rows: &[&str]) -> Grid {
    let mut g = Grid::empty();
    for (y, row) in rows.iter().enumerate() {
        for (x, c) in row.chars().enumerate() {
            let kind = match c {
                '.' => continue,
                'S' => RoomKind::Start,
                'n' => RoomKind::Normal,
                'B' => RoomKind::Boss,
                'T' => RoomKind::Treasure,
                other => panic!("unknown cell {other}"),
            };
            let cell = (y as u16) * WIDTH + x as u16;
            g.set(cell, Cell::Room { kind, shape: Shape::Single });
        }
    }
    g
}

fn rules() -> &'static Rules {
    Rules::embedded().expect("the embedded rules parse")
}

fn cells(s: &floor::Solution) -> Vec<u16> {
    s.candidates.iter().map(|c| c.cell).collect()
}

#[test]
fn a_cell_with_four_neighbours_outranks_one_with_two() {
    // rule secret-neighbours (rank 0, "3 neighbors … as it is with 4") against
    // secret-neighbours-two (rank 1, "2 neighbor locations are rare but possible").
    // Cell 14 is the hole in the middle of the ring: it touches 1, 13, 15 and 27.
    let g = parse_grid(&[
        ".n.",
        "n.n",
        ".n.",
    ]);
    let s = solve(&g, rules(), Target::Secret);
    let first = s.candidates.first().expect("at least one candidate");
    assert_eq!(first.cell, 14);
    assert_eq!(first.neighbours, 4);
    assert_eq!(first.rank, 0);
    assert!(
        s.candidates.windows(2).all(|w| w[0].rank <= w[1].rank),
        "candidates come out in rank order"
    );
    assert!(
        s.candidates.iter().any(|c| c.rank == 1 && c.neighbours == 2),
        "a two-neighbour cell is still a candidate, ranked below"
    );
}

#[test]
fn a_cell_touching_the_boss_room_is_not_a_secret_room_candidate() {
    // rule secret-forbidden-neighbours: "except Boss Rooms, Super Secret Rooms, and other
    // Secret Rooms". Cell 14 touches three painted rooms and would rank first without it.
    let g = parse_grid(&[
        "nnn",
        "n.B",
    ]);
    let s = solve(&g, rules(), Target::Secret);
    assert!(!cells(&s).contains(&14), "cell 14 touches the boss room");

    // The same floor with a normal room where the boss was: the instrument has to be shown
    // able to speak before its silence counts as evidence.
    let ok = parse_grid(&[
        "nnn",
        "n.n",
    ]);
    let s = solve(&ok, rules(), Target::Secret);
    assert_eq!(s.candidates.first().map(|c| c.cell), Some(14));
    assert_eq!(s.candidates.first().map(|c| c.neighbours), Some(3));
}

#[test]
fn a_candidate_names_the_rules_that_elected_it() {
    let g = parse_grid(&[
        ".n.",
        "nnn",
        ".n.",
    ]);
    let s = solve(&g, rules(), Target::Secret);
    let c = s.candidates.first().expect("at least one candidate");
    assert!(
        c.applied.iter().any(|id| id == "secret-neighbours"
            || id == "secret-neighbours-two"),
        "a lit cell says which rule lit it"
    );
}

#[test]
fn the_super_secret_room_is_a_dead_end_and_not_a_crossroads() {
    // rule super-secret-dead-end: "only located next to one other room".
    let g = parse_grid(&[
        "Snn",
        "...",
    ]);
    let s = solve(&g, rules(), Target::SuperSecret);
    for c in &s.candidates {
        assert_eq!(c.neighbours, 1, "cell {} is not a dead end", c.cell);
    }
}

#[test]
fn a_dead_end_hanging_off_a_special_room_is_not_a_super_secret_candidate() {
    // rule super-secret-neighbour-not-special: "this room can't be a Special Room".
    let g = parse_grid(&[
        "SnT",
    ]);
    let s = solve(&g, rules(), Target::SuperSecret);
    // cell 3 (row 0, x = 3) touches only the treasure room at cell 2.
    assert!(!cells(&s).contains(&3), "its only neighbour is a Special Room");
}

#[test]
fn the_ultra_secret_rule_is_reported_as_unresolved_rather_than_answered() {
    // rule ultra-secret-connections is Unmodelled: red rooms are not on this grid.
    let s = solve(&Grid::empty(), rules(), Target::UltraSecret);
    assert!(
        s.unresolved.iter().any(|u| u.rule == "ultra-secret-connections"),
        "an unmodelled rule says so"
    );
    assert!(
        !s.unresolved.is_empty() && s.candidates.is_empty(),
        "nothing painted, nothing claimed"
    );
}

#[test]
fn an_empty_grid_produces_no_candidate_for_any_target() {
    for target in [Target::Secret, Target::SuperSecret, Target::UltraSecret] {
        assert!(solve(&Grid::empty(), rules(), target).candidates.is_empty());
    }
}

#[test]
fn distance_from_start_counts_rooms_walked_and_stops_at_the_paint() {
    let g = parse_grid(&[
        "Snn",
        "..n",
    ]);
    let d = distance_from_start(&g);
    assert_eq!(d[0], Some(0), "the start room is zero rooms away");
    assert_eq!(d[1], Some(1));
    assert_eq!(d[2], Some(2));
    assert_eq!(d[15], Some(3), "cell 15 is reached through cell 2");
    assert_eq!(d[13], None, "an empty cell has no distance");
}

#[test]
fn without_a_start_room_the_distance_rule_is_unresolved_and_the_dead_ends_still_show() {
    // The grid has no Start: super-secret-second-longest cannot be evaluated, which is a
    // sentence the screen has to say — not a reason to light nothing.
    let g = parse_grid(&[
        "nnn",
    ]);
    let s = solve(&g, rules(), Target::SuperSecret);
    assert!(s.unresolved.iter().any(|u| u.rule == "super-secret-second-longest"));
    assert!(!s.candidates.is_empty(), "the dead-end rule still answers");
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p floor --test solve`
Expected: FAIL to compile — `solve` and `distance_from_start` do not exist.

- [ ] **Step 3: Write the implementation**

Create `crates/floor/src/solve.rs`:

```rust
use std::collections::VecDeque;

use serde::Serialize;

use crate::grid::{neighbours, Grid, CELLS};
use crate::room::Cell;
use crate::rules::{is_special, Constraint, Rules, Target};

/// One cell a rule allows, with the count that earned it and the rank of the rule that did.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Candidate {
    pub cell: u16,
    pub neighbours: u8,
    pub rank: u8,
    pub applied: Vec<String>,
}

/// A rule the grid cannot evaluate, carried out loud.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Unresolved {
    pub rule: String,
    pub note: String,
    pub quote: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Solution {
    pub target: Target,
    pub candidates: Vec<Candidate>,
    pub unresolved: Vec<Unresolved>,
}

/// How many rooms are walked from the start room to each painted cell. `None` for an empty
/// cell, for a cell no path reaches, and for every cell when nothing is painted Start.
pub fn distance_from_start(grid: &Grid) -> Vec<Option<u16>> {
    let mut out = vec![None; CELLS];
    let start = (0..CELLS as u16).find(|c| {
        matches!(grid.at(*c), Cell::Room { kind, shape: _ } if kind == crate::RoomKind::Start)
    });
    let Some(start) = start else { return out };
    out[start as usize] = Some(0);
    let mut queue = VecDeque::from([start]);
    while let Some(cell) = queue.pop_front() {
        let Some(d) = out[cell as usize] else { continue };
        for n in neighbours(cell) {
            if matches!(grid.at(n), Cell::Empty) || out[n as usize].is_some() {
                continue;
            }
            out[n as usize] = Some(d + 1);
            queue.push_back(n);
        }
    }
    out
}

pub fn solve(grid: &Grid, rules: &Rules, target: Target) -> Solution {
    let mut candidates: Vec<Candidate> = Vec::new();
    let mut unresolved: Vec<Unresolved> = Vec::new();

    let has_start = distance_from_start(grid).iter().any(Option::is_some);

    for rule in rules.for_target(target) {
        match &rule.constraint {
            Constraint::Unmodelled { note } => unresolved.push(Unresolved {
                rule: rule.id.clone(),
                note: note.clone(),
                quote: rule.quote.clone(),
                url: rule.url.clone(),
            }),
            Constraint::DeadEndDistanceRank { rank: _ } if !has_start => {
                unresolved.push(Unresolved {
                    rule: rule.id.clone(),
                    note: "no start room is painted".to_string(),
                    quote: rule.quote.clone(),
                    url: rule.url.clone(),
                })
            }
            Constraint::NeighbourCount { allowed, rank } => {
                for cell in 0..CELLS as u16 {
                    if !matches!(grid.at(cell), Cell::Empty) {
                        continue;
                    }
                    let count = grid.neighbour_kinds(cell).len() as u8;
                    if allowed.contains(&count) {
                        candidates.push(Candidate {
                            cell,
                            neighbours: count,
                            rank: *rank,
                            applied: vec![rule.id.clone()],
                        });
                    }
                }
            }
            // The three below narrow what the count rules proposed: they never add a cell.
            Constraint::ForbiddenNeighbour { kinds } => {
                candidates.retain(|c| {
                    !grid
                        .neighbour_kinds(c.cell)
                        .into_iter()
                        .any(|k| kinds.contains(&k))
                });
                note_applied(&mut candidates, &rule.id);
            }
            Constraint::NeighbourNotSpecial => {
                candidates.retain(|c| !grid.neighbour_kinds(c.cell).into_iter().any(is_special));
                note_applied(&mut candidates, &rule.id);
            }
            Constraint::DeadEndDistanceRank { rank } => {
                let distances = distance_from_start(grid);
                let mut reach: Vec<(u16, u16)> = candidates
                    .iter()
                    .filter_map(|c| {
                        let d = neighbours(c.cell)
                            .into_iter()
                            .filter_map(|n| distances[n as usize])
                            .max()?;
                        Some((c.cell, d))
                    })
                    .collect();
                reach.sort_by(|a, b| b.1.cmp(&a.1));
                let chosen = reach.get(*rank as usize).map(|(cell, _)| *cell);
                for c in candidates.iter_mut() {
                    if Some(c.cell) == chosen {
                        c.rank = 0;
                        c.applied.push(rule.id.clone());
                    } else {
                        c.rank = c.rank.saturating_add(1);
                    }
                }
            }
        }
    }

    candidates.sort_by(|a, b| a.rank.cmp(&b.rank).then(a.cell.cmp(&b.cell)));
    Solution { target, candidates, unresolved }
}

/// A narrowing rule applies to everything that survived it, and saying so is how a candidate
/// can name every rule behind it rather than only the one that proposed it.
fn note_applied(candidates: &mut [Candidate], id: &str) {
    for c in candidates.iter_mut() {
        c.applied.push(id.to_string());
    }
}
```

Update `crates/floor/src/lib.rs`:

```rust
mod grid;
mod room;
mod rules;
mod solve;

pub use grid::{neighbours, Grid, CELLS, HEIGHT, START, WIDTH};
pub use room::{Cell, RoomKind, Shape};
pub use rules::{is_special, Constraint, Rule, Rules, RulesError, Target, SPECIAL_KINDS};
pub use solve::{distance_from_start, solve, Candidate, Solution, Unresolved};
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p floor`
Expected: 26 passed.

If a test fails, **treat it as a hypothesis of a bug in `solve`**, not as an expectation to
adjust. The expected values come from quoted sentences; changing one means the quotation was
transcribed wrong, and that is a fix in `placement.json` and in the report, never a fix in the
test alone.

- [ ] **Step 5: Run clippy and fmt**

Run: `cargo fmt && cargo clippy -p floor --all-targets -- -D warnings`
Expected: no warnings.

- [ ] **Step 6: Commit**

```bash
git add crates/floor/src/solve.rs crates/floor/src/lib.rs crates/floor/tests/solve.rs
git commit -m "feat(floor): the solver, which narrows by cited rule and says what it cannot judge"
```

---

### Task 6: The view-model and the generated contract

**Files:**
- Create: `crates/ipc/src/floor.rs`
- Modify: `crates/ipc/src/lib.rs`
- Modify: `crates/ipc/Cargo.toml`
- Modify: `crates/ipc/src/contract.rs`
- Test: `crates/ipc/tests/floor_shape.rs`

**Interfaces:**
- Consumes: `floor::{Grid, Cell, RoomKind, Shape, Rules, Target, solve, Solution}`.
- Produces:
  - `ipc::RoomKindView` — fieldless, bare camelCase string, one variant per `floor::RoomKind`.
  - `ipc::TargetView` — fieldless, bare camelCase string.
  - `ipc::AppliedRule { id, quote, url }`.
  - `ipc::FloorCandidate { cell, neighbours, rank, applied: Vec<AppliedRule> }`.
  - `ipc::FloorUnresolved { rule, note, quote, url }`.
  - `ipc::FloorSolutionView { target, candidates, unresolved }`.
  - `ipc::FloorDiagnostic` — tagged, `GridEmpty`, `NoStartRoom`, `RulesUnreadable { reason }`.
  - `ipc::FloorView { solutions, painted, diagnostics }`.
  - `ipc::floor_view(cells: Vec<Option<RoomKindView>>) -> FloorView`.

- [ ] **Step 1: Write the failing test**

Create `crates/ipc/tests/floor_shape.rs`:

```rust
//! The wire shape. A missing `rename_all` does not fail a Rust test, it makes TypeScript read
//! `undefined` in silence — so the JSON is asserted here, key by key.

use ipc::{floor_view, RoomKindView};
use serde_json::Value;

fn empty_cells() -> Vec<Option<RoomKindView>> {
    vec![None; 169]
}

#[test]
fn an_empty_grid_says_so_instead_of_answering() {
    let v = floor_view(empty_cells());
    let json = serde_json::to_value(&v).expect("serializes");
    assert_eq!(json["painted"], Value::from(0));
    let kinds: Vec<&str> = json["diagnostics"]
        .as_array()
        .expect("an array")
        .iter()
        .filter_map(|d| d["kind"].as_str())
        .collect();
    assert!(kinds.contains(&"gridEmpty"));
}

#[test]
fn a_candidate_crosses_in_camel_case_and_carries_its_quotation() {
    let mut cells = empty_cells();
    // A plus of normal rooms around cell 84 leaves its four diagonals with two neighbours
    // each and makes the cell itself painted; the cell above the plus has three.
    for cell in [84usize, 83, 85, 71, 97] {
        cells[cell] = Some(RoomKindView::Normal);
    }
    cells[84] = Some(RoomKindView::Start);
    let v = floor_view(cells);
    let json = serde_json::to_value(&v).expect("serializes");
    let solution = &json["solutions"][0];
    assert!(solution["target"].is_string(), "a fieldless enum is a bare string");
    let first = &solution["candidates"][0];
    assert!(first["cell"].is_number());
    assert!(first["neighbours"].is_number());
    assert!(first["rank"].is_number());
    let applied = &first["applied"][0];
    assert!(applied["quote"].as_str().is_some_and(|q| !q.is_empty()));
    assert!(applied["url"].as_str().is_some_and(|u| u.starts_with("https://")));
}

#[test]
fn a_grid_of_the_wrong_length_is_a_diagnostic_and_not_a_panic() {
    let v = floor_view(vec![None; 3]);
    let json = serde_json::to_value(&v).expect("serializes");
    assert_eq!(json["solutions"].as_array().map(Vec::len), Some(0));
}

#[test]
fn every_room_kind_of_the_domain_has_a_view() {
    // Adding a kind to `floor` has to break this, not silently drop a paintable room.
    let all = [
        RoomKindView::Start,
        RoomKindView::Normal,
        RoomKindView::Boss,
        RoomKindView::Treasure,
        RoomKindView::Shop,
        RoomKindView::Curse,
        RoomKindView::Challenge,
        RoomKindView::Sacrifice,
        RoomKindView::Arcade,
        RoomKindView::Library,
        RoomKindView::Miniboss,
        RoomKindView::Secret,
        RoomKindView::SuperSecret,
        RoomKindView::UltraSecret,
    ];
    assert_eq!(all.len(), 14);
    for k in all {
        let json = serde_json::to_value(k).expect("serializes");
        assert!(json.as_str().is_some(), "a fieldless enum is a bare string");
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p ipc --test floor_shape`
Expected: FAIL to compile — `floor_view` and `RoomKindView` do not exist.

- [ ] **Step 3: Add the dependency**

In `crates/ipc/Cargo.toml`, under `[dependencies]`, after the `plan` line:

```toml
floor = { path = "../floor" }
```

- [ ] **Step 4: Write the view-model**

Create `crates/ipc/src/floor.rs`:

```rust
//! The painted grid's answer, as JSON. Pure: the grid arrives from the screen, the rules are
//! embedded in `floor`, and nothing here opens a file.
//!
//! Every lit cell carries the sentence that lit it. That is not decoration: the rules are
//! quotations from a CC BY-SA wiki, and showing the quotation is how the attribution reaches
//! the person reading the screen.

use serde::{Deserialize, Serialize};

/// A room as the screen paints it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum RoomKindView {
    Start,
    Normal,
    Boss,
    Treasure,
    Shop,
    Curse,
    Challenge,
    Sacrifice,
    Arcade,
    Library,
    Miniboss,
    Secret,
    SuperSecret,
    UltraSecret,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum TargetView {
    Secret,
    SuperSecret,
    UltraSecret,
}

/// One rule behind a candidate, with the sentence it was read from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct AppliedRule {
    pub id: String,
    pub quote: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct FloorCandidate {
    pub cell: u16,
    pub neighbours: u8,
    /// Its place in the preference order the cited rule states; 0 is that rule's first.
    pub rank: u8,
    pub applied: Vec<AppliedRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct FloorUnresolved {
    pub rule: String,
    pub note: String,
    pub quote: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct FloorSolutionView {
    pub target: TargetView,
    pub candidates: Vec<FloorCandidate>,
    pub unresolved: Vec<FloorUnresolved>,
}

/// Everything that stops the screen from answering, said out loud. None of them may be drawn
/// as "there is nowhere for a secret room": that is an answer, and these are the absence of one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum FloorDiagnostic {
    /// Nothing painted yet.
    GridEmpty,
    /// No Start room, so the rule about how many rooms are walked cannot be read.
    NoStartRoom,
    /// The embedded rules did not parse. A build-time mistake, reported rather than hidden.
    RulesUnreadable { reason: String },
    /// The grid did not have 169 cells. The screen sent something that is not a floor.
    GridMalformed { cells: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct FloorView {
    pub solutions: Vec<FloorSolutionView>,
    /// How many cells are painted. A count, not a fraction: nothing here knows how many rooms
    /// the floor has.
    pub painted: u32,
    pub diagnostics: Vec<FloorDiagnostic>,
}

fn kind_of(view: RoomKindView) -> floor::RoomKind {
    match view {
        RoomKindView::Start => floor::RoomKind::Start,
        RoomKindView::Normal => floor::RoomKind::Normal,
        RoomKindView::Boss => floor::RoomKind::Boss,
        RoomKindView::Treasure => floor::RoomKind::Treasure,
        RoomKindView::Shop => floor::RoomKind::Shop,
        RoomKindView::Curse => floor::RoomKind::Curse,
        RoomKindView::Challenge => floor::RoomKind::Challenge,
        RoomKindView::Sacrifice => floor::RoomKind::Sacrifice,
        RoomKindView::Arcade => floor::RoomKind::Arcade,
        RoomKindView::Library => floor::RoomKind::Library,
        RoomKindView::Miniboss => floor::RoomKind::Miniboss,
        RoomKindView::Secret => floor::RoomKind::Secret,
        RoomKindView::SuperSecret => floor::RoomKind::SuperSecret,
        RoomKindView::UltraSecret => floor::RoomKind::UltraSecret,
    }
}

fn target_view(t: floor::Target) -> TargetView {
    match t {
        floor::Target::Secret => TargetView::Secret,
        floor::Target::SuperSecret => TargetView::SuperSecret,
        floor::Target::UltraSecret => TargetView::UltraSecret,
    }
}

/// The join: the painted grid in, the three rankings out.
pub fn floor_view(cells: Vec<Option<RoomKindView>>) -> FloorView {
    let mut diagnostics = Vec::new();

    let count = cells.len();
    let painted = cells.iter().filter(|c| c.is_some()).count() as u32;
    let grid = floor::Grid::from_cells(
        cells
            .into_iter()
            .map(|c| match c {
                None => floor::Cell::Empty,
                Some(k) => floor::Cell::Room {
                    kind: kind_of(k),
                    shape: floor::Shape::Single,
                },
            })
            .collect(),
    );

    let Some(grid) = grid else {
        diagnostics.push(FloorDiagnostic::GridMalformed { cells: count as u32 });
        return FloorView { solutions: Vec::new(), painted, diagnostics };
    };
    if painted == 0 {
        diagnostics.push(FloorDiagnostic::GridEmpty);
    }
    if !floor::distance_from_start(&grid).iter().any(Option::is_some) {
        diagnostics.push(FloorDiagnostic::NoStartRoom);
    }

    let rules = match floor::Rules::embedded() {
        Ok(r) => r,
        Err(e) => {
            diagnostics.push(FloorDiagnostic::RulesUnreadable { reason: e.message });
            return FloorView { solutions: Vec::new(), painted, diagnostics };
        }
    };

    let solutions = [
        floor::Target::Secret,
        floor::Target::SuperSecret,
        floor::Target::UltraSecret,
    ]
    .into_iter()
    .map(|t| {
        let s = floor::solve(&grid, rules, t);
        FloorSolutionView {
            target: target_view(s.target),
            candidates: s
                .candidates
                .into_iter()
                .map(|c| FloorCandidate {
                    cell: c.cell,
                    neighbours: c.neighbours,
                    rank: c.rank,
                    applied: c
                        .applied
                        .into_iter()
                        .filter_map(|id| {
                            rules.all().find(|r| r.id == id).map(|r| AppliedRule {
                                id: r.id.clone(),
                                quote: r.quote.clone(),
                                url: r.url.clone(),
                            })
                        })
                        .collect(),
                })
                .collect(),
            unresolved: s
                .unresolved
                .into_iter()
                .map(|u| FloorUnresolved {
                    rule: u.rule,
                    note: u.note,
                    quote: u.quote,
                    url: u.url,
                })
                .collect(),
        }
    })
    .collect();

    FloorView { solutions, painted, diagnostics }
}
```

- [ ] **Step 5: Register the module and the exports**

A module named `floor` inside a crate that also depends on the crate `floor` is the shape
`crates/ipc/src/graph.rs` already has (`mod graph;` next to `graph = { path = "../graph" }`,
which reaches the crate as `graph::build::Node` from inside the module). Follow it. If any path
does turn ambiguous, disambiguate with `::floor::`, never by renaming the module.

In `crates/ipc/src/lib.rs`, add `mod floor;` in alphabetical order (between `mod error;` and
`mod for_tests;`) and the re-export block after the `pub use error::IpcError;` line:

```rust
pub use floor::{
    floor_view, AppliedRule, FloorCandidate, FloorDiagnostic, FloorSolutionView, FloorUnresolved,
    FloorView, RoomKindView, TargetView,
};
```

- [ ] **Step 6: Register the types in the contract**

In `crates/ipc/src/contract.rs`, inside `render()`, after the last `decl::<crate::RunsView>` line,
add — in this order, since a type must be declared before it is used as a field:

```rust
    decl::<crate::RoomKindView>(&cfg, &mut out);
    decl::<crate::TargetView>(&cfg, &mut out);
    decl::<crate::AppliedRule>(&cfg, &mut out);
    decl::<crate::FloorCandidate>(&cfg, &mut out);
    decl::<crate::FloorUnresolved>(&cfg, &mut out);
    decl::<crate::FloorSolutionView>(&cfg, &mut out);
    decl::<crate::FloorDiagnostic>(&cfg, &mut out);
    decl::<crate::FloorView>(&cfg, &mut out);
```

- [ ] **Step 7: Run the tests and generate the types**

Run: `cargo test -p ipc --test floor_shape`
Expected: 4 passed.

Run: `pnpm ipc:types`
Expected: `wrote …/ui/src/lib/ipc/types.ts`, and `git diff` on that file shows the eight new
declarations with `RoomKindView` and `TargetView` rendered as `const … as const` pairs.

- [ ] **Step 8: Commit**

```bash
git add crates/ipc/src/floor.rs crates/ipc/src/lib.rs crates/ipc/src/contract.rs crates/ipc/Cargo.toml crates/ipc/tests/floor_shape.rs ui/src/lib/ipc/types.ts Cargo.lock
git commit -m "feat(ipc): the painted floor's three rankings, each candidate with its quotation"
```

---

### Task 7: The command and the typed wrapper

**Files:**
- Create: `crates/app/src/commands/floor.rs`
- Modify: `crates/app/src/commands/mod.rs`
- Modify: `crates/app/src/lib.rs` (the `invoke_handler` list)
- Create: `ui/src/lib/ipc/floor.ts`
- Modify: `ui/src/lib/constants/commands.ts`
- Create: `ui/src/lib/ipc/fixtures/floor.ts`
- Modify: `ui/src/lib/ipc/fixtures/index.ts`

**Interfaces:**
- Consumes: `ipc::{floor_view, FloorView, RoomKindView, IpcError}`.
- Produces: the Tauri command `floor_candidates`, the constant `Command.FloorCandidates = 'floor_candidates'`, and `floorCandidates(cells: (RoomKindView | null)[]): Promise<FloorView>`.

- [ ] **Step 1: Write the command**

Create `crates/app/src/commands/floor.rs`:

```rust
//! The painted grid's ranking. Wiring only: the grid arrives from the screen, the rules are
//! embedded in `floor`, and nothing is read from disk — so there is no state to hold and no
//! failure that is not already a diagnostic.

use ipc::{FloorView, IpcError, RoomKindView};

#[tauri::command]
pub(crate) fn floor_candidates(cells: Vec<Option<RoomKindView>>) -> Result<FloorView, IpcError> {
    Ok(ipc::floor_view(cells))
}
```

In `crates/app/src/commands/mod.rs`, add `pub(crate) mod floor;` in alphabetical order.

In `crates/app/src/lib.rs`, add `floor::floor_candidates,` to the `tauri::generate_handler![…]`
list, after `runs::live` (remember the comma on the previous line).

- [ ] **Step 2: Add the command constant and the wrapper**

In `ui/src/lib/constants/commands.ts`, add to the `Command` object, after `Live: 'live',`:

```ts
  FloorCandidates: 'floor_candidates',
```

Create `ui/src/lib/ipc/floor.ts`:

```ts
import { Command } from '../constants/commands'
import { call } from './transport'
import type { FloorView, RoomKindView } from './types'

// The grid you painted, ranked by the game's own documented rules. Pure on the Rust side: it
// reads nothing, so calling it again with the same grid answers the same thing.
export const floorCandidates = (
  cells: (RoomKindView | null)[],
): Promise<FloorView> => call(Command.FloorCandidates, { cells })
```

- [ ] **Step 3: Add the fixture**

Create `ui/src/lib/ipc/fixtures/floor.ts`:

```ts
import type { CommandArgs } from '../transport'
import type { FloorView, RoomKindView } from '../types'

// `?floor=empty` answers an untouched grid; absent, the fixture solves whatever the screen
// sends, the way the backend does — with one rule, so the development server can draw the
// screen without a Rust build.
export const FloorScenario = {
  Empty: 'empty',
  Solve: 'solve',
} as const
export type FloorScenario = (typeof FloorScenario)[keyof typeof FloorScenario]

const WIDTH = 13
const CELLS = 169

const QUOTE =
  'Secret Rooms are equally as likely to be in a valid location with 3 neighbors, as it is with 4 neighbors.'
const URL = 'https://bindingofisaacrebirth.wiki.gg/wiki/Secret_Room'

const neighbours = (cell: number): number[] => {
  const x = cell % WIDTH
  const out: number[] = []
  if (x > 0) out.push(cell - 1)
  if (x + 1 < WIDTH) out.push(cell + 1)
  if (cell - WIDTH >= 0) out.push(cell - WIDTH)
  if (cell + WIDTH < CELLS) out.push(cell + WIDTH)
  return out
}

export const floorAnswer = (
  scenario: FloorScenario,
  args?: CommandArgs,
): FloorView => {
  const cells = (args?.cells ?? []) as (RoomKindView | null)[]
  const painted = cells.filter((c) => c !== null).length
  if (scenario === FloorScenario.Empty || painted === 0) {
    return { solutions: [], painted: 0, diagnostics: [{ kind: 'gridEmpty' }] }
  }
  const candidates = cells
    .map((cell, index) => ({ cell: index, empty: cell === null }))
    .filter((c) => c.empty)
    .map((c) => ({
      cell: c.cell,
      neighbours: neighbours(c.cell).filter((n) => cells[n] !== null).length,
    }))
    .filter((c) => c.neighbours >= 2)
    .map((c) => ({
      cell: c.cell,
      neighbours: c.neighbours,
      rank: c.neighbours >= 3 ? 0 : 1,
      applied: [{ id: 'secret-neighbours', quote: QUOTE, url: URL }],
    }))
    .sort((a, b) => a.rank - b.rank || a.cell - b.cell)
  return {
    solutions: [{ target: 'secret', candidates, unresolved: [] }],
    painted,
    diagnostics: [],
  }
}
```

In `ui/src/lib/ipc/fixtures/index.ts`: import `FloorScenario` and `floorAnswer`, add the query
parameter next to the others (`const FloorParam = 'floor'`), read it the way `QueueParam` is
read, and add the `Command.FloorCandidates` branch to the command switch, passing `args`.
Follow the file's existing shape exactly — the switch is exhaustive and `assertNever` is its
last arm, so a missing branch is a type error and not a runtime surprise.

- [ ] **Step 4: Verify**

Run: `cargo build -p app`
Expected: builds.

Run: `pnpm typecheck`
Expected: no errors.

- [ ] **Step 5: Commit**

```bash
git add crates/app/src/commands/floor.rs crates/app/src/commands/mod.rs crates/app/src/lib.rs ui/src/lib/constants/commands.ts ui/src/lib/ipc/floor.ts ui/src/lib/ipc/fixtures/floor.ts ui/src/lib/ipc/fixtures/index.ts
git commit -m "feat(app): the floor_candidates command, and the fixture that answers it without a backend"
```

---

### Task 8: The screen

**Files:**
- Create: `ui/src/lib/floor/painting.ts`
- Create: `ui/src/lib/floor/painting.test.ts`
- Create: `ui/src/assets/theme/floor.css`
- Modify: `ui/src/assets/main.css`
- Create: `ui/src/stores/floor.ts`
- Create: `ui/src/screens/FloorScreen.vue`
- Create: `ui/src/screens/floor/FloorGrid.vue`
- Create: `ui/src/screens/floor/FloorLegend.vue`
- Modify: `ui/src/router/routeTable.ts`
- Modify: `ui/src/router/routes.ts`
- Modify: `ui/src/i18n/messages/it.ts`
- Modify: `ui/src/i18n/messages/en.ts`

**Interfaces:**
- Consumes: `floorCandidates` from Task 7, `FloorView`, `RoomKindView` from the generated types.
- Produces: `RouteName.Floor`, `useFloorStore` with `cells`, `brush`, `paint(cell)`, `erase(cell)`, `clear()`, `solve()`, `view`.

- [ ] **Step 1: Write the failing test for the painting logic**

The logic goes in `lib/`, tested by Vitest; the SFC only binds it. Create
`ui/src/lib/floor/painting.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import { CELLS, WIDTH, emptyCells, paintStroke, xy } from './painting'
import { RoomKindView } from '@/lib/ipc/types'

describe('the painted grid', () => {
  it('is 13 by 13', () => {
    expect(WIDTH).toBe(13)
    expect(CELLS).toBe(169)
    expect(emptyCells()).toHaveLength(CELLS)
    expect(emptyCells().every((c) => c === null)).toBe(true)
  })

  it('places a cell at the row and column its index says', () => {
    expect(xy(84)).toEqual({ x: 6, y: 6 })
    expect(xy(0)).toEqual({ x: 0, y: 0 })
    expect(xy(168)).toEqual({ x: 12, y: 12 })
  })

  it('paints every cell a stroke crosses, once each', () => {
    const before = emptyCells()
    const after = paintStroke(before, [1, 2, 2, 3], RoomKindView.Normal)
    expect(after[1]).toBe(RoomKindView.Normal)
    expect(after[2]).toBe(RoomKindView.Normal)
    expect(after[3]).toBe(RoomKindView.Normal)
    expect(after[0]).toBeNull()
  })

  it('does not change the array it was given', () => {
    const before = emptyCells()
    paintStroke(before, [5], RoomKindView.Boss)
    expect(before[5]).toBeNull()
  })

  it('erases with a null brush instead of a second function', () => {
    const painted = paintStroke(emptyCells(), [7], RoomKindView.Shop)
    expect(paintStroke(painted, [7], null)[7]).toBeNull()
  })

  it('ignores a cell outside the grid rather than growing the array', () => {
    const after = paintStroke(emptyCells(), [-1, CELLS, 999], RoomKindView.Normal)
    expect(after).toHaveLength(CELLS)
    expect(after.every((c) => c === null)).toBe(true)
  })
})
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `pnpm ui:test -- painting`
Expected: FAIL — the module does not exist.

- [ ] **Step 3: Write the painting module**

Create `ui/src/lib/floor/painting.ts`:

```ts
import type { RoomKindView } from '@/lib/ipc/types'

// The grid the game uses: 13 wide, 13 tall, a cell's index is y * WIDTH + x. The start room
// is 84, which is what the game prints on every floor.
export const WIDTH = 13
export const HEIGHT = 13
export const CELLS = WIDTH * HEIGHT
export const START = 84

export type PaintedCells = (RoomKindView | null)[]

export const emptyCells = (): PaintedCells => Array<RoomKindView | null>(CELLS).fill(null)

export const xy = (cell: number): { x: number; y: number } => ({
  x: cell % WIDTH,
  y: Math.floor(cell / WIDTH),
})

// A stroke is the cells the pointer crossed, in order and with repeats; a null brush erases.
// It answers a new array, because the store's state is replaced rather than mutated.
export const paintStroke = (
  cells: PaintedCells,
  stroke: number[],
  brush: RoomKindView | null,
): PaintedCells => {
  const next = [...cells]
  for (const cell of stroke) {
    if (!Number.isInteger(cell) || cell < 0 || cell >= CELLS) continue
    next[cell] = brush
  }
  return next
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `pnpm ui:test -- painting`
Expected: 6 passed.

- [ ] **Step 5: Add the tokens**

Create `ui/src/assets/theme/floor.css`. Read `ui/src/assets/theme/colors.css` first and follow
its exact shape for light and dark; the names below are the contract the screen binds to.

```css
@theme {
  --size-floor-cell: 2rem;
  --size-floor-gap: 0.125rem;
  --color-floor-empty: var(--color-muted);
  --color-floor-room: var(--color-card);
  --color-floor-candidate-first: var(--color-chart-1);
  --color-floor-candidate-second: var(--color-chart-2);
  --color-floor-candidate-third: var(--color-chart-3);
}
```

If `--color-chart-1` does not exist in this repo, define three colours here in the same
light/dark form `colors.css` uses — **do not** put a hex value in a template.

In `ui/src/assets/main.css`, add the import next to the other theme imports, in the order the
file already keeps them:

```css
@import './theme/floor.css';
```

- [ ] **Step 6: Add the store**

Create `ui/src/stores/floor.ts`:

```ts
import { defineStore } from 'pinia'
import { ref, shallowRef } from 'vue'
import { floorCandidates } from '@/lib/ipc/floor'
import { emptyCells, paintStroke, type PaintedCells } from '@/lib/floor/painting'
import type { FloorView, RoomKindView } from '@/lib/ipc/types'
import { StoreId } from '@/lib/constants/stores'

// The painted floor is a scratchpad, not a document: it lives here and nowhere else, and it is
// gone when the app closes. Persisting it would outlive the floor it describes.
export const useFloorStore = defineStore(StoreId.Floor, () => {
  const cells = ref<PaintedCells>(emptyCells())
  const brush = ref<RoomKindView | null>(null)
  const view = shallowRef<FloorView | null>(null)
  const failed = ref(false)

  const solve = async (): Promise<void> => {
    try {
      view.value = await floorCandidates(cells.value)
      failed.value = false
    } catch {
      view.value = null
      failed.value = true
    }
  }

  const stroke = async (path: number[]): Promise<void> => {
    cells.value = paintStroke(cells.value, path, brush.value)
    await solve()
  }

  const clear = async (): Promise<void> => {
    cells.value = emptyCells()
    await solve()
  }

  return { cells, brush, view, failed, stroke, clear, solve }
})
```

Add `Floor: 'floor',` to the `StoreId` object in `ui/src/lib/constants/stores.ts` (that is where
it lives — `ui/src/stores/` holds the stores, not the ids), keeping the file's order.

This store is **not** a `defineViewStore`: that helper is for a screen that only reads a
command's answer, and this one owns state the user types in. A plain `defineStore` is right
here, and it is the first one in the repo of that shape outside `tabs`/`settings`.

- [ ] **Step 7: Add the route**

In `ui/src/router/routeTable.ts`:
- add `Floor: 'floor',` to `RouteName`, after `Live`;
- add `[RouteName.Floor]: '/progress/floor',` to `routePath`;
- add `[RouteName.Floor]: 'routes.floor',` to `routeTitle`;
- add `[RouteName.Floor]: TabOrigin.Progress,` to `routeOrigin`;
- add `[RouteName.Floor]: Grid3x3Icon,` to `routeIcon`, importing `Grid3x3Icon` from
  `@lucide/vue` in the existing alphabetical import list. **`@lucide/vue`, never
  `lucide-vue-next`.**
- do **not** add anything to `routeArrives`: the screen is real from this task on.

In `ui/src/router/routes.ts`, import `FloorScreen` and add `[RouteName.Floor]: FloorScreen,` to
the `screens` record.

`routeOrigin` decides `needsProfile`, and `TabOrigin.Progress` means the screen is gated behind
a chosen profile. **That is wrong for this screen** — painting a grid needs no save file. Use
`TabOrigin.Progress` for where it sits in the navigation, then exclude it in `routes.ts` by
changing the `needsProfile` line to:

```ts
      needsProfile: routeOrigin[name] === TabOrigin.Progress && name !== RouteName.Floor,
```

and add a comment above it saying why: the grid is the user's own drawing and answers without a
profile, which is the only Progress screen of which that is true.

- [ ] **Step 8: Add the messages**

In `ui/src/i18n/messages/it.ts`, add `floor: 'Piano di gioco',` to the `routes` block and a new
top-level block:

```ts
  floor: {
    intro:
      'Disegna la mappa del piano come la vedi sulla minimappa: le celle vuote si accendono dove le regole del gioco permettono una stanza segreta.',
    brush: 'Stanza da disegnare',
    erase: 'Cancella',
    clear: 'Svuota la griglia',
    painted: 'stanze disegnate',
    secret: 'Stanza segreta',
    superSecret: 'Super segreta',
    ultraSecret: 'Ultra segreta',
    neighbours: 'stanze adiacenti',
    source: 'Fonte',
    unresolved: 'Quello che la griglia non può giudicare',
    empty: 'Disegna almeno una stanza.',
    noStart: 'Segna la stanza di partenza: senza, la regola sui passi dalla partenza non si può leggere.',
    rulesUnreadable: 'Le regole di piazzamento non si sono caricate.',
    failed: 'Non è stato possibile calcolare i candidati.',
  },
```

Add the same keys to `ui/src/i18n/messages/en.ts` with English text. **Both files, same shape** —
`en.ts` types the schema and a missing key is a type error.

- [ ] **Step 9: Write the screen**

Create `ui/src/screens/floor/FloorGrid.vue` — the grid alone, and nothing else. It takes
`cells`, `candidates` and emits a stroke; it holds no store. Bind the cell size through a CSS
variable set from the token, never an inline pixel value:

```vue
<script setup lang="ts">
import { computed } from 'vue'
import { CELLS, WIDTH } from '@/lib/floor/painting'
import type { PaintedCells } from '@/lib/floor/painting'
import type { FloorCandidate } from '@/lib/ipc/types'

const props = defineProps<{ cells: PaintedCells; candidates: FloorCandidate[] }>()
const emit = defineEmits<{ stroke: [path: number[]] }>()

const rankOf = computed(() => {
  const m = new Map<number, number>()
  for (const c of props.candidates) m.set(c.cell, c.rank)
  return m
})

const indexes = Array.from({ length: CELLS }, (_, i) => i)

let path: number[] = []
let painting = false

const start = (cell: number): void => {
  painting = true
  path = [cell]
}
const over = (cell: number): void => {
  if (painting) path.push(cell)
}
const end = (): void => {
  if (!painting) return
  painting = false
  if (path.length > 0) emit('stroke', path)
  path = []
}
</script>

<template>
  <div
    class="grid gap-(--size-floor-gap)"
    :style="{ gridTemplateColumns: `repeat(${WIDTH}, var(--size-floor-cell))` }"
    @pointerup="end"
    @pointerleave="end"
  >
    <button
      v-for="i in indexes"
      :key="i"
      type="button"
      class="size-(--size-floor-cell) rounded-xs"
      :class="[
        cells[i] === null ? 'bg-floor-empty' : 'bg-floor-room',
        rankOf.get(i) === 0 ? 'ring-2 ring-floor-candidate-first' : '',
        rankOf.get(i) === 1 ? 'ring-2 ring-floor-candidate-second' : '',
      ]"
      @pointerdown="start(i)"
      @pointerenter="over(i)"
    />
  </div>
</template>
```

**Two rules to honour while writing this file.** Rule 4 forbids a raw `<button>`: use the
primitive in `ui/src/components/ui/button` with whatever prop makes it unstyled, or add a
variant to it — do not style a bare element by hand, and do not invent a second button. Rule 2
forbids `ring-2` if that is not already a token in this repo; check `ui/src/assets/theme/` and
use the token that exists. The structure above is the shape; the primitives are the repo's.

Create `ui/src/screens/floor/FloorLegend.vue` — the brush picker and one target's candidates:

```ts
defineProps<{ brush: RoomKindView | null; solution: FloorSolutionView | null }>()
defineEmits<{ pick: [brush: RoomKindView | null] }>()
```

It renders one toggle per `RoomKindView` plus an erase toggle (`brush === null`, label
`floor.erase`), then the solution's candidates as rows — `cell`, `neighbours` with the label
`floor.neighbours`, and under each the `applied` rules' `quote` and `url` with `floor.source` as
the label — then `unresolved` under the `floor.unresolved` heading. Use `ToggleGroup` from
`ui/src/components/ui/toggle-group` for the picker; it is already in the repo.

Create `ui/src/screens/FloorScreen.vue` to bind them to the store. Follow `LiveScreen.vue`
exactly for the outer shape: `ScreenHeader` with `Grid3x3Icon` and `t('floor.intro')`,
`DiagnosticsList` for `store.view?.diagnostics`, a `Card` for the grid and one per target,
`EmptyCategory` when a target has no candidate, and `store.failed` drawn with the `floor.failed`
message rather than an empty screen. Call `store.solve()` once on setup, the way `LiveScreen`
calls `store.load()`.

**Every candidate list shows the quotation and the URL** of its rules — that is the attribution
the CC BY-SA licence requires, not a detail of the layout, and `floor.source` is its label.

- [ ] **Step 10: Verify**

Run: `pnpm ui:test`
Expected: all green.

Run: `pnpm scan`
Expected: `0 violations`. If it reports one, **fix the code**; add an `EXEMPTIONS` entry only
with a written reason, and only if the rule genuinely cannot apply.

Run: `pnpm lint && pnpm typecheck && pnpm format:check`
Expected: clean.

Run: `pnpm ui:dev` and open `#/progress/floor`, plus `?floor=empty`. Paint a plus of five
rooms and check that cells with three neighbours are ringed before cells with two.

- [ ] **Step 11: Commit**

```bash
git add ui/src/lib/floor ui/src/assets/theme/floor.css ui/src/assets/main.css ui/src/stores/floor.ts ui/src/stores/storeId.ts ui/src/screens/FloorScreen.vue ui/src/screens/floor ui/src/router/routeTable.ts ui/src/router/routes.ts ui/src/i18n/messages/it.ts ui/src/i18n/messages/en.ts
git commit -m "feat(ui): Floor, the grid you paint and the rules that light it"
```

---

### Task 9: The way in from Live, the documents, and the whole suite

**Files:**
- Modify: `ui/src/screens/LiveScreen.vue`
- Modify: `ui/src/i18n/messages/it.ts`, `ui/src/i18n/messages/en.ts`
- Modify: `docs/STATUS.md`
- Modify: `docs/BACKLOG.md`
- Modify: `CLAUDE.md`

**Interfaces:**
- Consumes: `RouteName.Floor`.
- Produces: nothing further; this is the task that closes F1.

- [ ] **Step 1: Add the link from Live**

In `ui/src/screens/LiveScreen.vue`, add a line under the run's KPI row that navigates to
`RouteName.Floor`. **Do not invent a second way to open a screen**: the shell has a tab model
(`ui/src/stores/tabs.ts`, `ui/src/components/shell/tabs.ts`, `TabLocation` in
`ui/src/router/routeTable.ts`) and at least one screen already links to another — find it with
`grep -rn "TabLocation\|RouterLink" ui/src/screens` and copy that mechanism exactly. Its label
is a new message, `live.floorLink`, in both message files:

```ts
    floorLink: 'Dov’è la segreta su questo piano',
```

(English: `'Where the secret room is on this floor'`.)

- [ ] **Step 2: Update the documents**

In `docs/STATUS.md`, add the sub-project to the milestone list with its checkbox **checked only
after the commit lands** — a checked box in that file means committed work, never a note.

In `docs/BACKLOG.md`, under B8's new blockquote, add one line saying F1 has landed and naming
F2 as the remaining half.

In `CLAUDE.md`:
- add `floor` to the crate list in the **Layout** paragraph;
- add a row to the **Modules** table:

```markdown
| `floor` | Pure crate: the 13x13 grid a player paints and the game's own documented secret-room placement rules, each carrying the sentence it was read from. Knows nothing about the log: the grid is the user's drawing. |
```

- in the **State** section, add one sentence naming the Floor screen and the spec it came from.

- [ ] **Step 3: Run the whole suite**

Run: `pnpm check`
Expected: green — `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo test --workspace`, `pnpm typecheck`, `pnpm ui:test`, `pnpm lint`, `pnpm format:check`,
`pnpm scan`.

Run: `cargo test --workspace -- --nocapture 2>&1 | grep -c "^skip:"`
Expected: the same number as before this branch. **An "N passed" does not say how many were
skipped** — if the count went up, a test you wrote is skipping when it should run.

Do **not** run this while `live_probe` is running in the same profile: the example's exe is held
open and the link fails with `LNK1104`.

- [ ] **Step 4: Commit**

```bash
git add ui/src/screens/LiveScreen.vue ui/src/i18n/messages/it.ts ui/src/i18n/messages/en.ts docs/STATUS.md docs/BACKLOG.md CLAUDE.md
git commit -m "docs: Floor lands, and Live points at it"
```

- [ ] **Step 5: Finish the branch**

Use the `superpowers:finishing-a-development-branch` skill. The integration branch is
**`develop`**; `master` only receives releases.

---

## What F1 does not do

Named here so no one goes looking for it in a task: the log half. No new pattern in
`crates/run/rules/events.json`, no `generated_rooms`, no `LiveView.currentFloor`, no clearing of
the grid on `Level::Init`, and no sentence anywhere that subtracts the painted count from a
count the game printed. That is F2, and §10 of the spec says what it is.
