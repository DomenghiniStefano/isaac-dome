# Graph — mark and counter requirements: implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement
> this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let the graph express the two prerequisites it cannot say today — "beat this boss
with this character" and "have this tally at least N" — so that 138 of the 178 references it
currently declares uninterpreted become answerable from the save.

**Architecture:** Two new `Requirement` variants resolved from one new curated verdict
(`Progress`, holding an optional mark half and an optional counter half). `graph` stays pure
and symbolic: it names a column, a level and a counter, never an index. The save's layout
moves out of the view-model module `ipc::marks` into `core-save`, where the file's shape
belongs, and `ipc` implements the `Profile` trait `graph::evaluate` now takes.

**Tech Stack:** Rust 2021, `serde` / `serde_json`, the workspace's existing crates
(`graph`, `ipc`, `core-save`, `catalog`, `test-support`); TypeScript for the IPC mirror.

**Spec:** `docs/superpowers/specs/2026-09-12-graph-mark-requirements-design.md`

## Global Constraints

Copied from the spec's §3; every task's requirements include these.

- **A name is structural or measured, never plausible.** `MarkLevel` is `Base` / `Second`
  (bit 0, bit 1). Never `Hard`: that would assert something §2.2 does not measure.
- **`graph` stays pure**: no new dependency, no `core-save`, no I/O. Its `Cargo.toml`
  dependencies stay `catalog`, `wiki`, `serde`, `serde_json`.
- **No offsets in a rules file, none across the IPC.** `corrections.json` says
  `"hushKills"`; the number 158 lives in `core-save`.
- **Degrade, never fail.** Missing or short section 2, or an unlocated cell → `Unknown`,
  never "satisfied". Assert on the value, never on the absence of a panic.
- **Exhaustiveness**: no `_ =>` arm on any closed enum touched here.
- **IPC**: `#[serde(rename_all = "camelCase")]` on structs; tagged enums with struct variants
  also need `rename_all_fields = "camelCase"`; **fieldless enums are bare camelCase strings**,
  never tagged (`MarkColumn`, `MarkLevel`, `CounterName`).
- **Never `panic!`/`unwrap()`** outside tests on data read from disk.
- Before declaring anything done: `pnpm check` (`scripts/check`).
- Commits: Conventional Commits, `type(scope): subject`, English, atomic. **Never** a
  `Co-Authored-By` trailer.

---

### Task 1: The curated vocabulary — `MarkColumn`, `MarkLevel`, `CounterName`

**Files:**
- Modify: `crates/graph/src/rules.rs` (after the `Verdict` enum, around line 85)
- Test: `crates/graph/tests/rules.rs`

**Interfaces:**
- Consumes: nothing.
- Produces: `graph::rules::{MarkColumn, MarkLevel, CounterName, MarkRule, CounterRule}` and
  the new variant `Verdict::Progress { mark: Option<MarkRule>, counter: Option<CounterRule> }`.

- [ ] **Step 1: Write the failing test**

In `crates/graph/tests/rules.rs`:

```rust
#[test]
fn a_progress_verdict_parses_both_halves() {
    let json = r#"{
        "schemaVersion": 1,
        "verdicts": {
            "entity:Hush": { "progress": {
                "mark": { "column": "hush", "level": "base" },
                "counter": { "name": "hushKills", "atLeast": 1 }
            } }
        }
    }"#;
    let c: graph::Corrections = serde_json::from_str(json).expect("parses");
    let v = c.verdicts.get("entity:Hush").expect("the verdict is there");
    assert_eq!(
        v,
        &graph::rules::Verdict::Progress {
            mark: Some(graph::rules::MarkRule {
                column: graph::rules::MarkColumn::Hush,
                level: graph::rules::MarkLevel::Base,
            }),
            counter: Some(graph::rules::CounterRule {
                name: graph::rules::CounterName::HushKills,
                at_least: 1,
            }),
        }
    );
}

#[test]
fn a_progress_verdict_may_carry_only_the_mark() {
    let json = r#"{
        "schemaVersion": 1,
        "verdicts": {
            "entity:Ultra Greedier": { "progress": {
                "mark": { "column": "greed", "level": "second" }
            } }
        }
    }"#;
    let c: graph::Corrections = serde_json::from_str(json).expect("parses");
    let Some(graph::rules::Verdict::Progress { mark, counter }) =
        c.verdicts.get("entity:Ultra Greedier")
    else {
        panic!("expected a progress verdict");
    };
    assert_eq!(mark.as_ref().map(|m| m.level), Some(graph::rules::MarkLevel::Second));
    assert!(counter.is_none(), "an absent half stays absent, it is not defaulted");
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p graph --test rules -- a_progress_verdict`
Expected: FAIL to compile — `no variant named Progress`, `MarkRule` not found.

- [ ] **Step 3: Write the minimal implementation**

In `crates/graph/src/rules.rs`, add to the `Verdict` enum and below it:

```rust
    /// Answered by the profile rather than by another achievement: a cell of the
    /// completion matrix, a tally of section 2, or both. Both halves optional because a
    /// reference that names a character wants the mark and one that names only the boss
    /// wants the counter — and a target rarely has both located.
    Progress {
        #[serde(default)]
        mark: Option<MarkRule>,
        #[serde(default)]
        counter: Option<CounterRule>,
    },
}

/// The twelve columns of the completion matrix, in the game's own order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MarkColumn {
    MomsHeart,
    Isaac,
    Satan,
    BossRush,
    BlueBaby,
    TheLamb,
    MegaSatan,
    Greed,
    Hush,
    Delirium,
    Mother,
    TheBeast,
}

/// A level within a cell, named for the bit and not for a meaning. Bit 0 is `Base`, bit 1
/// is `Second`. `Second` is Ultra Greedier in the Greed column — measured; what it means
/// in the other eleven columns is not measured, and a name like `Hard` would assert it.
///
/// Ordered, `Base` before `Second`, so a cell reached at the second level satisfies a
/// requirement for the base one by `reached >= required`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MarkLevel {
    Base,
    Second,
}

/// A tally of section 2, named. The index it lives at is `core-save`'s business.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CounterName {
    HushKills,
    DeliriumKills,
    MotherKills,
    BeastKills,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkRule {
    pub column: MarkColumn,
    pub level: MarkLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CounterRule {
    pub name: CounterName,
    pub at_least: u32,
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p graph --test rules -- a_progress_verdict`
Expected: PASS, 2 tests.

- [ ] **Step 5: Reject a `Progress` with neither half**

Add to `crates/graph/tests/rules.rs`:

```rust
#[test]
fn a_progress_verdict_with_neither_half_is_malformed() {
    let r = graph::Requirements {
        schema_version: graph::SCHEMA_VERSION,
        ..Default::default()
    };
    let mut c = graph::Corrections {
        schema_version: 1,
        ..Default::default()
    };
    c.verdicts.insert(
        "entity:Nothing".to_string(),
        graph::rules::Verdict::Progress { mark: None, counter: None },
    );
    let err = graph::Rules::build(r, c).expect_err("an empty progress answers nothing");
    assert!(
        matches!(err, graph::RulesError::Malformed { .. }),
        "expected Malformed, got {err:?}"
    );
}
```

Then in `Rules::build`, before it returns `Ok`:

```rust
        for (key, v) in &c.verdicts {
            if let Verdict::Progress { mark: None, counter: None } = v {
                return Err(RulesError::Malformed {
                    reason: format!("{key}: a progress verdict with neither half answers nothing"),
                });
            }
        }
```

If `Requirements` / `Corrections` do not already derive `Default`, add `#[derive(Default)]`
to both rather than hand-building them in the test.

- [ ] **Step 6: Run the whole crate's tests**

Run: `cargo test -p graph`
Expected: PASS. The existing rules tests must be untouched — the new variant is additive.

- [ ] **Step 7: Commit**

```bash
git add crates/graph/src/rules.rs crates/graph/tests/rules.rs
git commit -m "feat(graph): a curated verdict the profile answers"
```

---

### Task 2: The two domain requirements

**Files:**
- Modify: `crates/graph/src/model.rs`
- Modify: `crates/graph/src/resolve.rs` (`from_verdict`, around line 131)
- Test: `crates/graph/tests/resolve.rs`

**Interfaces:**
- Consumes: Task 1's `MarkRule`, `CounterRule`, `Verdict::Progress`.
- Produces: `Requirement::Mark { character: CharacterId, column: MarkColumn, level: MarkLevel }`
  and `Requirement::Counter { name: CounterName, at_least: u32 }`.

**Key decision this task encodes:** which half is used is decided by the *reference*, not by
the target. `resolve` already receives the label; the caller knows whether the achievement's
reference list also names a character. So `from_verdict` grows a parameter: the character the
same achievement names, if any.

- [ ] **Step 1: Write the failing tests**

In `crates/graph/tests/resolve.rs`:

```rust
// The reference names a character: the mark half answers it.
#[test]
fn a_boss_with_a_character_resolves_to_a_mark() {
    let Some((c, _rs)) = support::real_catalog() else { return };
    let rules = graph::rules::embedded().expect("rules");
    let index = graph::resolve::NameIndex::new(&c);
    let row = graph::rules::RefRow {
        target: wiki::Target::Entity { id: 0, variant: 0, subtype: 0 },
        label: "Ultra Greedier".to_string(),
    };
    let magdalene = index.character("Magdalene").expect("Magdalene is in the catalog");
    let r = graph::resolve::requirement_with(&c, rules, &index, &row, Some(magdalene));
    assert_eq!(
        r,
        graph::model::Requirement::Mark {
            character: magdalene,
            column: graph::rules::MarkColumn::Greed,
            level: graph::rules::MarkLevel::Second,
        }
    );
}

// The reference names the boss alone: the counter half answers it.
#[test]
fn a_boss_without_a_character_resolves_to_a_counter() {
    let Some((c, _rs)) = support::real_catalog() else { return };
    let rules = graph::rules::embedded().expect("rules");
    let index = graph::resolve::NameIndex::new(&c);
    let row = graph::rules::RefRow {
        target: wiki::Target::Entity { id: 0, variant: 0, subtype: 0 },
        label: "Hush".to_string(),
    };
    let r = graph::resolve::requirement_with(&c, rules, &index, &row, None);
    assert_eq!(
        r,
        graph::model::Requirement::Counter {
            name: graph::rules::CounterName::HushKills,
            at_least: 1,
        }
    );
}

// The half the reference needs is absent: declared, never dropped.
#[test]
fn a_missing_half_is_unknown_and_keeps_its_label() {
    let Some((c, _rs)) = support::real_catalog() else { return };
    let rules = graph::rules::embedded().expect("rules");
    let index = graph::resolve::NameIndex::new(&c);
    let row = graph::rules::RefRow {
        target: wiki::Target::Entity { id: 0, variant: 0, subtype: 0 },
        label: "Ultra Greedier".to_string(),
    };
    // Ultra Greedier has no located tally, so the boss alone cannot be answered.
    let r = graph::resolve::requirement_with(&c, rules, &index, &row, None);
    assert_eq!(
        r,
        graph::model::Requirement::Unknown { label: "Ultra Greedier".to_string() }
    );
}
```

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test -p graph --test resolve`
Expected: FAIL to compile — `requirement_with` takes 4 arguments, not 5.

- [ ] **Step 3: Add the variants**

In `crates/graph/src/model.rs`, inside `enum Requirement`:

```rust
    /// One cell of the completion matrix: "beat `column` as `character`, at `level`".
    /// Symbolic: the cell's index is not this crate's business.
    Mark {
        character: CharacterId,
        column: crate::rules::MarkColumn,
        level: crate::rules::MarkLevel,
    },
    /// A tally of section 2, at or above a threshold. Named, never indexed.
    Counter {
        name: crate::rules::CounterName,
        at_least: u32,
    },
```

- [ ] **Step 4: Route the verdict**

In `crates/graph/src/resolve.rs`, change the signature and `from_verdict`:

```rust
pub fn requirement_with(
    c: &Catalog,
    rules: &Rules,
    index: &NameIndex,
    row: &RefRow,
    character: Option<CharacterId>,
) -> Requirement {
```

Pass `character` down into every `from_verdict(rules, &verdict_key, unknown)` call, and:

```rust
fn from_verdict(
    rules: &Rules,
    key: &str,
    character: Option<CharacterId>,
    unknown: impl Fn() -> Requirement,
) -> Requirement {
    match rules.verdict(key) {
        Some(Verdict::AlwaysAvailable(_)) | Some(Verdict::NotAPrerequisite(_)) => Requirement::None,
        Some(Verdict::Behind { .. }) => Requirement::Gate { gate: key.to_string() },
        // Which half answers is decided by the reference, not by the target: a reference
        // that names a character asks about that character's cell, one that names only the
        // boss asks whether it was ever done at all. The half it needs may be absent, and
        // then the answer is the same as for anything else this crate can't say.
        Some(Verdict::Progress { mark, counter }) => match (character, mark, counter) {
            (Some(ch), Some(m), _) => Requirement::Mark {
                character: ch,
                column: m.column,
                level: m.level,
            },
            (None, _, Some(c)) => Requirement::Counter {
                name: c.name,
                at_least: c.at_least,
            },
            (Some(_), None, _) | (None, _, None) => unknown(),
        },
        Some(Verdict::Unknown { .. }) | None => unknown(),
    }
}
```

Also update the one-shot `requirement(c, rules, row)` wrapper to take and forward
`character: Option<CharacterId>`.

- [ ] **Step 5: Fix the call site in `build.rs`**

`Graph::build` walks each achievement's references. Before resolving them, find the character
the same achievement names, and pass it to every reference of that achievement:

```rust
        // The character an achievement names applies to the whole sentence: "defeat Mother
        // as Magdalene" is one requirement in two references, and the boss reference is the
        // one that needs to know.
        let character = rules.refs(id).iter().find_map(|r| match &r.target {
            Target::Character { .. } => index.character(rules.alias(&r.label)),
            _ => None,
        });
```

- [ ] **Step 6: Run the tests**

Run: `cargo test -p graph --test resolve -- --nocapture`
Expected: PASS (or `skip:` lines if `samples/packed` is absent — then run it on a machine
with the game before moving on, because these three tests are the task).

- [ ] **Step 7: Commit**

```bash
git add crates/graph/src/model.rs crates/graph/src/resolve.rs crates/graph/src/build.rs crates/graph/tests/resolve.rs
git commit -m "feat(graph): resolve a boss reference to a mark or a tally"
```

---

### Task 3: The five curated rows

**Files:**
- Modify: `crates/graph/rules/corrections.json`
- Test: `crates/graph/tests/curation.rs`

**Interfaces:**
- Consumes: Tasks 1 and 2.
- Produces: the five `entity:` targets answered instead of declared unknown.

- [ ] **Step 1: Write the failing test**

In `crates/graph/tests/curation.rs`:

```rust
/// The five targets that §2 of the spec measured. Each is listed with the half it must
/// carry: `Ultra Greedier` has no located tally, so it carries the mark alone.
#[test]
fn the_five_progress_targets_are_curated() {
    let rules = graph::rules::embedded().expect("rules");
    let expected: &[(&str, graph::rules::MarkColumn, bool)] = &[
        ("entity:Hush", graph::rules::MarkColumn::Hush, true),
        ("entity:Delirium", graph::rules::MarkColumn::Delirium, true),
        ("entity:Mother", graph::rules::MarkColumn::Mother, true),
        ("entity:The Beast", graph::rules::MarkColumn::TheBeast, true),
        ("entity:Ultra Greedier", graph::rules::MarkColumn::Greed, false),
    ];
    for (key, column, has_counter) in expected {
        let Some(graph::rules::Verdict::Progress { mark, counter }) = rules.verdict(key) else {
            panic!("{key} must carry a progress verdict");
        };
        assert_eq!(mark.as_ref().map(|m| m.column), Some(*column), "{key}");
        assert_eq!(counter.is_some(), *has_counter, "{key}");
    }
}

/// Ultra Greedier is the one whose level is not the base bit, and that is the measurement
/// of 2026-09-12. A regression here is a silent loss of 34 answered nodes.
#[test]
fn ultra_greedier_is_the_second_level_of_the_greed_column() {
    let rules = graph::rules::embedded().expect("rules");
    let Some(graph::rules::Verdict::Progress { mark: Some(m), .. }) =
        rules.verdict("entity:Ultra Greedier")
    else {
        panic!("entity:Ultra Greedier must carry a mark");
    };
    assert_eq!(m.column, graph::rules::MarkColumn::Greed);
    assert_eq!(m.level, graph::rules::MarkLevel::Second);
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p graph --test curation`
Expected: FAIL — the five keys still hold `Verdict::Unknown`.

- [ ] **Step 3: Replace the five rows in `crates/graph/rules/corrections.json`**

Replace each of the five existing `"unknown"` entries with:

```json
    "entity:Hush": { "progress": {
      "mark": { "column": "hush", "level": "base" },
      "counter": { "name": "hushKills", "atLeast": 1 }
    } },
    "entity:Delirium": { "progress": {
      "mark": { "column": "delirium", "level": "base" },
      "counter": { "name": "deliriumKills", "atLeast": 1 }
    } },
    "entity:Mother": { "progress": {
      "mark": { "column": "mother", "level": "base" },
      "counter": { "name": "motherKills", "atLeast": 1 }
    } },
    "entity:The Beast": { "progress": {
      "mark": { "column": "theBeast", "level": "base" },
      "counter": { "name": "beastKills", "atLeast": 1 }
    } },
    "entity:Ultra Greedier": { "progress": {
      "mark": { "column": "greed", "level": "second" }
    } },
```

Keep the file's alphabetical key order. Do not remove the `reason` strings by deleting them
silently — move each one into the commit body, which is where the "why" now lives.

- [ ] **Step 4: Run the tests**

Run: `cargo test -p graph --test curation`
Expected: PASS, 2 tests.

- [ ] **Step 5: Commit**

```bash
git add crates/graph/rules/corrections.json crates/graph/tests/curation.rs
git commit -m "feat(graph): curate the five bosses the profile can answer"
```

---

### Task 4: The save's layout moves to `core-save`

**Files:**
- Create: `crates/core-save/src/marks.rs`
- Modify: `crates/core-save/src/lib.rs` (declare and re-export the module)
- Modify: `crates/ipc/src/marks.rs` (delete the moved constants, import them)
- Test: `crates/core-save/tests/marks_layout.rs`

**Interfaces:**
- Consumes: nothing.
- Produces:
  - `core_save::marks::{Column, Row, CHARACTER_ROWS, cell_index, counter_index_of}`
  - `cell_index(row: usize, column: Column) -> Option<usize>`
  - `counter_index_of(name: &str) -> Option<usize>`

**Why:** the block bases and `counter_index` are knowledge of the file's shape and currently
live in a view-model module. Moving them is what gives Task 7's `Profile` implementation an
obvious home instead of a convenient one. This is a move, not a redesign: the tables' values
do not change.

- [ ] **Step 1: Write the failing test**

In `crates/core-save/tests/marks_layout.rs`:

```rust
use core_save::marks::{cell_index, counter_index_of, Column};

/// The bases pinned on 2026-09-08 and re-derived on 2026-09-12 (spec §2.3). Row numbers are
/// positions in the 34-row order: Magdalene is 1, Cain is 2, Keeper is 12.
#[test]
fn the_located_bases_are_where_the_series_put_them() {
    assert_eq!(cell_index(1, Column::Mother), Some(424), "Mother base 423 + Magdalene");
    assert_eq!(cell_index(2, Column::Mother), Some(425), "Mother base 423 + Cain");
    assert_eq!(cell_index(1, Column::TheBeast), Some(458), "Beast base 457 + Magdalene");
    assert_eq!(cell_index(12, Column::Greed), Some(142), "Greed base 130 + Keeper");
}

/// Spec §2.4: Mother and The Beast for The Forgotten and the 19 are not located. The layout
/// says so rather than returning a plausible index.
#[test]
fn the_unlocated_cells_answer_none() {
    assert_eq!(cell_index(14, Column::Mother), None, "The Forgotten");
    assert_eq!(cell_index(15, Column::Mother), None, "Bethany");
    assert_eq!(cell_index(33, Column::TheBeast), None, "T. Jacob");
}

#[test]
fn the_four_named_tallies_have_their_indices() {
    assert_eq!(counter_index_of("hushKills"), Some(158));
    assert_eq!(counter_index_of("deliriumKills"), Some(187));
    assert_eq!(counter_index_of("motherKills"), Some(491));
    assert_eq!(counter_index_of("beastKills"), Some(492));
    assert_eq!(counter_index_of("nothing"), None);
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p core-save --test marks_layout`
Expected: FAIL to compile — `core_save::marks` does not exist.

- [ ] **Step 3: Create the module**

Move `BOSSES`-as-order, `CHARACTERS`-as-order, `BLOCKS_14`, `FORGOTTEN`, `BLOCKS_19`,
`FIRST_LATER` and `counter_index` verbatim from `crates/ipc/src/marks.rs` into
`crates/core-save/src/marks.rs`. Keep the values byte-for-byte; this task must not change a
single index. Then:

- rename `counter_index(character, boss)` to `cell_index(row: usize, column: Column)`, where
  `Column` is a `#[derive(Clone, Copy, PartialEq, Eq)] pub enum` of the twelve, with
  `fn position(self) -> usize` giving its index into the block tables;
- add the tally table:

```rust
/// The four located tallies of section 2. Names match `graph::rules::CounterName`'s wire
/// form; the mapping lives here because an index into the save is this crate's business and
/// nobody else's.
const TALLIES: &[(&str, usize)] = &[
    ("hushKills", 158),
    ("deliriumKills", 187),
    ("motherKills", 491),
    ("beastKills", 492),
];

pub fn counter_index_of(name: &str) -> Option<usize> {
    TALLIES.iter().find(|(n, _)| *n == name).map(|(_, i)| *i)
}
```

- keep the character *names* in `ipc::marks` — they are labels for a screen. What moves is
  the order and the bases.

In `crates/core-save/src/lib.rs` add `pub mod marks;`.

- [ ] **Step 4: Point `ipc::marks` at the moved tables**

In `crates/ipc/src/marks.rs`, delete the moved constants and replace the body of its own
`counter_index` with a call to `core_save::marks::cell_index`. `ipc::marks::BOSSES` and
`CHARACTERS` stay: they are the screen's labels, and their order must keep matching the
layout's — assert that in the same test file as Step 1 if it is not already asserted.

- [ ] **Step 5: Run both crates' tests**

Run: `cargo test -p core-save -p ipc`
Expected: PASS. Completion's existing tests (`crates/ipc/tests/marks_real.rs`,
`crates/ipc/tests/marks.rs`) must pass **unchanged** — if one of them needed editing, an
index moved, and that is a bug in this task, not in the test.

- [ ] **Step 6: Commit**

```bash
git add crates/core-save/src/marks.rs crates/core-save/src/lib.rs crates/core-save/tests/marks_layout.rs crates/ipc/src/marks.rs
git commit -m "refactor(core-save): the mark layout moves to the crate that owns the file"
```

---

### Task 5: `Profile`, and evaluation that takes one

**Files:**
- Modify: `crates/graph/src/evaluate.rs`
- Modify: `crates/graph/src/lib.rs` (export `Profile`)
- Test: `crates/graph/tests/evaluate.rs`

**Interfaces:**
- Consumes: Tasks 1–2.
- Produces:

```rust
pub trait Profile {
    fn done(&self) -> Option<&[bool]>;
    fn mark(&self, character: CharacterId, column: MarkColumn) -> Option<Option<MarkLevel>>;
    fn counter(&self, name: CounterName) -> Option<u32>;
}
```

  and `Graph::evaluate(&self, profile: &dyn Profile) -> Eval`,
  `Graph::missing_chain(&self, achievement: u32, profile: &dyn Profile) -> Vec<u32>`.

**The double `Option` is the whole point, not clumsiness.** One `None` cannot mean both
"cell not located" and "cell at zero": the first must make the node `Partial`, the second
must leave it `Computed` and unmet. So `None` is *cannot say*, `Some(None)` is *read, nothing
reached*, `Some(Some(level))` is *reached that level*. `counter` has the same shape for free:
`None` unread, `Some(0)` read and zero.

`MarkLevel` derives `PartialOrd, Ord` with `Base` declared before `Second`, so a cell at
`Second` satisfies a `Base` requirement by `reached >= required`.

- [ ] **Step 1: Write the failing tests**

In `crates/graph/tests/evaluate.rs`:

```rust
use graph::evaluate::NodeInfo;
use graph::model::Requirement;
use graph::rules::{CounterName, MarkColumn, MarkLevel};

/// A fake profile: the tests own the numbers, so nothing here depends on a sample.
/// `marks` absent = the cell is not located; `Some(None)` = located, nothing reached.
#[derive(Default)]
struct Fake {
    done: Vec<bool>,
    marks: std::collections::BTreeMap<(u32, MarkColumn), Option<MarkLevel>>,
    counters: std::collections::BTreeMap<CounterName, u32>,
}

impl graph::Profile for Fake {
    fn done(&self) -> Option<&[bool]> {
        Some(&self.done)
    }
    fn mark(
        &self,
        character: catalog::CharacterId,
        column: MarkColumn,
    ) -> Option<Option<MarkLevel>> {
        self.marks.get(&(character.0, column)).copied()
    }
    fn counter(&self, name: CounterName) -> Option<u32> {
        self.counters.get(&name).copied()
    }
}

impl Fake {
    /// The one node every test in this file builds, evaluated.
    fn node_1(&self, g: &graph::Graph) -> Option<NodeInfo> {
        g.evaluate(self).node(1).cloned()
    }
}

/// The one-node graph every test here uses: node 1, held only by `r`.
fn one_node(r: Requirement) -> graph::Graph {
    graph::Graph::from_requirements_for_tests(&[(1, vec![r])])
}

fn mother_of(id: u32) -> Requirement {
    Requirement::Mark {
        character: catalog::CharacterId(id),
        column: MarkColumn::Mother,
        level: MarkLevel::Base,
    }
}

#[test]
fn a_node_held_only_by_an_unmet_mark_is_available_now() {
    let g = one_node(mother_of(0));
    // The cell is located and read, and nothing has been reached in it.
    let p = Fake {
        done: vec![false, false],
        marks: [((0, MarkColumn::Mother), None)].into_iter().collect(),
        ..Default::default()
    };
    let Some(NodeInfo::Computed { available_now, blocked_by, .. }) = p.node_1(&g) else {
        panic!("a cell the profile can answer is not a reason to be Partial");
    };
    assert!(available_now, "nothing is locked: the content only has to be played");
    assert_eq!(blocked_by, 0, "blocked_by counts achievements, and a mark is not one");
}

#[test]
fn a_cell_the_layout_cannot_answer_keeps_the_node_partial() {
    let g = one_node(mother_of(99));
    // `marks` holds no entry: the cell is one of the 40 the series never located.
    let p = Fake { done: vec![false, false], ..Default::default() };
    assert!(
        matches!(p.node_1(&g), Some(NodeInfo::Partial { .. })),
        "an unlocated cell is 'we can't say', never 'not satisfied'"
    );
}

#[test]
fn the_second_level_satisfies_a_base_requirement() {
    let g = one_node(Requirement::Mark {
        character: catalog::CharacterId(0),
        column: MarkColumn::Greed,
        level: MarkLevel::Base,
    });
    let p = Fake {
        done: vec![false, false],
        marks: [((0, MarkColumn::Greed), Some(MarkLevel::Second))].into_iter().collect(),
        ..Default::default()
    };
    let Some(NodeInfo::Computed { available_now, .. }) = p.node_1(&g) else {
        panic!("a reached cell is answerable");
    };
    assert!(available_now, "reached at a higher level: the base requirement is met");
}

#[test]
fn a_tally_below_its_threshold_does_not_block_and_an_unread_one_is_partial() {
    let g = one_node(Requirement::Counter {
        name: CounterName::HushKills,
        at_least: 1,
    });
    let below = Fake {
        done: vec![false, false],
        counters: [(CounterName::HushKills, 0)].into_iter().collect(),
        ..Default::default()
    };
    assert!(
        matches!(below.node_1(&g), Some(NodeInfo::Computed { available_now: true, .. })),
        "read and zero: unmet, but nothing is locked"
    );

    let unread = Fake { done: vec![false, false], ..Default::default() };
    assert!(
        matches!(unread.node_1(&g), Some(NodeInfo::Partial { .. })),
        "section 2 unread must not read as 'the tally is zero'"
    );
}
```

`NodeInfo` already derives `Clone`; `Fake::node_1` relies on it.

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p graph --test evaluate`
Expected: FAIL to compile — `Profile` and `from_requirements_for_tests` do not exist.

- [ ] **Step 3: Add the test constructor**

In `crates/graph/src/build.rs`, beside the existing `from_edges_for_tests`:

```rust
    /// A graph whose nodes carry requirements directly, for tests that are about
    /// evaluation rather than about building. Prerequisites stay empty: these nodes are
    /// held by the profile, not by other achievements.
    pub fn from_requirements_for_tests(rows: &[(u32, Vec<Requirement>)]) -> Graph {
        Graph {
            nodes: rows
                .iter()
                .map(|(achievement, requirements)| Node {
                    achievement: *achievement,
                    requirements: requirements.clone(),
                    prerequisites: Vec::new(),
                    unknown: Vec::new(),
                })
                .collect(),
            diagnostics: Vec::new(),
        }
    }
```

- [ ] **Step 4: Add the trait and change `evaluate`**

In `crates/graph/src/evaluate.rs`:

```rust
/// What the graph is allowed to ask a save. Three questions, all already resolved by the
/// caller: this crate names a column, a level and a tally, and never an index.
pub trait Profile {
    fn done(&self) -> Option<&[bool]>;
    /// `None` — cannot say: the cell is not located, or section 2 was not read.
    /// `Some(None)` — located and read, nothing reached yet.
    /// `Some(Some(level))` — the highest level reached.
    fn mark(&self, character: CharacterId, column: MarkColumn) -> Option<Option<MarkLevel>>;
    /// `None` when section 2 was not read. Not `Some(0)`: an unread tally is not a zero one.
    fn counter(&self, name: CounterName) -> Option<u32>;
}
```

Change `pub fn evaluate(&self, flags: Option<&[bool]>)` to
`pub fn evaluate(&self, profile: &dyn Profile)`, take `let Some(flags) = profile.done()`
at the top exactly as it takes `flags` today, and add — next to the `unproven` count — a
count of the profile requirements this profile cannot answer:

```rust
            // A requirement the profile cannot answer joins the uninterpreted ones: an
            // unread section and an unlocated cell are both "we can't say", and neither is
            // allowed to read as satisfied.
            let unanswerable = n
                .requirements
                .iter()
                .filter(|r| match r {
                    Requirement::Mark { character, column, .. } => {
                        profile.mark(*character, *column).is_none()
                    }
                    Requirement::Counter { name, .. } => profile.counter(*name).is_none(),
                    Requirement::Character { .. }
                    | Requirement::Boss { .. }
                    | Requirement::Challenge { .. }
                    | Requirement::Item { .. }
                    | Requirement::Gate { .. }
                    | Requirement::Unknown { .. }
                    | Requirement::None => false,
                })
                .count() as u32;
```

and use `(unproven + unanswerable, transitive_known)` as the match scrutinee where
`(unproven, transitive_known)` is used today.

`available_now` and `blocked_by` are **not** touched: a mark or a tally is not an
achievement, so it does not enter `blocked_by`, and a node nothing else holds is available
now. That is the decision recorded in spec §4.6.

- [ ] **Step 5: Change `missing_chain` the same way**

`pub fn missing_chain(&self, achievement: u32, profile: &dyn Profile) -> Vec<u32>`, reading
`profile.done()` where it reads `flags` today. The two entry points must not disagree about
what a profile is.

- [ ] **Step 6: Export and fix every caller**

`pub use evaluate::Profile;` in `crates/graph/src/lib.rs`. Then
`cargo check --workspace` and fix each call site — `crates/ipc/src/graph.rs`,
`crates/ipc/src/queue.rs`, the graph tests' `support/mod.rs`. Task 7 gives `ipc` its real
implementation; until then a small adapter carrying only `done()` and `None` for the other
two keeps the workspace compiling.

- [ ] **Step 7: Run the tests**

Run: `cargo test -p graph`
Expected: PASS, including the five new ones.

- [ ] **Step 8: Commit**

```bash
git add crates/graph/src/evaluate.rs crates/graph/src/build.rs crates/graph/src/lib.rs crates/graph/tests/evaluate.rs
git commit -m "feat(graph): evaluate against a profile, not against a slice of flags"
```

---

### Task 6: `ipc` answers the three questions

**Files:**
- Create: `crates/ipc/src/profile.rs`
- Modify: `crates/ipc/src/lib.rs`, `crates/ipc/src/graph.rs` (the `evaluate` call sites)
- Test: `crates/ipc/tests/profile.rs`

**Interfaces:**
- Consumes: Task 4's `core_save::marks`, Task 5's `graph::Profile`.
- Produces: `ipc::profile::SaveProfile::new(flags: Option<&[bool]>, counters: Option<&[u32]>, catalog: Option<&Catalog>) -> SaveProfile`, implementing `graph::Profile`.

- [ ] **Step 1: Write the failing tests**

In `crates/ipc/tests/profile.rs`:

```rust
/// Bit 1 set means the cell reached its second level; bit 0 alone is the base. Bit 2 is not
/// a level and must not be read as one.
#[test]
fn a_cell_reports_the_highest_level_its_bits_show() {
    let mut counters = vec![0u32; 600];
    counters[142] = 3; // Greed, Keeper: base + second
    counters[131] = 1; // Greed, Magdalene: base only
    counters[133] = 5; // Greed, Judas: base + bit 2, which is not a level
    let p = ipc::profile::SaveProfile::new(Some(&[]), Some(&counters), None);
    assert_eq!(p.level_at(12, MarkColumn::Greed), Some(MarkLevel::Second));
    assert_eq!(p.level_at(1, MarkColumn::Greed), Some(MarkLevel::Base));
    assert_eq!(p.level_at(3, MarkColumn::Greed), Some(MarkLevel::Base));
}

#[test]
fn a_short_or_absent_section_two_answers_none() {
    let p = ipc::profile::SaveProfile::new(Some(&[]), None, None);
    assert_eq!(p.counter(CounterName::HushKills), None);
    let short = ipc::profile::SaveProfile::new(Some(&[]), Some(&[0u32; 10]), None);
    assert_eq!(short.counter(CounterName::MotherKills), None, "index 491 is past the end");
}

#[test]
fn a_tally_reads_its_value() {
    let mut counters = vec![0u32; 600];
    counters[158] = 17;
    let p = ipc::profile::SaveProfile::new(Some(&[]), Some(&counters), None);
    assert_eq!(p.counter(CounterName::HushKills), Some(17));
}
```

`level_at(row, column)` is the row-indexed helper the test uses; `mark(character, column)`
is the trait method, which maps `CharacterId` to a row through the catalog and then calls it.

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p ipc --test profile`
Expected: FAIL to compile — `ipc::profile` does not exist.

- [ ] **Step 3: Implement it**

`crates/ipc/src/profile.rs`:

```rust
//! The three questions `graph::Profile` asks, answered from the save. This is the only
//! place that knows a column is a block base and a tally is an index: `graph` names them.

pub struct SaveProfile<'a> {
    flags: Option<&'a [bool]>,
    counters: Option<&'a [u32]>,
    rows: std::collections::BTreeMap<u32, usize>,
}

impl<'a> SaveProfile<'a> {
    pub fn new(
        flags: Option<&'a [bool]>,
        counters: Option<&'a [u32]>,
        catalog: Option<&catalog::Catalog>,
    ) -> SaveProfile<'a> {
        // CharacterId -> matrix row, built once: resolution happens per requirement.
        let mut rows = std::collections::BTreeMap::new();
        if let Some(c) = catalog {
            for row in 0..crate::marks::CHARACTERS.len() {
                if let Some(ch) = crate::marks::character_for(row, c) {
                    rows.insert(ch.id.0, row);
                }
            }
        }
        SaveProfile { flags, counters, rows }
    }

    /// `None` — the cell is not located, past the end of what was read, or holds a value
    /// that is not a mask. `Some(None)` — read, nothing reached. `Some(Some(level))` — the
    /// highest level its bits show.
    ///
    /// Bit 2 is not a level: it is "won online", measured 2026-09-12. It is masked away
    /// before a level is read, so an online clear never reads as a second level.
    pub fn level_at(&self, row: usize, column: MarkColumn) -> Option<Option<MarkLevel>> {
        let i = core_save::marks::cell_index(row, column.into())?;
        let v = *self.counters?.get(i)?;
        if v > 7 {
            return None; // not a mask: the index points somewhere else
        }
        Some(match v & 0b11 {
            0 => None,
            0b01 => Some(MarkLevel::Base),
            _ => Some(MarkLevel::Second),
        })
    }
}

impl graph::Profile for SaveProfile<'_> {
    fn done(&self) -> Option<&[bool]> {
        self.flags
    }
    fn mark(
        &self,
        character: catalog::CharacterId,
        column: MarkColumn,
    ) -> Option<Option<MarkLevel>> {
        self.level_at(*self.rows.get(&character.0)?, column)
    }
    fn counter(&self, name: CounterName) -> Option<u32> {
        let i = core_save::marks::counter_index_of(wire_name(name))?;
        self.counters?.get(i).copied()
    }
}

/// The wire form of the name, which is also the key `core-save`'s table uses. Written out
/// rather than derived from `serde` so that a rename of either side breaks the build.
fn wire_name(n: CounterName) -> &'static str {
    match n {
        CounterName::HushKills => "hushKills",
        CounterName::DeliriumKills => "deliriumKills",
        CounterName::MotherKills => "motherKills",
        CounterName::BeastKills => "beastKills",
    }
}
```

`column.into()` converts `graph::rules::MarkColumn` to `core_save::marks::Column`. Write that
`From` by hand with a full `match` and no `_` arm: the two enums are the same twelve columns
in the same order, and the day one of them gains a thirteenth the build must break.

- [ ] **Step 4: Run the tests**

Run: `cargo test -p ipc --test profile`
Expected: PASS, 3 tests.

- [ ] **Step 5: Wire it into the real call sites**

In `crates/ipc/src/graph.rs`, every `g.evaluate(flags)` becomes
`g.evaluate(&SaveProfile::new(flags, counters, catalog))`, where `counters` is the caller's
`save.u32s(Kind::Counters)`. Follow the parameter up through `unlock_view`, `next_steps`
and `plan_view` — they already take the save's flags, and now take its counters too.

- [ ] **Step 6: Run the workspace**

Run: `cargo test --workspace`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/ipc/src/profile.rs crates/ipc/src/lib.rs crates/ipc/src/graph.rs crates/ipc/tests/profile.rs
git commit -m "feat(ipc): answer the graph's profile questions from the save"
```

---

### Task 7: The two new view-models

**Files:**
- Modify: `crates/ipc/src/graph.rs` (`RequirementView`, `missing_view`)
- Test: `crates/ipc/tests/summary_shape.rs`

**Interfaces:**
- Consumes: Tasks 2, 6.
- Produces: `RequirementView::Mark` and `RequirementView::Counter` on the wire.

- [ ] **Step 1: Write the failing test**

In `crates/ipc/tests/summary_shape.rs`:

```rust
/// The wire shape, pinned. A missing `rename_all_fields` reads as `undefined` in
/// TypeScript with no error at all, which is the failure this test exists for.
#[test]
fn the_new_requirement_views_serialise_in_camel_case() {
    let m = ipc::RequirementView::Mark {
        character: 1,
        character_name: "Magdalene".to_string(),
        column: ipc::MarkColumnView::Mother,
        level: ipc::MarkLevelView::Base,
    };
    let j = serde_json::to_value(&m).expect("serialises");
    assert_eq!(j["kind"], "mark");
    assert_eq!(j["characterName"], "Magdalene");
    assert_eq!(j["column"], "mother", "a fieldless enum is a bare string, never tagged");
    assert_eq!(j["level"], "base");

    let c = ipc::RequirementView::Counter {
        label: "Hush".to_string(),
        current: 0,
        at_least: 1,
    };
    let j = serde_json::to_value(&c).expect("serialises");
    assert_eq!(j["kind"], "counter");
    assert_eq!(j["atLeast"], 1);
    assert_eq!(j["current"], 0);
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p ipc --test summary_shape`
Expected: FAIL to compile — the variants do not exist.

- [ ] **Step 3: Add the variants**

In `crates/ipc/src/graph.rs`, inside `enum RequirementView`:

```rust
    /// One cell of the completion matrix: go and beat `column` with this character.
    /// No progress field: for one cell the state is binary, and an invented percentage
    /// would be a number nobody measured.
    Mark {
        character: u32,
        character_name: String,
        column: MarkColumnView,
        level: MarkLevelView,
    },
    /// A tally and its threshold, with where the profile stands.
    Counter {
        label: String,
        current: u32,
        at_least: u32,
    },
```

and the two fieldless view enums, **untagged, bare camelCase strings**:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MarkColumnView {
    MomsHeart, Isaac, Satan, BossRush, BlueBaby, TheLamb,
    MegaSatan, Greed, Hush, Delirium, Mother, TheBeast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MarkLevelView {
    Base,
    Second,
}
```

- [ ] **Step 4: Build them in `missing_view`**

In `missing_view`, add the two arms. A satisfied requirement is left out, exactly as the
existing arms leave out a done prerequisite:

```rust
            graph::model::Requirement::Mark { character, column, level } => {
                let Some(ch) = c.character(*character) else { continue };
                // Reached already: not missing. Cannot say: it is not this list's job to
                // report that — the node is Partial and says so there.
                match profile.mark(*character, *column) {
                    Some(Some(reached)) if reached >= *level => {}
                    None => {}
                    _ => out.push(RequirementView::Mark {
                        character: character.0,
                        character_name: c.text(&ch.name, en).to_string(),
                        column: column_view(*column),
                        level: level_view(*level),
                    }),
                }
            }
            graph::model::Requirement::Counter { name, at_least } => {
                let Some(current) = profile.counter(*name) else { continue };
                if current < *at_least {
                    out.push(RequirementView::Counter {
                        label: counter_label(*name).to_string(),
                        current,
                        at_least: *at_least,
                    });
                }
            }
```

`missing_view` gains a `profile: &SaveProfile` parameter. `MarkLevel` needs
`#[derive(PartialOrd, Ord)]` with `Base` declared before `Second` for `reached >= *level`
to mean what it reads as; add it in Task 1's enum if this step is reached first.

`counter_label` maps the four names to the boss's English name (`"Hush"`, `"Delirium"`,
`"Mother"`, `"The Beast"`) — a `match`, no `_` arm.

- [ ] **Step 5: Run the tests**

Run: `cargo test -p ipc`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/ipc/src/graph.rs crates/ipc/tests/summary_shape.rs
git commit -m "feat(ipc): a mark and a tally cross the boundary as requirements"
```

---

### Task 8: The TypeScript mirror

**Files:**
- Modify: `ui/src/lib/ipc/types.ts` (`RequirementView`, around line 244)
- Modify: the Unlock/Next-steps fixtures that build a `RequirementView`
- Test: `pnpm ui:test`

**Interfaces:**
- Consumes: Task 7's wire shape.
- Produces: the mirrored union.

- [ ] **Step 1: Add the types**

In `ui/src/lib/ipc/types.ts`, following the repo's rule that a fieldless Rust enum is a
**union of values** declared through a `const` object, never a bare string union:

```ts
export const MarkColumnView = {
  momsHeart: 'momsHeart',
  isaac: 'isaac',
  satan: 'satan',
  bossRush: 'bossRush',
  blueBaby: 'blueBaby',
  theLamb: 'theLamb',
  megaSatan: 'megaSatan',
  greed: 'greed',
  hush: 'hush',
  delirium: 'delirium',
  mother: 'mother',
  theBeast: 'theBeast',
} as const
export type MarkColumnView = (typeof MarkColumnView)[keyof typeof MarkColumnView]

export const MarkLevelView = { base: 'base', second: 'second' } as const
export type MarkLevelView = (typeof MarkLevelView)[keyof typeof MarkLevelView]
```

and two members on the `RequirementView` union:

```ts
  | { kind: 'mark'; character: number; characterName: string; column: MarkColumnView; level: MarkLevelView }
  | { kind: 'counter'; label: string; current: number; atLeast: number }
```

- [ ] **Step 2: Find every exhaustive switch over `RequirementView`**

Run: `rg "kind === 'gate'|RequirementView" ui/src --type ts --type vue`
Add the two cases wherever the union is matched. A `switch` that silently falls into no
branch is exactly what the repo's "no tagged unit enum" rule exists to prevent.

- [ ] **Step 3: Run the checks**

Run: `pnpm typecheck && pnpm ui:test && pnpm scan`
Expected: PASS, 0 violations.

- [ ] **Step 4: Commit**

```bash
git add ui/src/lib/ipc/types.ts ui/src
git commit -m "feat(ui): mirror the mark and tally requirements"
```

---

### Task 9: The measurement becomes a test

**Files:**
- Create: `crates/graph/tests/progress_real.rs`
- Test: itself

**Interfaces:**
- Consumes: everything above.
- Produces: the property that guards spec §2.2 and §2.3.

- [ ] **Step 1: Write the test**

```rust
//! Properties over the historical series, not pinned values: they hold across eras, and
//! they are what would catch a column moving under us. Spec §2.2 and §2.3.

mod support;

use core_save::{Kind, Save};
use graph::target_key;

/// Spec §2.2. On every day of the series where an achievement whose references are
/// (`entity:Ultra Greedier`, character X) flipped, X's Greed cell gained bit 1.
#[test]
fn winning_greedier_sets_the_second_bit_of_that_character_s_greed_cell() {
    let series = test_support::dated_series("rep+persistentgamedata1.dat");
    if series.len() < 2 {
        return; // declared by dated_series
    }
    let Some((catalog, _rs)) = support::real_catalog() else { return };
    let rules = graph::rules::embedded().expect("rules");

    let mut checked = 0u32;
    let mut prev: Option<(Vec<bool>, Vec<u32>)> = None;
    for path in &series {
        let Ok(s) = Save::open(path) else { continue };
        let (Some(flags), Some(counters)) = (s.flags(Kind::Achievements), s.u32s(Kind::Counters))
        else {
            continue;
        };
        if let Some((pflags, pcounters)) = &prev {
            for id in 0..flags.len().min(pflags.len()) {
                if !(flags[id] && !pflags[id]) {
                    continue;
                }
                let keys: Vec<String> = rules
                    .refs(id as u32)
                    .iter()
                    .map(|r| target_key(&r.target, rules.alias(&r.label)))
                    .collect();
                if !keys.iter().any(|k| k == "entity:Ultra Greedier") {
                    continue;
                }
                let Some(name) = keys.iter().find_map(|k| k.strip_prefix("character:")) else {
                    continue;
                };
                let index = graph::resolve::NameIndex::new(&catalog);
                let Some(ch) = index.character(name) else { continue };
                let Some(row) = ipc_row_of(&catalog, ch) else { continue };
                let Some(cell) = core_save::marks::cell_index(row, core_save::marks::Column::Greed)
                else {
                    continue;
                };
                let (before, after) = (
                    pcounters.get(cell).copied().unwrap_or(0),
                    counters.get(cell).copied().unwrap_or(0),
                );
                assert_eq!(
                    (before & 0b10, after & 0b10),
                    (0, 0b10),
                    "achievement {id} is Ultra Greedier as {name} (row {row}, cell {cell}): \
                     bit 1 had to turn on that day, saw {before} -> {after}"
                );
                checked += 1;
            }
        }
        prev = Some((flags, counters));
    }
    eprintln!("greedier days checked: {checked}");
    assert!(
        checked >= 3,
        "the series held {checked} Ultra Greedier days; the measurement of 2026-09-12 had 3, \
         so a smaller number means the samples shrank, not that the property got weaker"
    );
}
```

`ipc_row_of` is a local helper mapping a `CharacterId` to its matrix row via the catalog;
`graph` must not gain a dependency for it, so write it in this test file by scanning
`ipc::marks::CHARACTERS` — or, if `graph`'s dev-dependencies do not include `ipc`, add
`ipc` to `[dev-dependencies]` of `crates/graph/Cargo.toml`. A dev-dependency is not a
dependency: the global constraint is about what `graph` ships, not what its tests read.

- [ ] **Step 2: Run it**

Run: `cargo test -p graph --test progress_real -- --nocapture`
Expected: PASS with `greedier days checked: 3`, or a `skip:` line and a pass on a machine
without `samples/`.

- [ ] **Step 3: Add the §2.3 reproduction**

In the same file, the same walk asserting that a (`entity:Mother`, character X) day moves
`cell_index(row(X), Column::Mother)` — which is base 423 for the originals — and that a
(`entity:The Beast`, X) day moves base 457. Assert the *cell that moved*, not the base:
the base is what the layout claims and the moved cell is what the file says.

- [ ] **Step 4: Commit**

```bash
git add crates/graph/tests/progress_real.rs crates/graph/Cargo.toml
git commit -m "test(graph): the series guards the mark measurement"
```

---

### Task 10: Close the books

**Files:**
- Modify: `docs/STATUS.md`
- Modify: `reference/isaac_counters.py` (the comment block on the Greed column)
- Modify: `CLAUDE.md` (the "Counters and marks" paragraph)

- [ ] **Step 1: Run the full check**

Run: `pnpm check`
Expected: every command green. Read the skip count at the end — a green suite that skipped
the real-data tests has verified none of this task's evidence.

- [ ] **Step 2: Record the measurement where the project keeps measurements**

In `CLAUDE.md`, the "Counters and marks" paragraph says the two **levels** are the unmeasured
half. One column is now measured: in **Greed**, the level on bit 1 is **Ultra Greedier**,
pinned on three days by three characters (spec §2.2), and guarded by
`winning_greedier_sets_the_second_bit_of_that_character_s_greed_cell`. The other eleven
columns stay unmeasured — say so in the same sentence, or the next reader will take the
generalisation for granted.

In `reference/isaac_counters.py`, add the same note beside the `("Greed", 130)` block.

- [ ] **Step 3: Update `docs/STATUS.md`**

A checkbox is landed work, so tick these only once they are committed: the graph answers 138
of the 178 references it used to decline; the 40 that remain are Mother and The Beast for The
Forgotten and the 19, and spec §2.4 **verified** that the series contains no such completion
— so the blocker keeps its entry and gains the evidence that it is real.

- [ ] **Step 4: Hand the contract on**

`RequirementView` gained two variants and Unlock's "unlockable now" count rises: spec §4.6.
This is a change to material the design system is built on, so it is handed on, not merely
committed. Note it in `docs/STATUS.md` where the IPC contract's changes are recorded.

- [ ] **Step 5: Commit**

```bash
git add docs/STATUS.md reference/isaac_counters.py CLAUDE.md
git commit -m "docs: the graph answers 138 references it used to decline"
```
