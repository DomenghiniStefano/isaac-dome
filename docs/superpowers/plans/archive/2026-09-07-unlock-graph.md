# Unlock graph (M2) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fill `GraphInfo` with a real unlock graph — what blocks a node, what it opens, how far it is — derived from the wiki's typed requirements and the game's own unlock links.

**Architecture:** A new pure crate `crates/graph`. A generator turns `dataset/wiki.json` into a committed `requirements.json` (wiki targets only, no ids of ours); a hand-written `corrections.json` carries aliases and three verdicts; both are embedded at build time. At runtime `Graph::build(&Catalog, &Rules)` resolves those targets onto the user's catalog, `evaluate(flags)` walks the edges against the save, and `ipc` maps the result onto the contract.

**Tech Stack:** Rust 2021, `serde` / `serde_json`, existing crates `catalog`, `wiki`, `core-save`, `ipc`, `test-support`. No new third-party dependency.

**Spec:** `docs/superpowers/specs/2026-09-07-unlock-graph-design.md` — read it first; this plan argues from it.

## Global Constraints

- **Read-only on saves.** Nothing here opens a `.dat`; flags arrive as `Option<&[bool]>`.
- **No network, ever.** The generator reads `dataset/wiki.json` from disk. Nothing fetches.
- **No `panic!` / `unwrap()`** outside tests on data read from disk. The rules file counts as data read from disk even when embedded.
- **Every struct crossing the IPC** carries `#[serde(rename_all = "camelCase")]`; every enum with struct variants also carries `rename_all_fields = "camelCase"`.
- **Fieldless enums on the IPC are bare camelCase strings**, never tagged objects.
- **Exhaustiveness is mandatory:** no `_ =>` arm on a closed enum.
- **Comments, doc-comments and `assert!` messages in English** (B7, 2026-09-07).
- **Commits:** Conventional Commits, `type(scope): subject`, scope `graph` / `ipc` / `app`. **Never** a `Co-Authored-By` trailer or any reference to Claude.
- **Before declaring done:** `pnpm check` (which is `scripts/check`).
- Real-data tests go through `test-support` and **skip with a note** when `samples/` or `samples/packed` is missing. Run them with `cargo test --workspace -- --nocapture` so the skips are visible.

---

### Task 1: Crate scaffold and the rules model

**Files:**
- Create: `crates/graph/Cargo.toml`
- Create: `crates/graph/src/lib.rs`
- Create: `crates/graph/src/rules.rs`
- Test: `crates/graph/tests/rules.rs`
- Modify: `Cargo.toml` (workspace members)

**Interfaces:**
- Consumes: nothing.
- Produces: `graph::rules::{Requirements, Corrections, Rules, RefRow, TargetRow, Verdict, RulesError, SCHEMA_VERSION}`, `Rules::build(Requirements, Corrections) -> Result<Rules, RulesError>`, `Rules::alias(&str) -> &str`, `Rules::verdict(&str) -> Option<&Verdict>`, and the free function `target_key(&wiki::Target, label: &str) -> String`.

- [ ] **Step 1: Write the failing test**

`crates/graph/tests/rules.rs`:

```rust
use graph::rules::{Corrections, Requirements, Rules, Verdict, SCHEMA_VERSION};

const REQS: &str = r#"{
  "schemaVersion": 1,
  "generatedFrom": { "snapshotAt": "2026-09-04T17:33:31Z", "maxRevid": 269057 },
  "achievements": {
    "1": { "refs": [{ "target": { "kind": "entity", "id": 5, "variant": 10, "subtype": 1 },
                      "label": "Red Heart" }] }
  },
  "targets": [{ "key": "entity:Red Heart", "label": "Red Heart", "uses": 3 }]
}"#;

const CORR: &str = r#"{
  "schemaVersion": 1,
  "aliases": { "Jacob and Esau": "Jacob & Esau" },
  "verdicts": { "entity:Red Heart": { "notAPrerequisite": true } }
}"#;

#[test]
fn reads_the_two_files_and_joins_them() {
    let r: Requirements = serde_json::from_str(REQS).expect("requirements parse");
    let c: Corrections = serde_json::from_str(CORR).expect("corrections parse");
    assert_eq!(r.schema_version, SCHEMA_VERSION);
    let rules = Rules::build(r, c).expect("rules build");
    assert_eq!(rules.alias("Jacob and Esau"), "Jacob & Esau");
    assert_eq!(rules.alias("Mom"), "Mom", "an unaliased label passes through unchanged");
    assert_eq!(rules.verdict("entity:Red Heart"), Some(&Verdict::NotAPrerequisite));
    assert_eq!(rules.verdict("stage:Nowhere"), None, "no verdict is not a verdict of 'none'");
}

#[test]
fn a_file_from_another_schema_is_refused_whole() {
    let bumped = REQS.replace("\"schemaVersion\": 1", "\"schemaVersion\": 2");
    let r: Requirements = serde_json::from_str(&bumped).expect("parses");
    let c: Corrections = serde_json::from_str(CORR).expect("parses");
    let err = Rules::build(r, c).expect_err("a newer schema must not be read half-broken");
    assert!(
        format!("{err:?}").contains("SchemaMismatch"),
        "expected SchemaMismatch, got {err:?}"
    );
}

#[test]
fn every_target_row_is_addressable_by_its_key() {
    let r: Requirements = serde_json::from_str(REQS).expect("parses");
    let c: Corrections = serde_json::from_str(CORR).expect("parses");
    let rules = Rules::build(r, c).expect("rules build");
    assert_eq!(rules.targets().len(), 1);
    assert_eq!(rules.targets()[0].key, "entity:Red Heart");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p graph --test rules`
Expected: FAIL — the package `graph` does not exist.

- [ ] **Step 3: Write minimal implementation**

`crates/graph/Cargo.toml`:

```toml
[package]
name = "graph"
version = "0.1.0"
edition = "2021"
description = "Unlock graph: typed requirements, curated verdicts, evaluation against a profile"

[dependencies]
catalog = { path = "../catalog" }
wiki = { path = "../wiki" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[dev-dependencies]
test-support = { path = "../test-support" }
unpack = { path = "../unpack" }
core-save = { path = "../core-save" }
```

Add `"crates/graph"` to the workspace `members` in the root `Cargo.toml`.

`crates/graph/src/rules.rs`:

```rust
//! The two rules files and their join. `requirements.json` is generated from the wiki
//! snapshot and holds no id of ours; `corrections.json` is written by hand. Nothing here
//! knows about the user's catalog: resolution happens later, in `graph::build`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use wiki::Target;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Requirements {
    pub schema_version: u32,
    pub generated_from: GeneratedFrom,
    pub achievements: BTreeMap<u32, AchievementRefs>,
    /// Every target that doesn't reduce to an achievement on its own: the form the
    /// curation fills in, not a list anyone has to invent.
    pub targets: Vec<TargetRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedFrom {
    pub snapshot_at: String,
    pub max_revid: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AchievementRefs {
    pub refs: Vec<RefRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefRow {
    pub target: Target,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetRow {
    pub key: String,
    pub label: String,
    pub uses: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Corrections {
    pub schema_version: u32,
    #[serde(default)]
    pub aliases: BTreeMap<String, String>,
    #[serde(default)]
    pub verdicts: BTreeMap<String, Verdict>,
}

/// Exactly three verdicts, and no fourth. A target with no verdict is not "no
/// prerequisite": it stays unknown, and the node that carries it drops to `Partial`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Verdict {
    /// Fought on night one: Satan, Mom, Isaac.
    AlwaysAvailable(bool),
    /// Sits behind an achievement of the graph.
    Behind { achievement: u32 },
    /// Not a prerequisite at all: a pickup that appears in the sentence.
    NotAPrerequisite(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RulesError {
    SchemaMismatch { found: u32, expected: u32 },
    Malformed { reason: String },
}

pub struct Rules {
    requirements: Requirements,
    corrections: Corrections,
}

impl Rules {
    pub fn build(r: Requirements, c: Corrections) -> Result<Rules, RulesError> {
        for found in [r.schema_version, c.schema_version] {
            if found != SCHEMA_VERSION {
                return Err(RulesError::SchemaMismatch {
                    found,
                    expected: SCHEMA_VERSION,
                });
            }
        }
        Ok(Rules {
            requirements: r,
            corrections: c,
        })
    }

    pub fn alias<'a>(&'a self, label: &'a str) -> &'a str {
        self.corrections
            .aliases
            .get(label)
            .map(String::as_str)
            .unwrap_or(label)
    }

    pub fn verdict(&self, key: &str) -> Option<&Verdict> {
        self.corrections.verdicts.get(key)
    }

    pub fn targets(&self) -> &[TargetRow] {
        &self.requirements.targets
    }

    pub fn refs(&self, achievement: u32) -> &[RefRow] {
        self.requirements
            .achievements
            .get(&achievement)
            .map(|a| a.refs.as_slice())
            .unwrap_or(&[])
    }

    pub fn generated_from(&self) -> &GeneratedFrom {
        &self.requirements.generated_from
    }
}

/// The key a target is addressed by in `corrections.json`: `kind:label`. The label is the
/// bridge, not the id — the wiki's entity ids and the catalog's boss ids are two different
/// numbering spaces (Gish is entity 43 and boss 19).
pub fn target_key(t: &Target, label: &str) -> String {
    let kind = match t {
        Target::Item { .. } => "item",
        Target::Trinket { .. } => "trinket",
        Target::Character { .. } => "character",
        Target::Achievement { .. } => "achievement",
        Target::Challenge { .. } => "challenge",
        Target::Entity { .. } => "entity",
        Target::Transformation { .. } => "transformation",
        Target::Stage { .. } => "stage",
        Target::Room { .. } => "room",
        Target::Pickup { .. } => "pickup",
    };
    format!("{kind}:{label}")
}
```

`crates/graph/src/lib.rs`:

```rust
//! The unlock graph. Pure: no I/O, no Tauri, no network.
//!
//! Where the data comes from is the design's core decision: **what is needed** comes from
//! the wiki's typed refs, **who unlocks what** comes from the game's own files through
//! `catalog`. The graph never invents an edge from the wiki; it reads it from the game.

pub mod rules;

pub use rules::{target_key, Corrections, Requirements, Rules, RulesError, SCHEMA_VERSION};
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p graph --test rules`
Expected: PASS, 3 tests.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml crates/graph
git commit -m "feat(graph): rules model for requirements and hand corrections"
```

---

### Task 2: Generate the requirements from the wiki dataset

**Files:**
- Create: `crates/graph/src/generate.rs`
- Modify: `crates/graph/src/lib.rs`
- Test: `crates/graph/tests/generate.rs`

**Interfaces:**
- Consumes: `graph::rules::{Requirements, RefRow, TargetRow, target_key}` from Task 1; `wiki::{Dataset, Entry, Infobox, Inline, Target}`.
- Produces: `graph::generate::generate(&wiki::Dataset) -> Requirements`, and `graph::generate::collect_refs(&[wiki::Inline], &mut Vec<RefRow>)`.

The rule for what lands in `targets`: a ref whose target is **not** `Item`, `Trinket`, `Character`, `Achievement` or `Challenge` — those five reduce to an achievement through the catalog's own `unlocked_by` links, everything else needs a human verdict. `Inline::Concept` has no `Target` at all and is recorded as `Target::Pickup { name: page }`, because a concept page is exactly the "named thing with no id" case that variant already models.

- [ ] **Step 1: Write the failing test**

`crates/graph/tests/generate.rs`:

```rust
use graph::generate::generate;
use std::collections::BTreeMap;
use wiki::{Dataset, Entry, Infobox, Inline, Style, Target};

fn achievement(id: u32, requirements: Vec<Inline>) -> (u32, Entry) {
    (
        id,
        Entry {
            title: format!("A{id}"),
            revid: 1,
            infobox: Infobox::Achievement {
                description: String::new(),
                requirements,
                unlocks: None,
            },
            sections: Vec::new(),
        },
    )
}

fn dataset(entries: Vec<(u32, Entry)>) -> Dataset {
    let mut d = Dataset::empty_for_tests();
    d.achievements = entries.into_iter().collect::<BTreeMap<_, _>>();
    d
}

fn text(t: &str) -> Inline {
    Inline::Text {
        text: t.into(),
        style: Style::Plain,
    }
}

#[test]
fn a_typed_ref_becomes_a_requirement_row() {
    let d = dataset(vec![achievement(
        7,
        vec![
            text("Defeat "),
            Inline::Ref {
                target: Target::Entity {
                    id: 84,
                    variant: 0,
                    subtype: 0,
                },
                label: "Satan".into(),
            },
        ],
    )]);
    let r = generate(&d);
    let refs = &r.achievements.get(&7).expect("achievement 7 is present").refs;
    assert_eq!(refs.len(), 1, "the plain text around the ref is not a requirement");
    assert_eq!(refs[0].label, "Satan");
}

#[test]
fn refs_nested_in_an_edition_block_are_not_lost() {
    let d = dataset(vec![achievement(
        8,
        vec![Inline::Edition {
            only: vec![wiki::Dlc::Repentance],
            inline: vec![Inline::Ref {
                target: Target::Character { id: 19 },
                label: "Jacob and Esau".into(),
            }],
        }],
    )]);
    let r = generate(&d);
    assert_eq!(
        r.achievements.get(&8).expect("present").refs.len(),
        1,
        "an Edition block wraps inline content: its refs still count"
    );
}

#[test]
fn only_unreducible_targets_reach_the_inventory() {
    let d = dataset(vec![
        achievement(
            1,
            vec![Inline::Ref {
                target: Target::Character { id: 2 },
                label: "Cain".into(),
            }],
        ),
        achievement(
            2,
            vec![Inline::Ref {
                target: Target::Stage {
                    name: "The Void".into(),
                },
                label: "The Void".into(),
            }],
        ),
        achievement(
            3,
            vec![Inline::Concept {
                page: "Hard mode".into(),
                label: "Hard mode".into(),
            }],
        ),
    ]);
    let r = generate(&d);
    let keys: Vec<&str> = r.targets.iter().map(|t| t.key.as_str()).collect();
    assert!(
        !keys.contains(&"character:Cain"),
        "a character reduces through the catalog: it needs no verdict"
    );
    assert!(keys.contains(&"stage:The Void"), "got {keys:?}");
    assert!(keys.contains(&"pickup:Hard mode"), "a concept has no id: {keys:?}");
}

#[test]
fn the_inventory_counts_uses_and_is_deterministic() {
    let r#ref = |label: &str| Inline::Ref {
        target: Target::Stage {
            name: label.into(),
        },
        label: label.into(),
    };
    let d = dataset(vec![
        achievement(1, vec![r#ref("Basement")]),
        achievement(2, vec![r#ref("Basement"), r#ref("The Void")]),
    ]);
    let r = generate(&d);
    let basement = r
        .targets
        .iter()
        .find(|t| t.key == "stage:Basement")
        .expect("present");
    assert_eq!(basement.uses, 2);
    let keys: Vec<&str> = r.targets.iter().map(|t| t.key.as_str()).collect();
    let mut sorted = keys.clone();
    sorted.sort_unstable();
    assert_eq!(keys, sorted, "the inventory is sorted: the file must not churn");
}
```

`Dataset::empty()` is `pub(crate)` today. Add next to it, in `crates/wiki/src/dataset.rs`:

```rust
    /// The same empty dataset, for tests in crates downstream: they need a `Dataset`
    /// to build one field of, and constructing `meta` by hand in every test would
    /// copy this block around.
    pub fn empty_for_tests() -> Dataset {
        Dataset::empty()
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p graph --test generate`
Expected: FAIL — `graph::generate` does not exist.

- [ ] **Step 3: Write minimal implementation**

`crates/graph/src/generate.rs`:

```rust
//! One pass from `dataset/wiki.json` to `requirements.json`. Runs offline, once per
//! snapshot, never from the app.

use std::collections::BTreeMap;

use wiki::{Dataset, Infobox, Inline, Target};

use crate::rules::{
    target_key, AchievementRefs, GeneratedFrom, RefRow, Requirements, TargetRow, SCHEMA_VERSION,
};

/// Walks the inline tree and keeps what points at something. `Inline::Text` carries no
/// target; `Inline::Edition` wraps content and is recursed into, or the refs inside a
/// DLC-only sentence would vanish silently.
pub fn collect_refs(inline: &[Inline], out: &mut Vec<RefRow>) {
    for i in inline {
        match i {
            Inline::Text { .. } => {}
            Inline::Ref { target, label } => out.push(RefRow {
                target: target.clone(),
                label: label.clone(),
            }),
            Inline::Concept { page, label } => out.push(RefRow {
                target: Target::Pickup { name: page.clone() },
                label: label.clone(),
            }),
            Inline::Edition { only: _, inline } => collect_refs(inline, out),
        }
    }
}

/// True when the target reduces to an achievement through the catalog's own
/// `unlocked_by` links, and therefore needs no hand verdict.
fn reduces_on_its_own(t: &Target) -> bool {
    match t {
        Target::Item { .. }
        | Target::Trinket { .. }
        | Target::Character { .. }
        | Target::Achievement { .. }
        | Target::Challenge { .. } => true,
        Target::Entity { .. }
        | Target::Transformation { .. }
        | Target::Stage { .. }
        | Target::Room { .. }
        | Target::Pickup { .. } => false,
    }
}

pub fn generate(d: &Dataset) -> Requirements {
    let mut achievements = BTreeMap::new();
    let mut uses: BTreeMap<String, (String, u32)> = BTreeMap::new();
    for (&id, entry) in &d.achievements {
        let Infobox::Achievement { requirements, .. } = &entry.infobox else {
            // An achievement page with another infobox is a wiki anomaly, not our error:
            // it contributes no requirement and no inventory row.
            continue;
        };
        let mut refs = Vec::new();
        collect_refs(requirements, &mut refs);
        for r in &refs {
            if reduces_on_its_own(&r.target) {
                continue;
            }
            let key = target_key(&r.target, &r.label);
            let e = uses.entry(key).or_insert((r.label.clone(), 0));
            e.1 += 1;
        }
        achievements.insert(id, AchievementRefs { refs });
    }
    Requirements {
        schema_version: SCHEMA_VERSION,
        generated_from: GeneratedFrom {
            snapshot_at: d.meta.snapshot_at.clone(),
            max_revid: d.meta.max_revid,
        },
        achievements,
        // `BTreeMap` iterates sorted: the file is stable across runs, which is what
        // makes the `derived` test meaningful.
        targets: uses
            .into_iter()
            .map(|(key, (label, uses))| TargetRow { key, label, uses })
            .collect(),
    }
}
```

Add `pub mod generate;` to `crates/graph/src/lib.rs`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p graph --test generate && cargo test -p wiki`
Expected: PASS, 4 new tests; the `wiki` suite still green.

- [ ] **Step 5: Commit**

```bash
git add crates/graph crates/wiki/src/dataset.rs
git commit -m "feat(graph): generate typed requirements from the wiki snapshot"
```

---

### Task 3: The generator binary, the committed file, and the `derived` test

**Files:**
- Create: `crates/graph/src/bin/graph-rules.rs`
- Create: `crates/graph/rules/requirements.json` (generated, committed)
- Create: `crates/graph/tests/derived.rs`
- Modify: `package.json` (root, `scripts`)

**Interfaces:**
- Consumes: `graph::generate::generate` from Task 2.
- Produces: the committed `crates/graph/rules/requirements.json`, and the `graph:rules` script.

- [ ] **Step 1: Write the failing test**

`crates/graph/tests/derived.rs`:

```rust
//! `requirements.json` is exactly `generate(dataset/wiki.json)`. The same guarantee
//! `crates/wiki`'s `derived` test gives the dataset: regenerate and forget to commit,
//! and this goes red instead of the two files drifting apart in silence.

use std::path::Path;

fn repo(rel: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join(rel)
}

#[test]
fn requirements_json_is_what_the_generator_produces() {
    let wiki_json = std::fs::read_to_string(repo("dataset/wiki.json")).expect("dataset/wiki.json");
    let d = wiki::Dataset::from_json(&wiki_json).expect("the dataset parses");
    let expected = serde_json::to_string_pretty(&graph::generate::generate(&d)).expect("serializes");
    let committed = std::fs::read_to_string(repo("crates/graph/rules/requirements.json"))
        .expect("crates/graph/rules/requirements.json");
    assert_eq!(
        committed.replace("\r\n", "\n"),
        format!("{expected}\n"),
        "requirements.json differs from generate(dataset/wiki.json): run `pnpm graph:rules`"
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p graph --test derived`
Expected: FAIL — `crates/graph/rules/requirements.json` does not exist.

- [ ] **Step 3: Write minimal implementation**

`crates/graph/src/bin/graph-rules.rs`:

```rust
//! Writes `crates/graph/rules/requirements.json` from `dataset/wiki.json`. Offline, one
//! pass per snapshot: the app never runs this.

use std::path::Path;

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let src = root.join("dataset/wiki.json");
    let json = match std::fs::read_to_string(&src) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("cannot read {}: {e}", src.display());
            std::process::exit(1);
        }
    };
    let dataset = match wiki::Dataset::from_json(&json) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("invalid dataset: {e}");
            std::process::exit(1);
        }
    };
    let out = root.join("crates/graph/rules/requirements.json");
    let requirements = graph::generate::generate(&dataset);
    let body = match serde_json::to_string_pretty(&requirements) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("cannot serialize: {e}");
            std::process::exit(1);
        }
    };
    if let Some(dir) = out.parent() {
        if let Err(e) = std::fs::create_dir_all(dir) {
            eprintln!("cannot create {}: {e}", dir.display());
            std::process::exit(1);
        }
    }
    if let Err(e) = std::fs::write(&out, format!("{body}\n")) {
        eprintln!("cannot write {}: {e}", out.display());
        std::process::exit(1);
    }
    eprintln!(
        "{} achievements, {} targets to curate -> {}",
        requirements.achievements.len(),
        requirements.targets.len(),
        out.display()
    );
}
```

Add to the root `package.json` `scripts`, next to `wiki:build`:

```json
    "graph:rules": "cargo run -p graph --bin graph-rules"
```

Then generate the file:

```bash
pnpm graph:rules
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p graph --test derived`
Expected: PASS. The generator's stderr should report 641 achievements and ~75 targets; if the target count differs, read the difference before going on — it means the snapshot moved.

- [ ] **Step 5: Commit**

```bash
git add crates/graph package.json
git commit -m "feat(graph): generator binary and the committed requirements file"
```

---

### Task 4: Curate the verdicts

**Files:**
- Create: `crates/graph/rules/corrections.json`
- Test: `crates/graph/tests/curation.rs`

**Interfaces:**
- Consumes: `graph::rules::{Corrections, Requirements, Rules, Verdict}` from Task 1; the committed `requirements.json` from Task 3.
- Produces: the committed `crates/graph/rules/corrections.json`, complete for the current snapshot.

This is the one task whose content is judgement, not code. The inventory generated in Task 3 is the form; this task fills it in. Reference values measured on 2026-09-07 against snapshot `2026-09-04T17:33:31Z`: 75 rows, of which the thirteen most used are `entity:Isaac` (76 uses), `room:Boss Rush` (41), `entity:The Lamb` (41), `entity:Mother` (38), `entity:Satan` (37), `entity:Hush` (36), `entity:Mega Satan` (36), `entity:Delirium` (36), `entity:The Beast` (34), `pickup:Hard mode` (33), `stage:Basement` (28), `pickup:Completion Mark` (18), `entity:Mom` (12).

- [ ] **Step 1: Write the failing test**

`crates/graph/tests/curation.rs`:

```rust
//! The curation is complete for the snapshot in the repo, and it stays complete: when a
//! new snapshot introduces a target nobody has judged, this test names it. The red is
//! the point — a warning would mean nodes dropping to `Partial` with nobody noticing.

use graph::rules::{Corrections, Requirements, Rules};
use std::path::Path;

fn rules() -> Rules {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("rules");
    let r: Requirements =
        serde_json::from_str(&std::fs::read_to_string(dir.join("requirements.json")).expect("read"))
            .expect("requirements.json parses");
    let c: Corrections =
        serde_json::from_str(&std::fs::read_to_string(dir.join("corrections.json")).expect("read"))
            .expect("corrections.json parses");
    Rules::build(r, c).expect("rules build")
}

#[test]
fn every_target_has_a_verdict() {
    let rules = rules();
    let missing: Vec<String> = rules
        .targets()
        .iter()
        .filter(|t| rules.verdict(&t.key).is_none())
        .map(|t| format!("{} ({} uses)", t.key, t.uses))
        .collect();
    assert!(
        missing.is_empty(),
        "{} targets have no verdict in corrections.json; judge each one \
         (alwaysAvailable / behind / notAPrerequisite):\n{}",
        missing.len(),
        missing.join("\n")
    );
}

#[test]
fn the_alias_that_carries_fifteen_refs_is_there() {
    // The wiki writes "Jacob and Esau", the game writes "Jacob & Esau": 15 refs hang on
    // this one line, measured on 2026-09-07 against snapshot 2026-09-04T17:33:31Z.
    assert_eq!(rules().alias("Jacob and Esau"), "Jacob & Esau");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p graph --test curation`
Expected: FAIL — `corrections.json` does not exist, then FAIL listing every uncurated target.

- [ ] **Step 3: Write minimal implementation**

Create `crates/graph/rules/corrections.json` with the aliases and one verdict per row the failing test lists. The shape:

```json
{
  "schemaVersion": 1,
  "aliases": {
    "Jacob and Esau": "Jacob & Esau"
  },
  "verdicts": {
    "entity:Satan": { "alwaysAvailable": true },
    "entity:Mom": { "alwaysAvailable": true },
    "entity:Red Heart": { "notAPrerequisite": true },
    "stage:The Void": { "behind": { "achievement": 0 } }
  }
}
```

Judging rules, applied row by row against the list the test prints:

- **`alwaysAvailable`** — content reachable on a first run with no unlocks: Mom, Satan, Isaac, the early chapters, Boss Rush.
- **`behind: { achievement }`** — content gated by progression. Find the gating achievement by searching the catalog: `cargo run -p catalog --example …` is not needed; use `rg` over `crates/graph/rules/requirements.json` for the label, and the achievement text in `samples/sprites/achievements.xml` if extracted, or the wiki entry's title. **Record in the plan report which achievement id was chosen for each gate and why** — this is curation, and the reasoning is the artefact.
- **`notAPrerequisite`** — pickups and enemies that appear in the sentence without gating anything: `Red Heart`, `Soul Heart`, `Locked Chest`, `Lil' Batteries`, `Battery Bum`, `Shopkeeper`, `Portal`, `poops`, `rainbow poop`, `Krampus`, `Angel`, `Shell Game`, `Ultra Pride`, `Die`, `dying`.

When a gate's own gating achievement cannot be established with confidence, **do not guess**: leave the row out, let the test name it, and record it in the report as an open row. An `Unknown` requirement is honest; a wrong `behind` edge is a wrong graph.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p graph --test curation`
Expected: PASS, 2 tests.

- [ ] **Step 5: Commit**

```bash
git add crates/graph/rules/corrections.json crates/graph/tests/curation.rs
git commit -m "feat(graph): curated verdicts for the unreducible targets"
```

---

### Task 5: Resolve requirements against the user's catalog

**Files:**
- Create: `crates/graph/src/resolve.rs`
- Create: `crates/graph/src/model.rs`
- Modify: `crates/graph/src/lib.rs`
- Test: `crates/graph/tests/resolve.rs`

**Interfaces:**
- Consumes: `graph::rules::{Rules, Verdict, target_key}`; `catalog::{Catalog, ItemKind, Language}`.
- Produces: `graph::model::Requirement` (the enum from the spec) and `graph::resolve::requirement(&Catalog, &Rules, &RefRow) -> Requirement`, plus `graph::resolve::NameIndex::new(&Catalog)` with `character(&str)`, `boss(&str)`, `item(&str)`.

Resolution order, and it matters: **label first, id second**. The wiki's entity ids are the game's entity types, the catalog's `BossId` comes from `bossportraits.xml`, and the two do not line up (Gish is entity 43 and boss 19). The label is the bridge and it carried 468 of 488 entity refs on 2026-09-07.

- [ ] **Step 1: Write the failing test**

`crates/graph/tests/resolve.rs`:

```rust
use catalog::{Catalog, ItemKind};
use graph::model::Requirement;
use graph::rules::{Corrections, RefRow, Requirements, Rules};
use graph::resolve::requirement;
use wiki::Target;

const PLAYERS: &str = r#"<players root="gfx/">
  <player id="0" name="Isaac" portrait="a.png"/>
  <player id="19" name="Jacob &amp; Esau" portrait="b.png"/>
</players>"#;

const BOSSES: &str = r#"<bossportraits gfxroot="gfx/">
  <boss id="19" name="Gish" portrait="g.png"/>
  <boss id="84" name="Satan" portrait="s.png"/>
</bossportraits>"#;

fn catalog() -> Catalog {
    Catalog::build(|p| match p {
        "players.xml" => Some(PLAYERS.as_bytes().to_vec()),
        "bossportraits.xml" => Some(BOSSES.as_bytes().to_vec()),
        _ => None,
    })
}

fn rules(corrections: &str) -> Rules {
    let r: Requirements = serde_json::from_str(
        r#"{"schemaVersion":1,
            "generatedFrom":{"snapshotAt":"","maxRevid":0},
            "achievements":{},"targets":[]}"#,
    )
    .expect("parses");
    let c: Corrections = serde_json::from_str(corrections).expect("parses");
    Rules::build(r, c).expect("build")
}

fn row(target: Target, label: &str) -> RefRow {
    RefRow {
        target,
        label: label.into(),
    }
}

#[test]
fn an_entity_ref_resolves_to_a_boss_by_name_not_by_id() {
    let c = catalog();
    let rules = rules(r#"{"schemaVersion":1}"#);
    let r = requirement(
        &c,
        &rules,
        &row(
            Target::Entity {
                id: 43,
                variant: 0,
                subtype: 0,
            },
            "Gish",
        ),
    );
    assert_eq!(
        r,
        Requirement::Boss {
            id: catalog::BossId(19)
        },
        "entity 43 is boss 19: the id spaces differ, the name is the bridge"
    );
}

#[test]
fn an_alias_is_applied_before_the_lookup() {
    let c = catalog();
    let rules = rules(r#"{"schemaVersion":1,"aliases":{"Jacob and Esau":"Jacob & Esau"}}"#);
    let r = requirement(&c, &rules, &row(Target::Character { id: 19 }, "Jacob and Esau"));
    assert_eq!(
        r,
        Requirement::Character {
            id: catalog::CharacterId(19)
        }
    );
}

#[test]
fn always_available_drops_the_requirement() {
    let c = catalog();
    let rules = rules(r#"{"schemaVersion":1,"verdicts":{"room:Boss Rush":{"alwaysAvailable":true}}}"#);
    let r = requirement(
        &c,
        &rules,
        &row(
            Target::Room {
                name: "Boss Rush".into(),
            },
            "Boss Rush",
        ),
    );
    assert_eq!(r, Requirement::None, "content reachable on night one gates nothing");
}

#[test]
fn a_target_with_no_verdict_stays_unknown() {
    let c = catalog();
    let rules = rules(r#"{"schemaVersion":1}"#);
    let r = requirement(
        &c,
        &rules,
        &row(
            Target::Stage {
                name: "Nowhere".into(),
            },
            "Nowhere",
        ),
    );
    assert_eq!(
        r,
        Requirement::Unknown {
            label: "Nowhere".into()
        },
        "an uncurated target must never read as 'no prerequisite'"
    );
}

#[test]
fn a_name_the_catalog_does_not_know_is_unknown_not_dropped() {
    let c = catalog();
    let rules = rules(r#"{"schemaVersion":1}"#);
    let r = requirement(
        &c,
        &rules,
        &row(
            Target::Entity {
                id: 999,
                variant: 0,
                subtype: 0,
            },
            "Nobody",
        ),
    );
    assert_eq!(
        r,
        Requirement::Unknown {
            label: "Nobody".into()
        }
    );
}

#[test]
fn an_item_resolves_across_the_three_collectible_kinds() {
    let c = Catalog::build(|p| match p {
        "items.xml" => Some(
            r#"<items gfxroot="gfx/"><active id="35" name="The Bible" gfx="b.png"/></items>"#
                .as_bytes()
                .to_vec(),
        ),
        _ => None,
    });
    let rules = rules(r#"{"schemaVersion":1}"#);
    let r = requirement(&c, &rules, &row(Target::Item { id: 35 }, "The Bible"));
    assert_eq!(
        r,
        Requirement::Item {
            kind: ItemKind::Active,
            id: catalog::ItemId(35)
        },
        "the wiki has one item id space; ours is keyed by (kind, id)"
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p graph --test resolve`
Expected: FAIL — `graph::model` and `graph::resolve` do not exist.

- [ ] **Step 3: Write minimal implementation**

`crates/graph/src/model.rs`:

```rust
//! The domain types. They do not cross the IPC: `ipc` defines its own views, as it
//! already does for `ItemKindView` and `OriginView`.

use catalog::{BossId, ChallengeId, CharacterId, ItemId, ItemKind};

/// What an achievement demands. The type is the point: it says whether the prerequisite
/// has an achievement of its own behind it (character, challenge, item) or is content
/// available from the start (Satan, Mom).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Requirement {
    Character { id: CharacterId },
    Boss { id: BossId },
    Challenge { id: ChallengeId },
    Item { kind: ItemKind, id: ItemId },
    /// Stage, room, mode: outside the catalog by nature. `gate` is the target's key in
    /// the curated table — `kind:label` — and not an id of the game's.
    Gate { gate: String },
    /// Declared, never dropped: the node knows what it doesn't know.
    Unknown { label: String },
    /// Judged as gating nothing: `alwaysAvailable` or `notAPrerequisite`. It is kept as a
    /// variant rather than filtered away at the source so that resolution is total —
    /// every ref maps to exactly one outcome, and none disappears without a name.
    None,
}
```

`crates/graph/src/resolve.rs`:

```rust
//! From a generated ref to a typed requirement, against the catalog the user actually
//! has. Nothing here reads a file.

use std::collections::HashMap;

use catalog::{BossId, Catalog, ChallengeId, CharacterId, ItemId, ItemKind, Language};
use wiki::Target;

use crate::model::Requirement;
use crate::rules::{target_key, RefRow, Rules, Verdict};

/// English names to ids. Built once per graph: the resolution is by name, so this is the
/// hot path and a per-ref linear scan over 909 items would show.
pub struct NameIndex {
    characters: HashMap<String, CharacterId>,
    bosses: HashMap<String, BossId>,
    items: HashMap<String, (ItemKind, ItemId)>,
}

fn key(s: &str) -> String {
    s.trim().to_lowercase()
}

impl NameIndex {
    pub fn new(c: &Catalog) -> NameIndex {
        let en = Language::English;
        let mut characters = HashMap::new();
        for ch in c.characters() {
            characters.insert(key(c.text(&ch.name, en)), ch.id);
        }
        let mut bosses = HashMap::new();
        for b in c.bosses() {
            bosses.insert(key(&b.name), b.id);
        }
        let mut items = HashMap::new();
        for i in c.items() {
            items.insert(key(c.text(&i.name, en)), (i.kind, i.id));
        }
        NameIndex {
            characters,
            bosses,
            items,
        }
    }

    pub fn character(&self, name: &str) -> Option<CharacterId> {
        self.characters.get(&key(name)).copied()
    }

    pub fn boss(&self, name: &str) -> Option<BossId> {
        self.bosses.get(&key(name)).copied()
    }

    pub fn item(&self, name: &str) -> Option<(ItemKind, ItemId)> {
        self.items.get(&key(name)).copied()
    }
}

/// One ref, one outcome. Never `None` by omission: a target we can't judge becomes
/// `Requirement::Unknown` and demotes its node to `Partial`.
pub fn requirement(c: &Catalog, rules: &Rules, row: &RefRow) -> Requirement {
    let index = NameIndex::new(c);
    requirement_with(c, rules, &index, row)
}

/// The form used when resolving many refs: the caller builds the index once.
pub fn requirement_with(
    c: &Catalog,
    rules: &Rules,
    index: &NameIndex,
    row: &RefRow,
) -> Requirement {
    let label = rules.alias(&row.label).to_string();
    let verdict_key = target_key(&row.target, &label);
    let unknown = || Requirement::Unknown {
        label: label.clone(),
    };
    match &row.target {
        Target::Character { id } => index
            .character(&label)
            .or_else(|| c.character(CharacterId(*id)).map(|ch| ch.id))
            .map(|id| Requirement::Character { id })
            .unwrap_or_else(unknown),
        Target::Entity { .. } => index
            .boss(&label)
            .map(|id| Requirement::Boss { id })
            .unwrap_or_else(|| from_verdict(rules, &verdict_key, unknown)),
        Target::Challenge { number } => c
            .challenge(ChallengeId(*number))
            .map(|ch| Requirement::Challenge { id: ch.id })
            .unwrap_or_else(unknown),
        Target::Item { id } | Target::Trinket { id } => index
            .item(&label)
            .or_else(|| {
                [
                    ItemKind::Passive,
                    ItemKind::Active,
                    ItemKind::Familiar,
                    ItemKind::Trinket,
                ]
                .into_iter()
                .find_map(|k| c.item(k, ItemId(*id)).map(|i| (i.kind, i.id)))
            })
            .map(|(kind, id)| Requirement::Item { kind, id })
            .unwrap_or_else(unknown),
        // An achievement referenced directly is already a node: the edge is built in
        // Task 6, which is the only place that turns requirements into edges.
        Target::Achievement { id } => Requirement::Gate {
            gate: format!("achievement:{id}"),
        },
        Target::Transformation { .. } | Target::Stage { .. } | Target::Room { .. }
        | Target::Pickup { .. } => from_verdict(rules, &verdict_key, unknown),
    }
}

fn from_verdict(
    rules: &Rules,
    key: &str,
    unknown: impl Fn() -> Requirement,
) -> Requirement {
    match rules.verdict(key) {
        Some(Verdict::AlwaysAvailable(_)) | Some(Verdict::NotAPrerequisite(_)) => {
            Requirement::None
        }
        Some(Verdict::Behind { .. }) => Requirement::Gate {
            gate: key.to_string(),
        },
        None => unknown(),
    }
}
```

Add `pub mod model;` and `pub mod resolve;` to `crates/graph/src/lib.rs`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p graph --test resolve`
Expected: PASS, 6 tests.

- [ ] **Step 5: Commit**

```bash
git add crates/graph
git commit -m "feat(graph): resolve wiki targets onto the user's catalog"
```

---

### Task 6: Edges — from a requirement to the achievement behind it

**Files:**
- Create: `crates/graph/src/build.rs`
- Modify: `crates/graph/src/lib.rs`
- Test: `crates/graph/tests/build.rs`

**Interfaces:**
- Consumes: `graph::model::Requirement`, `graph::resolve::{NameIndex, requirement_with}`, `graph::rules::{Rules, Verdict}`.
- Produces: `graph::build::Graph` with `Graph::build(&Catalog, &Rules) -> Graph`, `Graph::nodes() -> &[Node]`, `Graph::node(u32) -> Option<&Node>`, `Graph::diagnostics() -> &[GraphDiagnostic]`; `graph::build::{Node, GraphDiagnostic}`.

`Node { pub achievement: u32, pub requirements: Vec<Requirement>, pub prerequisites: Vec<u32>, pub unknown: u32 }`. `prerequisites` are achievement ids, sorted and deduplicated.

A challenge unlocked by **more than one** achievement is a disjunction ("either of these"), and the model has no OR. It becomes `Requirement::Unknown` plus a `GraphDiagnostic::Disjunction` — honest, and the report records how many rows hit it.

- [ ] **Step 1: Write the failing test**

`crates/graph/tests/build.rs`:

```rust
use catalog::Catalog;
use graph::build::{Graph, GraphDiagnostic};
use graph::rules::{Corrections, Requirements, Rules};

const PLAYERS: &str = r#"<players root="gfx/">
  <player id="0" name="Isaac" portrait="a.png"/>
  <player id="1" name="Magdalene" portrait="b.png" achievement="1"/>
</players>"#;

const ACHIEVEMENTS: &str = r#"<achievements gfxroot="gfx/">
  <achievement id="1" name="Magdalene" text="Magdalene" gfx="1.png"/>
  <achievement id="2" name="Second" text="Second" gfx="2.png"/>
</achievements>"#;

fn catalog() -> Catalog {
    Catalog::build(|p| match p {
        "players.xml" => Some(PLAYERS.as_bytes().to_vec()),
        "achievements.xml" => Some(ACHIEVEMENTS.as_bytes().to_vec()),
        _ => None,
    })
}

fn rules(requirements: &str, corrections: &str) -> Rules {
    let r: Requirements = serde_json::from_str(requirements).expect("requirements parse");
    let c: Corrections = serde_json::from_str(corrections).expect("corrections parse");
    Rules::build(r, c).expect("build")
}

#[test]
fn a_character_requirement_becomes_an_edge_to_its_unlocking_achievement() {
    let c = catalog();
    let g = Graph::build(
        &c,
        &rules(
            r#"{"schemaVersion":1,"generatedFrom":{"snapshotAt":"","maxRevid":0},
                "achievements":{"2":{"refs":[
                  {"target":{"kind":"character","id":1},"label":"Magdalene"}]}},
                "targets":[]}"#,
            r#"{"schemaVersion":1}"#,
        ),
    );
    let node = g.node(2).expect("node 2");
    assert_eq!(
        node.prerequisites,
        vec![1],
        "Magdalene is unlocked by achievement 1: that is the edge, and it comes from the game"
    );
    assert_eq!(node.unknown, 0);
}

#[test]
fn content_available_from_the_start_produces_no_edge() {
    let c = catalog();
    let g = Graph::build(
        &c,
        &rules(
            r#"{"schemaVersion":1,"generatedFrom":{"snapshotAt":"","maxRevid":0},
                "achievements":{"2":{"refs":[
                  {"target":{"kind":"character","id":0},"label":"Isaac"}]}},
                "targets":[]}"#,
            r#"{"schemaVersion":1}"#,
        ),
    );
    let node = g.node(2).expect("node 2");
    assert!(
        node.prerequisites.is_empty(),
        "Isaac has no unlocked_by: no edge, and that is a fact, not a gap"
    );
    assert_eq!(node.unknown, 0, "no edge is not the same as unknown");
}

#[test]
fn an_uncurated_target_counts_as_unknown_on_its_node() {
    let c = catalog();
    let g = Graph::build(
        &c,
        &rules(
            r#"{"schemaVersion":1,"generatedFrom":{"snapshotAt":"","maxRevid":0},
                "achievements":{"2":{"refs":[
                  {"target":{"kind":"stage","name":"Nowhere"},"label":"Nowhere"}]}},
                "targets":[{"key":"stage:Nowhere","label":"Nowhere","uses":1}]}"#,
            r#"{"schemaVersion":1}"#,
        ),
    );
    assert_eq!(g.node(2).expect("node 2").unknown, 1);
}

#[test]
fn a_gate_edge_comes_from_the_verdict() {
    let c = catalog();
    let g = Graph::build(
        &c,
        &rules(
            r#"{"schemaVersion":1,"generatedFrom":{"snapshotAt":"","maxRevid":0},
                "achievements":{"2":{"refs":[
                  {"target":{"kind":"stage","name":"The Void"},"label":"The Void"}]}},
                "targets":[{"key":"stage:The Void","label":"The Void","uses":1}]}"#,
            r#"{"schemaVersion":1,
                "verdicts":{"stage:The Void":{"behind":{"achievement":1}}}}"#,
        ),
    );
    assert_eq!(g.node(2).expect("node 2").prerequisites, vec![1]);
}

#[test]
fn a_verdict_pointing_at_an_achievement_that_does_not_exist_is_diagnosed() {
    let c = catalog();
    let g = Graph::build(
        &c,
        &rules(
            r#"{"schemaVersion":1,"generatedFrom":{"snapshotAt":"","maxRevid":0},
                "achievements":{"2":{"refs":[
                  {"target":{"kind":"stage","name":"The Void"},"label":"The Void"}]}},
                "targets":[{"key":"stage:The Void","label":"The Void","uses":1}]}"#,
            r#"{"schemaVersion":1,
                "verdicts":{"stage:The Void":{"behind":{"achievement":999}}}}"#,
        ),
    );
    assert!(
        g.diagnostics().iter().any(|d| matches!(
            d,
            GraphDiagnostic::EdgeOutsideCatalog { achievement: 999, .. }
        )),
        "an edge to an id the catalog doesn't know must be named, got {:?}",
        g.diagnostics()
    );
    assert!(
        g.node(2).expect("node 2").prerequisites.is_empty(),
        "a dangling edge is dropped, not followed"
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p graph --test build`
Expected: FAIL — `graph::build` does not exist.

- [ ] **Step 3: Write minimal implementation**

`crates/graph/src/build.rs`:

```rust
//! The graph itself: one node per achievement the catalog knows, with the achievements
//! it sits behind. Edges come from the game's own `unlocked_by` links, never from the
//! wiki — the wiki only says *what* is needed.

use catalog::{AchievementId, Catalog};

use crate::model::Requirement;
use crate::resolve::{requirement_with, NameIndex};
use crate::rules::{Rules, Verdict};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub achievement: u32,
    pub requirements: Vec<Requirement>,
    /// Achievement ids this node sits behind. Sorted and deduplicated.
    pub prerequisites: Vec<u32>,
    /// How many requirements couldn't be interpreted. Above zero the node can never
    /// claim "available now".
    pub unknown: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphDiagnostic {
    /// A curated `behind` points at an achievement this catalog doesn't have.
    EdgeOutsideCatalog { node: u32, achievement: u32 },
    /// A challenge unlocked by several achievements: "either of these", which the model
    /// has no way to say. The requirement becomes unknown rather than wrong.
    Disjunction { node: u32, count: u32 },
    /// Nodes that form a cycle. Filled in during evaluation (Task 7).
    Cycle { nodes: Vec<u32> },
}

pub struct Graph {
    nodes: Vec<Node>,
    diagnostics: Vec<GraphDiagnostic>,
}

impl Graph {
    pub fn build(c: &Catalog, rules: &Rules) -> Graph {
        let index = NameIndex::new(c);
        let mut nodes = Vec::new();
        let mut diagnostics = Vec::new();
        for a in c.achievements() {
            let id = a.id.0;
            let mut requirements = Vec::new();
            let mut prerequisites = Vec::new();
            let mut unknown = 0u32;
            for row in rules.refs(id) {
                let r = requirement_with(c, rules, &index, row);
                match &r {
                    Requirement::Unknown { .. } => unknown += 1,
                    Requirement::None => {}
                    Requirement::Character { id: cid } => {
                        if let Some(by) = c.character(*cid).and_then(|ch| ch.unlocked_by) {
                            prerequisites.push(by.0);
                        }
                    }
                    Requirement::Boss { id: bid } => {
                        if let Some(by) = c.boss(*bid).and_then(|b| b.unlocked_by) {
                            prerequisites.push(by.0);
                        }
                    }
                    Requirement::Item { kind, id: iid } => {
                        if let Some(by) = c.item(*kind, *iid).and_then(|i| i.unlocked_by) {
                            prerequisites.push(by.0);
                        }
                    }
                    Requirement::Challenge { id: chid } => {
                        let by = c.challenge(*chid).map(|ch| ch.unlocked_by.clone()).unwrap_or_default();
                        match by.len() {
                            0 => {}
                            1 => prerequisites.push(by[0].0),
                            n => {
                                unknown += 1;
                                diagnostics.push(GraphDiagnostic::Disjunction {
                                    node: id,
                                    count: n as u32,
                                });
                            }
                        }
                    }
                    Requirement::Gate { gate } => {
                        if let Some(Verdict::Behind { achievement }) = rules.verdict(gate) {
                            if c.achievement(AchievementId(*achievement)).is_some() {
                                prerequisites.push(*achievement);
                            } else {
                                diagnostics.push(GraphDiagnostic::EdgeOutsideCatalog {
                                    node: id,
                                    achievement: *achievement,
                                });
                            }
                        }
                    }
                }
                requirements.push(r);
            }
            prerequisites.sort_unstable();
            prerequisites.dedup();
            nodes.push(Node {
                achievement: id,
                requirements,
                prerequisites,
                unknown,
            });
        }
        Graph { nodes, diagnostics }
    }

    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn node(&self, achievement: u32) -> Option<&Node> {
        self.nodes.iter().find(|n| n.achievement == achievement)
    }

    pub fn diagnostics(&self) -> &[GraphDiagnostic] {
        &self.diagnostics
    }
}
```

The `Gate` arm has to handle `Requirement::Gate { gate: "achievement:N" }` from Task 5 —
an achievement referenced directly, which has no verdict and would otherwise produce no
edge. The arm in full:

```rust
                    Requirement::Gate { gate } => {
                        // A ref straight to another achievement: the edge is the id
                        // itself, no verdict involved.
                        let direct = gate
                            .strip_prefix("achievement:")
                            .and_then(|n| n.parse::<u32>().ok());
                        let edge = match (direct, rules.verdict(gate)) {
                            (Some(id), _) => Some(id),
                            (None, Some(Verdict::Behind { achievement })) => Some(*achievement),
                            (None, Some(Verdict::AlwaysAvailable(_)))
                            | (None, Some(Verdict::NotAPrerequisite(_)))
                            | (None, None) => None,
                        };
                        if let Some(target) = edge {
                            if c.achievement(AchievementId(target)).is_some() {
                                prerequisites.push(target);
                            } else {
                                diagnostics.push(GraphDiagnostic::EdgeOutsideCatalog {
                                    node: id,
                                    achievement: target,
                                });
                            }
                        }
                    }
```

Add `pub mod build;` to `crates/graph/src/lib.rs` and re-export `Graph`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p graph --test build`
Expected: PASS, 5 tests.

- [ ] **Step 5: Commit**

```bash
git add crates/graph
git commit -m "feat(graph): edges from requirements to the achievements behind them"
```

---

### Task 7: Evaluation against a profile

**Files:**
- Create: `crates/graph/src/evaluate.rs`
- Modify: `crates/graph/src/lib.rs`, `crates/graph/src/build.rs`
- Test: `crates/graph/tests/evaluate.rs`

**Interfaces:**
- Consumes: `graph::build::{Graph, Node, GraphDiagnostic}`.
- Produces: `Graph::evaluate(flags: Option<&[bool]>) -> Eval`; `graph::evaluate::{Eval, NodeInfo}` with `Eval::node(u32) -> Option<&NodeInfo>` and `Eval::diagnostics() -> &[GraphDiagnostic]`.

```rust
pub enum NodeInfo {
    Computed { available_now: bool, blocked_by: u32, fan_out: u32, steps_missing: u32 },
    Partial { blocked_by: u32, fan_out: u32, unknown: u32 },
}
```

Definitions, from the spec, and they are the contract of this task: `blocked_by` counts **direct** prerequisites not yet done; `available_now` is `blocked_by == 0 && unknown == 0 && !done`; `fan_out` is how many nodes name this one as a direct prerequisite; `steps_missing` is the **cardinality of the transitive set** of not-done prerequisites — a node shared by two branches counts once. `Partial` carries no `steps_missing` on purpose.

`flags: None` means section 1 wasn't read: `evaluate` returns an `Eval` with no nodes, and the caller keeps its own diagnostic. `flags[i]` is slot `i`, slot 0 unused, per the mapping pinned on 2026-09-05.

- [ ] **Step 1: Write the failing test**

`crates/graph/tests/evaluate.rs`:

```rust
use graph::build::{Graph, GraphDiagnostic};
use graph::evaluate::NodeInfo;

/// Builds a graph straight from edges, without a catalog: this task is about the walk,
/// and mixing in XML fixtures would test Task 6 again.
fn graph(edges: &[(u32, &[u32])], unknown: &[(u32, u32)]) -> Graph {
    Graph::from_edges_for_tests(edges, unknown)
}

fn flags(done: &[u32], slots: usize) -> Vec<bool> {
    let mut f = vec![false; slots];
    for &d in done {
        f[d as usize] = true;
    }
    f
}

#[test]
fn a_node_with_no_prerequisites_is_available_now() {
    let g = graph(&[(1, &[])], &[]);
    let e = g.evaluate(Some(&flags(&[], 2)));
    assert_eq!(
        e.node(1),
        Some(&NodeInfo::Computed {
            available_now: true,
            blocked_by: 0,
            fan_out: 0,
            steps_missing: 0
        })
    );
}

#[test]
fn a_done_prerequisite_stops_blocking() {
    let g = graph(&[(1, &[]), (2, &[1])], &[]);
    let blocked = g.evaluate(Some(&flags(&[], 3)));
    assert_eq!(
        blocked.node(2),
        Some(&NodeInfo::Computed {
            available_now: false,
            blocked_by: 1,
            fan_out: 0,
            steps_missing: 1
        })
    );
    let freed = g.evaluate(Some(&flags(&[1], 3)));
    assert_eq!(
        freed.node(2),
        Some(&NodeInfo::Computed {
            available_now: true,
            blocked_by: 0,
            fan_out: 0,
            steps_missing: 0
        })
    );
}

#[test]
fn steps_missing_counts_a_shared_ancestor_once() {
    // 4 needs 2 and 3; both need 1. Three runs, not four.
    let g = graph(&[(1, &[]), (2, &[1]), (3, &[1]), (4, &[2, 3])], &[]);
    let e = g.evaluate(Some(&flags(&[], 5)));
    let NodeInfo::Computed { steps_missing, .. } = e.node(4).expect("node 4") else {
        panic!("node 4 should be Computed: {:?}", e.node(4));
    };
    assert_eq!(*steps_missing, 3, "1, 2 and 3 — the shared ancestor counts once");
}

#[test]
fn fan_out_counts_the_nodes_this_one_opens() {
    let g = graph(&[(1, &[]), (2, &[1]), (3, &[1])], &[]);
    let e = g.evaluate(Some(&flags(&[], 4)));
    let NodeInfo::Computed { fan_out, .. } = e.node(1).expect("node 1") else {
        panic!("node 1 should be Computed");
    };
    assert_eq!(*fan_out, 2);
}

#[test]
fn an_unknown_requirement_makes_the_node_partial() {
    let g = graph(&[(1, &[])], &[(1, 2)]);
    let e = g.evaluate(Some(&flags(&[], 2)));
    assert_eq!(
        e.node(1),
        Some(&NodeInfo::Partial {
            blocked_by: 0,
            fan_out: 0,
            unknown: 2
        }),
        "zero prerequisites is not 'available now' when something wasn't understood"
    );
}

#[test]
fn a_cycle_is_declared_and_never_walked_twice() {
    let g = graph(&[(1, &[2]), (2, &[1])], &[]);
    let e = g.evaluate(Some(&flags(&[], 3)));
    assert!(
        e.diagnostics()
            .iter()
            .any(|d| matches!(d, GraphDiagnostic::Cycle { .. })),
        "a cycle must be named, got {:?}",
        e.diagnostics()
    );
    assert!(
        matches!(e.node(1), Some(NodeInfo::Partial { .. })),
        "a node inside a cycle cannot claim a transitive count"
    );
}

#[test]
fn a_node_already_done_is_not_available_now() {
    let g = graph(&[(1, &[])], &[]);
    let e = g.evaluate(Some(&flags(&[1], 2)));
    let NodeInfo::Computed { available_now, steps_missing, .. } = e.node(1).expect("node 1") else {
        panic!("node 1 should be Computed");
    };
    assert!(!available_now, "it is done: there is nothing to unlock");
    assert_eq!(*steps_missing, 0);
}

#[test]
fn without_section_one_there_are_no_nodes_and_no_invented_zeros() {
    let g = graph(&[(1, &[])], &[]);
    let e = g.evaluate(None);
    assert_eq!(e.node(1), None, "unread is not the same as not done");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p graph --test evaluate`
Expected: FAIL — `graph::evaluate` and `Graph::from_edges_for_tests` do not exist.

- [ ] **Step 3: Write minimal implementation**

Add to `crates/graph/src/build.rs`:

```rust
impl Graph {
    /// A graph straight from edges, for tests on the walk that don't need a catalog.
    pub fn from_edges_for_tests(edges: &[(u32, &[u32])], unknown: &[(u32, u32)]) -> Graph {
        let nodes = edges
            .iter()
            .map(|(id, prerequisites)| Node {
                achievement: *id,
                requirements: Vec::new(),
                prerequisites: prerequisites.to_vec(),
                unknown: unknown
                    .iter()
                    .find(|(n, _)| n == id)
                    .map(|(_, u)| *u)
                    .unwrap_or(0),
            })
            .collect();
        Graph {
            nodes,
            diagnostics: Vec::new(),
        }
    }
}
```

`crates/graph/src/evaluate.rs`:

```rust
//! The walk against a profile. Pure arithmetic on the edges built in `build.rs`: the
//! save is only ever a slice of booleans.

use std::collections::{BTreeMap, BTreeSet};

use crate::build::{Graph, GraphDiagnostic};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeInfo {
    Computed {
        available_now: bool,
        blocked_by: u32,
        fan_out: u32,
        steps_missing: u32,
    },
    /// Requirements only partly interpreted, or a node caught in a cycle. It carries no
    /// `steps_missing`: the transitive count isn't knowable, and a zero would be the
    /// exact lie this variant exists to prevent.
    Partial {
        blocked_by: u32,
        fan_out: u32,
        unknown: u32,
    },
}

pub struct Eval {
    infos: BTreeMap<u32, NodeInfo>,
    diagnostics: Vec<GraphDiagnostic>,
}

impl Eval {
    pub fn node(&self, achievement: u32) -> Option<&NodeInfo> {
        self.infos.get(&achievement)
    }

    pub fn diagnostics(&self) -> &[GraphDiagnostic] {
        &self.diagnostics
    }
}

impl Graph {
    pub fn evaluate(&self, flags: Option<&[bool]>) -> Eval {
        let Some(flags) = flags else {
            // Section 1 wasn't read. No nodes: "unread" must not become "not done".
            return Eval {
                infos: BTreeMap::new(),
                diagnostics: Vec::new(),
            };
        };
        let done = |id: u32| flags.get(id as usize).copied().unwrap_or(false);

        let mut fan_out: BTreeMap<u32, u32> = BTreeMap::new();
        for n in self.nodes() {
            for &p in &n.prerequisites {
                *fan_out.entry(p).or_insert(0) += 1;
            }
        }

        let mut missing: BTreeMap<u32, Option<BTreeSet<u32>>> = BTreeMap::new();
        let mut cycles: Vec<Vec<u32>> = Vec::new();
        for n in self.nodes() {
            let mut stack = Vec::new();
            transitive(self, n.achievement, &done, &mut missing, &mut stack, &mut cycles);
        }

        let mut infos = BTreeMap::new();
        for n in self.nodes() {
            let id = n.achievement;
            let blocked_by = n.prerequisites.iter().filter(|&&p| !done(p)).count() as u32;
            let fan = fan_out.get(&id).copied().unwrap_or(0);
            let transitive_known = missing.get(&id).and_then(|m| m.as_ref());
            let info = match (n.unknown, transitive_known) {
                (0, Some(set)) => NodeInfo::Computed {
                    available_now: blocked_by == 0 && !done(id),
                    blocked_by,
                    fan_out: fan,
                    steps_missing: set.len() as u32,
                },
                (unknown, _) => NodeInfo::Partial {
                    blocked_by,
                    fan_out: fan,
                    unknown,
                },
            };
            infos.insert(id, info);
        }

        let mut diagnostics = self.diagnostics().to_vec();
        for nodes in cycles {
            diagnostics.push(GraphDiagnostic::Cycle { nodes });
        }
        Eval { infos, diagnostics }
    }
}

/// The set of not-done achievements standing between the profile and this node.
/// `None` means "not knowable": the node sits in a cycle. Memoized, so each node is
/// computed once even when many nodes share an ancestor.
fn transitive(
    g: &Graph,
    id: u32,
    done: &impl Fn(u32) -> bool,
    memo: &mut BTreeMap<u32, Option<BTreeSet<u32>>>,
    stack: &mut Vec<u32>,
    cycles: &mut Vec<Vec<u32>>,
) -> Option<BTreeSet<u32>> {
    if let Some(hit) = memo.get(&id) {
        return hit.clone();
    }
    if stack.contains(&id) {
        let mut nodes = stack.clone();
        nodes.push(id);
        cycles.push(nodes);
        memo.insert(id, None);
        return None;
    }
    stack.push(id);
    let mut set = BTreeSet::new();
    let mut knowable = true;
    if let Some(node) = g.node(id) {
        for &p in &node.prerequisites {
            if !done(p) {
                set.insert(p);
            }
            match transitive(g, p, done, memo, stack, cycles) {
                Some(inner) => set.extend(inner),
                None => knowable = false,
            }
        }
    }
    stack.pop();
    let out = if knowable { Some(set) } else { None };
    memo.insert(id, out.clone());
    out
}
```

Add `pub mod evaluate;` to `crates/graph/src/lib.rs`, and derive `Clone` on `GraphDiagnostic` if it isn't already.

`Graph { nodes, diagnostics }` is constructed in `build.rs`; make both fields `pub(crate)` so `from_edges_for_tests` and `evaluate` can reach them.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p graph --test evaluate`
Expected: PASS, 8 tests.

- [ ] **Step 5: Commit**

```bash
git add crates/graph
git commit -m "feat(graph): evaluate a profile against the graph"
```

---

### Task 8: Embed the rules

**Files:**
- Create: `crates/graph/build.rs`
- Modify: `crates/graph/Cargo.toml`, `crates/graph/src/rules.rs`
- Test: `crates/graph/tests/embedded.rs`

**Interfaces:**
- Consumes: the two committed files under `crates/graph/rules/`.
- Produces: `graph::rules::embedded() -> Result<&'static Rules, &'static RulesError>`.

Same shape as `wiki::Dataset::embedded()`, and for the same reason: there is deliberately no runtime path for "rules missing", because there is no file that can be missing. No compression here — the two files are small; `wiki.json` needed it at 21 MB.

- [ ] **Step 1: Write the failing test**

`crates/graph/tests/embedded.rs`:

```rust
#[test]
fn the_embedded_rules_are_the_committed_ones() {
    let rules = graph::rules::embedded().expect("the embedded rules parse");
    assert!(
        !rules.targets().is_empty(),
        "the inventory must not be empty: the generator ran on a real snapshot"
    );
    assert!(
        rules.targets().iter().all(|t| rules.verdict(&t.key).is_some()),
        "the embedded rules carry a verdict for every target"
    );
    assert_eq!(
        rules.generated_from().snapshot_at,
        "2026-09-04T17:33:31Z",
        "era of the pinned numbers in this suite"
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p graph --test embedded`
Expected: FAIL — `graph::rules::embedded` does not exist.

- [ ] **Step 3: Write minimal implementation**

Add to `crates/graph/src/rules.rs`:

```rust
use std::sync::OnceLock;

static EMBEDDED: OnceLock<Result<Rules, RulesError>> = OnceLock::new();

/// The rules compiled into the binary. There is no runtime path for "rules missing":
/// there is no file that can be missing.
pub fn embedded() -> Result<&'static Rules, &'static RulesError> {
    EMBEDDED
        .get_or_init(|| {
            let r: Requirements = serde_json::from_str(include_str!("../rules/requirements.json"))
                .map_err(|e| RulesError::Malformed {
                    reason: e.to_string(),
                })?;
            let c: Corrections = serde_json::from_str(include_str!("../rules/corrections.json"))
                .map_err(|e| RulesError::Malformed {
                    reason: e.to_string(),
                })?;
            Rules::build(r, c)
        })
        .as_ref()
}
```

`crates/graph/build.rs`, so a change to either file triggers a rebuild:

```rust
fn main() {
    println!("cargo:rerun-if-changed=rules/requirements.json");
    println!("cargo:rerun-if-changed=rules/corrections.json");
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p graph`
Expected: PASS, the whole `graph` suite.

- [ ] **Step 5: Commit**

```bash
git add crates/graph
git commit -m "feat(graph): embed the rules at build time"
```

---

### Task 9: The IPC contract

**Files:**
- Modify: `crates/ipc/src/graph.rs`
- Modify: `crates/ipc/Cargo.toml` (add `graph = { path = "../graph" }`)
- Modify: `ui/src/lib/ipc/types.ts`
- Modify: `ui/src/App.vue` (the branch that draws `stub`)
- Test: `crates/ipc/tests/graph_shape.rs` (extend the existing JSON-shape tests)

**Interfaces:**
- Consumes: `graph::evaluate::{Eval, NodeInfo}`.
- Produces: `ipc::graph::RequirementView`; `GraphInfo::Partial`; `GraphInfo::Stub` removed; `unlock_view(catalog, flags, info: Option<&graph::evaluate::Eval>, requirements: impl Fn(u32) -> Vec<RequirementView>, icon)`.

Three changes, and the third is the only non-additive one in M2:

1. `UnlockNode` gains `missing: Vec<RequirementView>` — the typed list of what's absent, which is what the screen groups by.
2. `GraphInfo` gains `Partial { blocked_by, fan_out, unknown }`.
3. `GraphInfo::Stub` leaves the wire, together with its TypeScript mirror and the branch that draws it.

`RequirementView` names the item-kind field `item_kind`, not `kind`: `kind` is already the tag, the same lesson `ItemKindView` carries.

- [ ] **Step 1: Write the failing test**

Add to `crates/ipc/tests/graph_shape.rs`:

```rust
#[test]
fn requirement_view_shapes() {
    use ipc::graph::RequirementView;
    assert_eq!(
        serde_json::to_value(RequirementView::Character {
            id: 1,
            name: "Magdalene".into()
        })
        .expect("serializes"),
        serde_json::json!({ "kind": "character", "id": 1, "name": "Magdalene" })
    );
    assert_eq!(
        serde_json::to_value(RequirementView::Item {
            item_kind: ipc::ItemKindView::Passive,
            id: 35,
            name: "The Bible".into()
        })
        .expect("serializes"),
        serde_json::json!({
            "kind": "item", "itemKind": "passive", "id": 35, "name": "The Bible"
        }),
        "`kind` is the tag: the item's own kind is `itemKind`, and a fieldless enum is a \
         bare string"
    );
    assert_eq!(
        serde_json::to_value(RequirementView::Unknown {
            label: "Nowhere".into()
        })
        .expect("serializes"),
        serde_json::json!({ "kind": "unknown", "label": "Nowhere" })
    );
}

#[test]
fn graph_info_partial_shape() {
    use ipc::graph::GraphInfo;
    assert_eq!(
        serde_json::to_value(GraphInfo::Partial {
            blocked_by: 1,
            fan_out: 2,
            unknown: 3
        })
        .expect("serializes"),
        serde_json::json!({
            "kind": "partial", "blockedBy": 1, "fanOut": 2, "unknown": 3
        }),
        "rename_all_fields is what keeps blockedBy from arriving as blocked_by"
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p ipc --test graph_shape`
Expected: FAIL — `RequirementView` and `GraphInfo::Partial` do not exist.

- [ ] **Step 3: Write minimal implementation**

In `crates/ipc/src/graph.rs`:

```rust
/// What a node is still missing, typed by the nature of the target: this is what the
/// screen groups by, so it can say "1 character and 2 bosses" instead of "blocked by 3".
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum RequirementView {
    Character { id: u32, name: String },
    Boss { id: u32, name: String },
    Challenge { id: u32, name: String },
    Item { item_kind: ItemKindView, id: u32, name: String },
    /// A curated gate: stage, room, mode. The label is what the wiki calls it.
    Gate { label: String },
    /// Not interpreted. The node carrying one cannot claim "available now".
    Unknown { label: String },
}
```

Replace `GraphInfo`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum GraphInfo {
    Computed {
        available_now: bool,
        blocked_by: u32,
        fan_out: u32,
        steps_missing: u32,
    },
    /// Requirements only partly interpreted, or a node inside a cycle: it carries no
    /// `steps_missing` on purpose.
    Partial {
        blocked_by: u32,
        fan_out: u32,
        unknown: u32,
    },
}
```

Add `missing: Vec<RequirementView>` to `UnlockNode`. `unlock_view` gains one parameter and
fills both new fields:

```rust
pub fn unlock_view(
    catalog: Option<&Catalog>,
    flags: Option<&[bool]>,
    eval: Option<&graph::evaluate::Eval>,
    graph: Option<&graph::Graph>,
    mut icon: impl FnMut(&str) -> Option<Vec<u8>>,
) -> UnlockView {
```

and inside the per-slot loop, where `graph: GraphInfo::Stub` stood:

```rust
        let info = match eval.and_then(|e| e.node(slot)) {
            Some(graph::evaluate::NodeInfo::Computed {
                available_now,
                blocked_by,
                fan_out,
                steps_missing,
            }) => GraphInfo::Computed {
                available_now: *available_now,
                blocked_by: *blocked_by,
                fan_out: *fan_out,
                steps_missing: *steps_missing,
            },
            Some(graph::evaluate::NodeInfo::Partial {
                blocked_by,
                fan_out,
                unknown,
            }) => GraphInfo::Partial {
                blocked_by: *blocked_by,
                fan_out: *fan_out,
                unknown: *unknown,
            },
            // No graph for this slot — no catalog, or a slot beyond it. It is a node the
            // graph has nothing to say about, which is `Partial` with everything at zero
            // and one unknown: never `Computed`, which would read as "nothing blocks it".
            None => GraphInfo::Partial {
                blocked_by: 0,
                fan_out: 0,
                unknown: 1,
            },
        };
```

`missing` is built from the node's requirements, keeping only what is not yet satisfied:

```rust
/// The requirements still in the way, resolved to names. `Requirement::None` never
/// reaches here: it was judged as gating nothing.
fn missing_view(
    c: &Catalog,
    node: &graph::build::Node,
    flags: &[bool],
) -> Vec<RequirementView> {
    let en = catalog::Language::English;
    let done = |a: Option<catalog::AchievementId>| {
        a.and_then(|a| flags.get(a.0 as usize).copied()).unwrap_or(false)
    };
    let mut out = Vec::new();
    for r in &node.requirements {
        match r {
            graph::model::Requirement::None => {}
            graph::model::Requirement::Character { id } => {
                let Some(ch) = c.character(*id) else { continue };
                if !done(ch.unlocked_by) {
                    out.push(RequirementView::Character {
                        id: id.0,
                        name: c.text(&ch.name, en).to_string(),
                    });
                }
            }
            graph::model::Requirement::Boss { id } => {
                let Some(b) = c.boss(*id) else { continue };
                if !done(b.unlocked_by) {
                    out.push(RequirementView::Boss {
                        id: id.0,
                        name: b.name.clone(),
                    });
                }
            }
            graph::model::Requirement::Challenge { id } => {
                let Some(ch) = c.challenge(*id) else { continue };
                let unlocked = ch.unlocked_by.iter().any(|a| done(Some(*a)));
                if !ch.unlocked_by.is_empty() && !unlocked {
                    out.push(RequirementView::Challenge {
                        id: id.0,
                        name: ch.name.clone(),
                    });
                }
            }
            graph::model::Requirement::Item { kind, id } => {
                let Some(i) = c.item(*kind, *id) else { continue };
                if !done(i.unlocked_by) {
                    out.push(RequirementView::Item {
                        item_kind: kind_view(*kind),
                        id: id.0,
                        name: c.text(&i.name, en).to_string(),
                    });
                }
            }
            graph::model::Requirement::Gate { gate } => out.push(RequirementView::Gate {
                label: gate.split_once(':').map(|(_, l)| l).unwrap_or(gate).to_string(),
            }),
            graph::model::Requirement::Unknown { label } => {
                out.push(RequirementView::Unknown {
                    label: label.clone(),
                })
            }
        }
    }
    out
}
```

`next_steps` orders by fan-out descending, ties broken by achievement id ascending so the
list is stable between calls, and reports the basis it actually used:

```rust
pub fn next_steps(view: &UnlockView) -> NextSteps {
    let mut candidates: Vec<&UnlockNode> = view
        .nodes
        .iter()
        .filter(|n| {
            !n.done
                && matches!(
                    n.graph,
                    GraphInfo::Computed {
                        available_now: true,
                        ..
                    }
                )
        })
        .collect();
    candidates.sort_by_key(|n| {
        let fan = match n.graph {
            GraphInfo::Computed { fan_out, .. } | GraphInfo::Partial { fan_out, .. } => fan_out,
        };
        let id = match &n.achievement {
            AchievementRef::Known { id, .. } => *id,
            AchievementRef::Unknown { slot } => *slot,
        };
        (std::cmp::Reverse(fan), id)
    });
    NextSteps {
        steps: candidates.into_iter().take(STEPS).cloned().collect(),
        basis: StepsBasis::FanOut,
    }
}
```

Keep whatever signature `next_steps` has today if it differs — read it before editing, and
change only the body and the `basis`.

In `ui/src/lib/ipc/types.ts`, mirror `RequirementView`, add `Partial`, delete `Stub`, and add `missing` to `UnlockNode`. In `ui/src/App.vue`, delete the branch that renders `stub`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p ipc && pnpm typecheck && pnpm scan`
Expected: PASS on all three.

- [ ] **Step 5: Commit**

```bash
git add crates/ipc ui/src
git commit -m "feat(ipc): real graph info on the unlock nodes"
```

---

### Task 10: Wire it into the app

**Files:**
- Modify: `crates/app/src/lib.rs`

**Interfaces:**
- Consumes: `graph::{Graph, rules::embedded}`, `graph::evaluate::Eval`.
- Produces: `GraphState`, and the `unlock`, `next_steps`, `plan` commands passing a real `Eval`.

`GraphState(OnceLock<Graph>)` mirrors `CatalogState`: built once from the catalog and the embedded rules. Rules that fail to parse are an **expected case**, not an `Err`: the commands degrade to nodes without graph info rather than failing.

- [ ] **Step 1: Write the failing test**

The `app` crate is wiring and is not tested (`CLAUDE.md`). The check for this task is behavioural, run by hand:

```bash
pnpm dev
```

Expected on screen, on the live profile: the Unlock totals unchanged from before M2, and the five Next steps now ordered by fan-out with a non-empty `missing` list on at least some rows.

- [ ] **Step 2: Verify the current state first**

Run `pnpm dev` **before** touching `app` and note the five Next steps shown. They are ordered by slot today; after this task the order changes, and knowing the old list is how you tell "it changed" from "it broke".

- [ ] **Step 3: Write the implementation**

```rust
/// The graph, built once from the catalog and the embedded rules. Same shape as
/// `CatalogState`: expensive to build, cheap to consult, and an expected failure (rules
/// that don't parse) degrades instead of being cached as an error.
#[derive(Default)]
struct GraphState(OnceLock<Graph>);

impl GraphState {
    fn get(&self, catalog: &Catalog) -> Option<&Graph> {
        if let Some(g) = self.0.get() {
            return Some(g);
        }
        let rules = graph::rules::embedded().ok()?;
        Some(self.0.get_or_init(|| Graph::build(catalog, rules)))
    }
}
```

Register it with `.manage(GraphState::default())`, take `graph: tauri::State<'_, GraphState>` in `unlock`, `next_steps` and `plan`, and pass `graph.get(catalog).map(|g| g.evaluate(flags))` into `ipc::unlock_view`.

- [ ] **Step 4: Verify**

Run: `pnpm dev`
Expected: the window opens, Unlock totals match what Step 2 recorded, Next steps is reordered, and no row shows an empty `missing` **and** `available_now: false` at once — that combination would mean a node blocked by nothing, which is the bug this whole design is built to prevent.

- [ ] **Step 5: Commit**

```bash
git add crates/app/src/lib.rs
git commit -m "feat(app): build the unlock graph once and serve it to the commands"
```

---

### Task 11: The tests that only real data can run

**Files:**
- Create: `crates/graph/tests/real_data.rs`
- Create: `crates/graph/tests/series.rs`
- Create: `crates/graph/tests/cross_check.rs`

**Interfaces:**
- Consumes: everything above; `test_support::{packed_dir, dated_series, sample_bytes, skip}`; `unpack::ResourceSet`; `core_save`.
- Produces: nothing new — this task is the verification the spec promised.

- [ ] **Step 1: Write the failing tests**

`crates/graph/tests/real_data.rs` — the invariants:

```rust
//! Invariants that survive any patch of the game and any snapshot of the wiki. They are
//! deliberately not pinned numbers: the graph moves with the world, the invariants don't.

use graph::evaluate::NodeInfo;
use graph::Graph;

mod support;

#[test]
fn available_now_and_blocked_by_never_contradict_each_other() {
    let Some((g, flags)) = support::real_graph_and_flags() else {
        return;
    };
    let e = g.evaluate(Some(&flags));
    for n in g.nodes() {
        let Some(info) = e.node(n.achievement) else {
            continue;
        };
        if let NodeInfo::Computed {
            available_now,
            blocked_by,
            ..
        } = info
        {
            assert!(
                !(*available_now && *blocked_by > 0),
                "node {} claims available_now with {blocked_by} prerequisites missing",
                n.achievement
            );
        }
    }
}

#[test]
fn steps_missing_is_zero_exactly_when_the_node_is_done_or_available() {
    let Some((g, flags)) = support::real_graph_and_flags() else {
        return;
    };
    let e = g.evaluate(Some(&flags));
    for n in g.nodes() {
        let Some(NodeInfo::Computed {
            available_now,
            steps_missing,
            ..
        }) = e.node(n.achievement)
        else {
            continue;
        };
        let done = flags.get(n.achievement as usize).copied().unwrap_or(false);
        assert_eq!(
            *steps_missing == 0,
            done || *available_now,
            "node {} : steps_missing={steps_missing}, done={done}, available={available_now}",
            n.achievement
        );
    }
}

#[test]
fn no_edge_points_outside_the_catalog() {
    let Some((g, _)) = support::real_graph_and_flags() else {
        return;
    };
    let known: std::collections::BTreeSet<u32> =
        g.nodes().iter().map(|n| n.achievement).collect();
    for n in g.nodes() {
        for p in &n.prerequisites {
            assert!(
                known.contains(p),
                "node {} points at achievement {p}, which the catalog doesn't have — \
                 this is the silent failure mode of name resolution",
                n.achievement
            );
        }
    }
}

#[test]
fn the_era_numbers_hold_for_this_snapshot() {
    let Some((g, _)) = support::real_graph_and_flags() else {
        return;
    };
    // Measured on 2026-09-07 against snapshot 2026-09-04T17:33:31Z. Pinned, not fragile:
    // it moves when requirements.json moves, which the `derived` test already guards.
    let unknown: u32 = g.nodes().iter().map(|n| n.unknown).sum();
    let edges: usize = g.nodes().iter().map(|n| n.prerequisites.len()).sum();
    eprintln!("graph: {} nodes, {edges} edges, {unknown} unknown requirements", g.nodes().len());
    assert!(
        edges > 1500,
        "the graph collapsed to {edges} edges: resolution is broken, not the world"
    );
}
```

`crates/graph/tests/support/mod.rs`:

```rust
//! Shared setup for the real-data tests. Every function says on stderr which slice of
//! the real domain it ran on, or why it skipped: a green suite that skipped everything
//! is the failure mode this crate exists to prevent.

use catalog::Catalog;
use graph::Graph;

use core_save::{Kind, Save};
use graph::evaluate::NodeInfo;
use std::collections::BTreeMap;

/// The real catalog, built once per test from the game's archives.
fn real_catalog() -> Option<(Catalog, unpack::ResourceSet)> {
    let Some(packed) = test_support::packed_dir() else {
        test_support::skip("samples/packed missing: no game to build the catalog from");
        return None;
    };
    let rs = unpack::ResourceSet::open(&packed);
    let c = Catalog::build(|p| rs.read(p));
    Some((c, rs))
}

pub fn real_graph_and_flags() -> Option<(Graph, Vec<bool>)> {
    let (catalog, _rs) = real_catalog()?;
    let series = test_support::dated_series(".rep+persistentgamedata1.dat");
    let Some(save) = series.last() else {
        test_support::skip("no dated save in samples/: nothing to evaluate against");
        return None;
    };
    let s = match Save::open(save) {
        Ok(s) => s,
        Err(_) => {
            // A sample that is present but unreadable is declared, never a silent skip.
            test_support::skip("the most recent dated save exists but doesn't read");
            return None;
        }
    };
    let Some(flags) = s.flags(Kind::Achievements) else {
        test_support::skip("section 1 missing from the most recent dated save");
        return None;
    };
    let rules = match graph::rules::embedded() {
        Ok(r) => r,
        Err(e) => panic!("the embedded rules must parse: {e:?}"),
    };
    eprintln!("sample: {}", save.display());
    Some((Graph::build(&catalog, rules), flags))
}

/// One entry per dated save, oldest first: file name, the evaluation, and the flags.
/// Skips when fewer than two eras are present — a comparison needs two.
pub fn series_evals() -> Option<Vec<(String, BTreeMap<u32, NodeInfo>, Vec<bool>)>> {
    let (catalog, _rs) = real_catalog()?;
    let rules = match graph::rules::embedded() {
        Ok(r) => r,
        Err(e) => panic!("the embedded rules must parse: {e:?}"),
    };
    let g = Graph::build(&catalog, rules);
    let mut out = Vec::new();
    for path in test_support::dated_series(".rep+persistentgamedata1.dat") {
        let Ok(s) = Save::open(&path) else { continue };
        let Some(flags) = s.flags(Kind::Achievements) else {
            continue;
        };
        let e = g.evaluate(Some(&flags));
        let infos: BTreeMap<u32, NodeInfo> = g
            .nodes()
            .iter()
            .filter_map(|n| e.node(n.achievement).map(|i| (n.achievement, i.clone())))
            .collect();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        eprintln!("sample: {name}");
        out.push((name, infos, flags));
    }
    if out.len() < 2 {
        test_support::skip("fewer than two dated saves: the series properties need two eras");
        return None;
    }
    Some(out)
}
```

`NodeInfo` needs `Clone` for `series_evals`; add the derive in Task 7 if it isn't there.

`crates/graph/tests/series.rs` — the two properties over the historical series:

```rust
//! Properties over the historical series. These find what a pinned number cannot: they
//! compare an era against the next one, and the graph has to stay coherent across both.

use graph::evaluate::NodeInfo;

mod support;

#[test]
fn an_available_node_is_never_taken_away() {
    let Some(series) = support::series_evals() else {
        return;
    };
    for pair in series.windows(2) {
        let (before, after) = (&pair[0], &pair[1]);
        for (id, info) in &before.1 {
            let NodeInfo::Computed { available_now: true, .. } = info else {
                continue;
            };
            let done_after = after.2.get(*id as usize).copied().unwrap_or(false);
            let still = matches!(
                after.1.get(id),
                Some(NodeInfo::Computed { available_now: true, .. })
            );
            assert!(
                done_after || still,
                "node {id} was unlockable in {} and is neither done nor unlockable in {}: \
                 achievements are not lost in this game, so the graph is wrong",
                before.0,
                after.0
            );
        }
    }
}

#[test]
fn steps_missing_never_grows() {
    let Some(series) = support::series_evals() else {
        return;
    };
    for pair in series.windows(2) {
        let (before, after) = (&pair[0], &pair[1]);
        for (id, info) in &before.1 {
            let (
                NodeInfo::Computed { steps_missing: was, .. },
                Some(NodeInfo::Computed { steps_missing: now, .. }),
            ) = (info, after.1.get(id))
            else {
                continue;
            };
            assert!(
                now <= was,
                "node {id}: {was} steps in {} became {now} in {} — progress does not undo",
                before.0,
                after.0
            );
        }
    }
}
```

Add `series_evals()` to `support/mod.rs`, returning `Vec<(String, BTreeMap<u32, NodeInfo>, Vec<bool>)>` — one entry per dated save, in date order, each printing its own `sample:` line. Skip with a note when fewer than two saves are present: two eras are the minimum a comparison needs.

`crates/graph/tests/cross_check.rs` — the wiki against the game's own files:

```rust
//! Two independent derivations of the same relation: the wiki's `unlocks` and the
//! catalog's own `unlocked_by` links. They agreed on 397 of 404 on 2026-09-07. The seven
//! divergences below are not errors — six are two different id spaces, one is a genuine
//! ambiguity — and a new one is news, to be read before it is added here.

mod support;

/// Achievement ids whose wiki `unlocks` disagrees with the catalog, with the reason.
const KNOWN_DIVERGENCES: &[(u32, &str)] = &[
    (16, "entity id vs boss id: Steven is entity 79, boss 20"),
    (17, "entity id vs boss id: C.H.A.D. is entity 28, boss 21"),
    (18, "entity id vs boss id: Gish is entity 43, boss 19"),
    (34, "entity id vs boss id: It Lives! is entity 78, boss 25"),
    (66, "entity id vs boss id: Conquest is entity 65, boss 38"),
    (68, "entity id vs boss id: Triachnid is entity 101, boss 42"),
    (132, "The Soul: character 17 for the wiki, item 335 for the catalog"),
];

#[test]
fn the_wiki_and_the_game_still_agree_where_they_both_speak() {
    let Some((catalog, _rs)) = support::real_catalog_public() else {
        return;
    };
    let Ok(dataset) = wiki::Dataset::embedded() else {
        test_support::skip("the embedded wiki dataset doesn't parse");
        return;
    };
    let known: std::collections::BTreeMap<u32, &str> =
        KNOWN_DIVERGENCES.iter().map(|(id, why)| (*id, *why)).collect();

    let (mut both, mut agreed) = (0u32, 0u32);
    let mut unexpected = Vec::new();
    for (&id, entry) in &dataset.achievements {
        let wiki::Infobox::Achievement { unlocks: Some(u), .. } = &entry.infobox else {
            continue;
        };
        let ours = catalog.unlocks(catalog::AchievementId(id));
        if ours.is_empty() {
            continue;
        }
        both += 1;
        if ours.iter().any(|o| same_thing(o, u)) {
            agreed += 1;
        } else if !known.contains_key(&id) {
            unexpected.push(format!("achievement {id}: wiki says {u:?}, catalog says {ours:?}"));
        }
    }
    eprintln!("cross-check: {agreed} of {both} agree ({} known divergences)", known.len());
    assert!(
        unexpected.is_empty(),
        "new divergences between the wiki and the game's files — read each one before \
         adding it to KNOWN_DIVERGENCES:\n{}",
        unexpected.join("\n")
    );
}

/// Do the two sources point at the same thing? Item kinds collapse — the wiki has one
/// collectible id space, we key by `(kind, id)` — and an entity is compared by nothing
/// here: entity ids and boss ids are different spaces, which is why six of the seven
/// known divergences exist.
fn same_thing(ours: &catalog::Unlock, theirs: &wiki::Target) -> bool {
    match (ours, theirs) {
        (catalog::Unlock::Item { kind, id }, wiki::Target::Item { id: w }) => {
            *kind != catalog::ItemKind::Trinket && id.0 == *w
        }
        (catalog::Unlock::Item { kind, id }, wiki::Target::Trinket { id: w }) => {
            *kind == catalog::ItemKind::Trinket && id.0 == *w
        }
        (catalog::Unlock::Character { id }, wiki::Target::Character { id: w }) => id.0 == *w,
        (catalog::Unlock::Challenge { id }, wiki::Target::Challenge { number }) => id.0 == *number,
        (catalog::Unlock::Boss { .. }, wiki::Target::Entity { .. }) => false,
        _ => false,
    }
}
```

The `_ => false` arm here is on a **pair** of enums, not a single closed enum: the
exhaustiveness rule bans a catch-all that hides a new variant of one type, and a match on
two independent enums has 40 combinations of which 5 mean anything. Keep the arm, and keep
the five meaningful ones written out above it.

Expose `real_catalog` from the support module as `real_catalog_public` (or make
`real_catalog` itself `pub`) so this test can reuse it. The `eprintln!` line is not
decoration: that count is the measurement the spec rests on, and it must be visible in
every run.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p graph -- --nocapture`
Expected: FAIL on the new files (missing `support` module, unimplemented cross-check), and the skips visible on stderr.

- [ ] **Step 3: Implement the support module and the cross-check body**

As described above. Nothing new in `src/` — if a test needs a new public function, add it and note that in the report.

- [ ] **Step 4: Run the whole suite**

Run: `cargo test --workspace -- --nocapture 2>&1 | rg "^(sample|skip):" | sort | uniq -c`
Expected: every real-data test in `graph` prints either a `sample:` or a `skip:` line. A test that prints neither is not running on real data and must say so.

Then: `pnpm check`
Expected: all seven commands green.

- [ ] **Step 5: Commit**

```bash
git add crates/graph/tests
git commit -m "test(graph): invariants, historical series and the wiki cross-check"
```

---

### Task 12: Close the milestone in the documents

**Files:**
- Modify: `docs/STATUS.md`
- Modify: `docs/BACKLOG.md` (entry B4)
- Modify: `CLAUDE.md` (the `graph` row of the modules table)
- Create: `docs/superpowers/reports/2026-09-07-unlock-graph-report.md`

- [ ] **Step 1: Write the report**

The report says what execution found that the plan didn't know. At minimum it must answer, with numbers: how many targets needed which verdict; which gates were left uncurated and why; how many challenges hit the disjunction case; how many cycles the real graph has; the final edge and unknown counts; and what `pnpm check` reported.

- [ ] **Step 2: Update the state**

In `docs/STATUS.md`: tick M2, add the session entry, and move `GraphInfo::Stub` out of the "open" list. In `docs/BACKLOG.md`: close B4 with a pointer to the report. In `CLAUDE.md`: the `graph` row stops saying "Doesn't exist yet (M2)" and says what the crate does, and the `ipc` row loses the `{ kind: "stub" }` sentence.

- [ ] **Step 3: Verify**

Run: `pnpm check`
Expected: green.

- [ ] **Step 4: Commit**

```bash
git add docs CLAUDE.md
git commit -m "docs: close M2, the unlock graph"
```
