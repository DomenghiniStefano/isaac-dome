# A contract that is generated, not mirrored — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `ui/src/lib/ipc/types.ts` stops being written by hand: a binary in `crates/ipc`
emits its 865 lines from the Rust types themselves, and `scripts/check` goes red when the two
disagree.

**Architecture:** `ts-rs` derives on the types that cross the boundary, in `ipc` and in the
four crates that own the five foreign ones. The logic lives in `crates/ipc/src/contract.rs` —
an explicit ordered list of the types, `render()` which assembles the file from `TS::docs()`
and `TS::decl()`, and `to_const_enums()` which rewrites the one shape `ts-rs` cannot spell the
way frontend rule 5 demands. `crates/ipc/src/bin/ipc-types.rs` is thin: paths, I/O and exit
codes, exactly like `crates/graph/src/bin/graph-rules.rs`, whose logic likewise lives in
`graph::generate`.

**Tech Stack:** Rust, `ts-rs` 12.0.1 (default features, i.e. `serde-compat`), prettier
(`semi: false`, `singleQuote: true`), pnpm.

**Spec:** `docs/superpowers/specs/2026-09-13-generated-contract-design.md`

## Global Constraints

- **`ts-rs` is an ordinary dependency**, not behind a feature gate, in `ipc`, `core-save`,
  `wiki` and `discovery`. The spec's §2 argues it; do not reintroduce the gate.
- **The binary sets `Config::new().with_large_int("number")`.** JSON has no bigint:
  `serde_json` writes a number for `i64`, so `bigint` would be a lie about the wire.
- **No string unions survive in `types.ts`.** Frontend rule 5, enforced by `pnpm scan`. The
  rewriter works **by form** — every union of string literals — never from a list of names.
- **A fieldless enum is a bare camelCase string on the wire**, a tagged union otherwise. This
  is already true of the Rust types; generation must not change any of it.
- **Doc comments move, they do not get rewritten.** A comment in `types.ts` that carries a
  fact becomes a `///` on the Rust type, verbatim where the wording still fits.
- **Never `git add -A`.** Stage by explicit path; other sessions edit `docs/superpowers/`.
- Commits: Conventional Commits, `type(scope): subject`, English, atomic. No `Co-Authored-By`.

---

### Task 1: the rewriter, before anything generates

**Files:**
- Create: `crates/ipc/src/contract.rs`
- Modify: `crates/ipc/src/lib.rs`
- Test: `crates/ipc/tests/contract.rs`

**Interfaces:**
- Produces: `ipc::contract::to_const_enums(file: &str) -> String` and
  `ipc::contract::pascal_case(value: &str) -> String`. Task 5 calls the first one on the
  whole assembled file.

Why this comes first: it is pure string work with no dependency on any derive, so it is the
one part of the job that can be driven entirely by tests before `ts-rs` exists in the repo.

The input it must handle is `ts-rs`'s raw spelling — double quotes, a trailing semicolon —
because it runs **before** prettier. Verified in the vendored source: `generate_decl`
(`ts-rs-12.0.1/src/export.rs:314-322`) writes the docs, then the literal `"export "`, then
`TS::decl()`.

- [ ] **Step 1: Write the failing tests**

`crates/ipc/tests/contract.rs`:

```rust
//! The one shape `ts-rs` cannot spell the way frontend rule 5 demands.
//!
//! The input is the generator's raw output — double quotes, trailing semicolon — because the
//! rewriter runs before prettier does.

use ipc::contract::{pascal_case, to_const_enums};

#[test]
fn a_union_of_string_literals_becomes_the_const_pair() {
    let input = r#"export type MissingReason = "steamNotFound" | "gameNotFound" | "noSaves";"#;

    let expected = r#"export const MissingReason = {
  SteamNotFound: "steamNotFound",
  GameNotFound: "gameNotFound",
  NoSaves: "noSaves",
} as const;
export type MissingReason = (typeof MissingReason)[keyof typeof MissingReason];"#;

    assert_eq!(to_const_enums(input), expected);
}

#[test]
fn a_snake_case_value_still_yields_a_pascal_case_key() {
    // `SavePrefix` comes from a domain crate and keeps its own snake_case on the wire.
    let input = r#"export type SavePrefix = "rep" | "rep_plus";"#;

    assert!(to_const_enums(input).contains(r#"RepPlus: "rep_plus","#));
}

#[test]
fn a_tagged_union_is_left_alone() {
    // Not a union of string literals: every member is an object. Rule 5 is about values.
    let input = r#"export type SaveDiagnostic = { "kind": "trailingBytes" } | { "kind": "sectionOverrun", "section": number };"#;

    assert_eq!(to_const_enums(input), input);
}

#[test]
fn a_union_with_one_non_string_member_is_left_alone() {
    let input = r#"export type Weird = "a" | number;"#;

    assert_eq!(to_const_enums(input), input);
}

#[test]
fn an_interface_is_left_alone() {
    let input = "export interface Goal { id: GoalId, note: string | null, }";

    assert_eq!(to_const_enums(input), input);
}

#[test]
fn the_docs_above_a_rewritten_union_survive_it() {
    let input = r#"/**
 * Why we could not find a save.
 */
export type MissingReason = "steamNotFound" | "noSaves";"#;

    let out = to_const_enums(input);
    assert!(out.starts_with("/**\n * Why we could not find a save.\n */\n"));
    assert!(out.contains("export const MissingReason = {"));
}

#[test]
fn every_declaration_in_a_whole_file_is_visited() {
    let input = r#"export type A = "one" | "two";

export interface B { a: A, }

export type C = "three";"#;

    let out = to_const_enums(input);
    assert!(out.contains("export const A = {"));
    assert!(out.contains("export const C = {"));
    assert!(out.contains("export interface B { a: A, }"));
}

#[test]
fn pascal_case_reads_both_spellings() {
    assert_eq!(pascal_case("steamNotFound"), "SteamNotFound");
    assert_eq!(pascal_case("rep_plus"), "RepPlus");
    assert_eq!(pascal_case("rep"), "Rep");
    assert_eq!(pascal_case("championVersions"), "ChampionVersions");
}
```

- [ ] **Step 2: Run them and watch them fail**

Run: `cargo test -p ipc --test contract`
Expected: FAIL — `unresolved import ipc::contract`.

- [ ] **Step 3: Write the module**

`crates/ipc/src/contract.rs`:

```rust
//! The TypeScript contract, assembled from the types that cross the boundary.
//!
//! `ts-rs` renders a fieldless enum as a union of string literals. Frontend rule 5 forbids
//! literal unions — every component compares against a symbol (`MissingReason.SteamNotFound`)
//! — so each one is rewritten into the `const … as const` pair the repo already spells by
//! hand. **By form and not by a list of names**: rule 5 admits no exception, so there is no
//! union that should survive and nothing for a list to forget.

/// `steamNotFound` → `SteamNotFound`, `rep_plus` → `RepPlus`.
pub fn pascal_case(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut upper = true;
    for c in value.chars() {
        if c == '_' {
            upper = true;
        } else if upper {
            out.extend(c.to_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}

/// The string literals of a union, or `None` if any member is something else.
fn string_members(body: &str) -> Option<Vec<&str>> {
    let mut members = Vec::new();
    for part in body.split('|') {
        let part = part.trim();
        let inner = part.strip_prefix('"')?.strip_suffix('"')?;
        if inner.contains('"') {
            return None;
        }
        members.push(inner);
    }
    (!members.is_empty()).then_some(members)
}

/// Rewrite every union of string literals in `file` into the `const … as const` pair.
pub fn to_const_enums(file: &str) -> String {
    let mut out = String::with_capacity(file.len());
    for (i, line) in file.lines().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        match rewrite_line(line) {
            Some(pair) => out.push_str(&pair),
            None => out.push_str(line),
        }
    }
    out
}

fn rewrite_line(line: &str) -> Option<String> {
    let rest = line.strip_prefix("export type ")?;
    let (name, body) = rest.split_once(" = ")?;
    let body = body.strip_suffix(';')?;
    let members = string_members(body)?;

    let mut pair = format!("export const {name} = {{\n");
    for m in &members {
        pair.push_str(&format!("  {}: \"{m}\",\n", pascal_case(m)));
    }
    pair.push_str("} as const;\n");
    pair.push_str(&format!(
        "export type {name} = (typeof {name})[keyof typeof {name}];"
    ));
    Some(pair)
}
```

`crates/ipc/src/lib.rs`: add `pub mod contract;` beside the other modules.

**Note for the implementer:** `rewrite_line` works a line at a time, which is why the test
`every_declaration_in_a_whole_file_is_visited` exists. `ts-rs` emits each declaration on one
line; if that ever stops being true, that test is the one that will say so.

- [ ] **Step 4: Run them and watch them pass**

Run: `cargo test -p ipc --test contract`
Expected: 8 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/ipc/src/contract.rs crates/ipc/src/lib.rs crates/ipc/tests/contract.rs
git commit -m "feat(ipc): the one shape a generator cannot spell"
```

---

### Task 2: `ts-rs` on the five types that come from elsewhere

**Files:**
- Modify: `crates/ipc/Cargo.toml`, `crates/core-save/Cargo.toml`, `crates/wiki/Cargo.toml`,
  `crates/discovery/Cargo.toml`
- Modify: `crates/core-save/src/section.rs:18`, `crates/wiki/src/model.rs:147`,
  `crates/discovery/src/lib.rs:61`, `:71`, `:98`
- Test: `crates/ipc/tests/contract_foreign.rs`

**Interfaces:**
- Produces: `impl TS` for `core_save::Kind`, `wiki::Target`, `discovery::Edition`,
  `discovery::Dlc`, `discovery::SavePrefix`. Task 5's list names all five.

These five are the reason the item exists. `CLAUDE.md` records `core_save::Kind` crossing the
boundary inside `ipc::SectionCount` while the TypeScript mirror types that field as `string` —
a rename changed the wire with the whole suite green. Declaring them by hand in the rewriter
would leave exactly that possible.

- [ ] **Step 1: Add the dependency to the four manifests**

In each of `crates/ipc/Cargo.toml`, `crates/core-save/Cargo.toml`, `crates/wiki/Cargo.toml`,
`crates/discovery/Cargo.toml`, under `[dependencies]`:

```toml
ts-rs = "12.0.1"
```

Default features on purpose: `serde-compat` is one of them
(`ts-rs-12.0.1/Cargo.toml:48`), and it is what makes `#[serde(rename_all)]`, `rename_all_fields`,
`tag`, `rename` and `transparent` the generator's input instead of something restated.

- [ ] **Step 2: Write the failing test**

`crates/ipc/tests/contract_foreign.rs`:

```rust
//! The five types that cross the boundary from crates that are not `ipc`.
//!
//! `core_save::Kind` is the documented incident: it travels inside `ipc::SectionCount`, the
//! hand-written mirror typed it as `string`, and renaming a variant changed the wire with the
//! whole suite green. These assertions are what makes a rename fail the build instead.

use ts_rs::{Config, TS};

fn cfg() -> Config {
    Config::new().with_large_int("number")
}

#[test]
fn the_sections_of_the_save_are_a_closed_set_of_names() {
    assert_eq!(
        <core_save::Kind as TS>::decl(&cfg()),
        r#"type Kind = "achievements" | "counters" | "level_counters" | "items" | "unknown5" | "bosses" | "challenges" | "unknown8" | "unknown9" | "bestiary";"#
    );
}

#[test]
fn the_two_save_prefixes_keep_their_own_spelling() {
    assert_eq!(
        <discovery::SavePrefix as TS>::decl(&cfg()),
        r#"type SavePrefix = "rep" | "rep_plus";"#
    );
}
```

**Where the expected values come from:** `Kind` is `#[serde(rename_all = "snake_case")]` over
the ten variants at `crates/core-save/src/section.rs:18-29`, in file order. `SavePrefix`'s two
values are what `ui/src/lib/ipc/types.ts:14` spells today — the live contract.

- [ ] **Step 3: Run it and watch it fail**

Run: `cargo test -p ipc --test contract_foreign`
Expected: FAIL — `Kind: TS` is not satisfied.

- [ ] **Step 4: Add the derives**

Five edits, each adding `TS` to the existing derive list and nothing else. `core_save::Kind`
(`section.rs:18`) becomes:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
```

Do the same for `wiki::Target` (`crates/wiki/src/model.rs:147`) and for `Edition`, `Dlc` and
`SavePrefix` (`crates/discovery/src/lib.rs:61`, `:71`, `:98`). **Do not add `#[ts(export)]`
to any of them** — nothing in this repo exports through `cargo test`; Task 5's binary is the
only writer.

- [ ] **Step 5: Run it and watch it pass**

Run: `cargo test -p ipc --test contract_foreign`
Expected: 2 passed.

If `Kind`'s assertion fails on the *shape* rather than the names — for instance `ts-rs`
disagreeing about where `rename_all` applies — stop and read the difference before adjusting
the expectation. The expectation came from the serde attribute, and serde is what writes the
wire; a disagreement is a finding, not a number to update.

- [ ] **Step 6: Commit**

```bash
git add crates/ipc/Cargo.toml crates/core-save/Cargo.toml crates/wiki/Cargo.toml crates/discovery/Cargo.toml crates/core-save/src/section.rs crates/wiki/src/model.rs crates/discovery/src/lib.rs crates/ipc/tests/contract_foreign.rs Cargo.lock
git commit -m "feat(ipc): the five foreign wire types declare themselves"
```

---

### Task 3: the derives across `ipc`, and the delicate five

**Files:**
- Modify: every file under `crates/ipc/src/` that declares a type crossing the boundary —
  `catalog_view.rs`, `collection.rs`, `error.rs`, `goals.rs`, `graph.rs`, `icon.rs`,
  `mark_art.rs`, `marks.rs`, `profile.rs`, `progress.rs`, `queue.rs`, `reasons.rs`,
  `resources.rs`, `search.rs`, `settings.rs`, `summary.rs`, `target_sprite.rs`, `tray.rs`,
  `want.rs`, `wiki.rs`
- Test: `crates/ipc/tests/contract_shapes.rs`

**Interfaces:**
- Produces: `impl TS` for the types Task 5's list names. Nothing else changes.

`ipc` has 92 public types and the contract exports 81: the difference is types that never
cross (`SearchIndex`, `Doc`, the lifetime-bearing `QueueInputs<'a>`, `SaveFlags<'a>`,
`SaveProgress<'a>`, `TargetSprite<'a>`). **Derive `TS` only on what crosses.** A type that
takes a lifetime cannot derive it anyway, which is a useful accident: the compiler refuses
exactly the ones that are not view-models.

- [ ] **Step 1: Write the failing test for the five delicate cases**

These are the five the spike singled out. `crates/ipc/tests/contract_shapes.rs`:

```rust
//! The five cases the B2 spike singled out, pinned on the real types rather than on copies.

use ts_rs::{Config, TS};

fn cfg() -> Config {
    Config::new().with_large_int("number")
}

#[test]
fn the_fields_of_a_struct_variant_are_camel_case_too() {
    // `rename_all` on an enum renames the variants; the fields inside them need
    // `rename_all_fields`. Forgetting it is silent — TypeScript reads `undefined`.
    assert!(<ipc::SetupState as TS>::decl(&cfg()).contains("autoSelected"));
    assert!(!<ipc::SetupState as TS>::decl(&cfg()).contains("auto_selected"));
}

#[test]
fn a_serde_rename_is_read_and_not_guessed() {
    // B9's rule in its original form: `miniZ` is not a typo and no generator may tidy it.
    assert!(<ipc::ArchiveMode as TS>::decl(&cfg()).contains(r#""miniZ""#));
}

#[test]
fn a_transparent_newtype_is_its_inner_type() {
    assert_eq!(<ipc::GoalId as TS>::decl(&cfg()), "type GoalId = string;");
}

#[test]
fn a_unix_timestamp_is_a_number_and_never_a_bigint() {
    // `i64` takes the configured large-int type, whose default is `bigint`. JSON has no
    // bigint: `serde_json` writes a number, so `bigint` would be a lie about the wire.
    assert!(<ipc::Goal as TS>::decl(&cfg()).contains("createdUnix: number"));
}

#[test]
fn the_section_kind_stops_being_a_bare_string() {
    // The documented incident, from the other side: the hand-written mirror typed this
    // `string`, which is why a renamed variant changed the wire with the suite green.
    assert!(<ipc::SectionCount as TS>::decl(&cfg()).contains("kind: Kind"));
}
```

- [ ] **Step 2: Run it and watch it fail**

Run: `cargo test -p ipc --test contract_shapes`
Expected: FAIL — the trait is not implemented for these types yet.

- [ ] **Step 3: Add the derives, file by file**

Work through the twenty files in the order listed above. In each, for every type that crosses
the boundary, add `ts_rs::TS` to the existing `#[derive(...)]`. Change nothing else — not a
field, not an attribute, not a name.

Two rules while doing it, both of which the compiler will enforce for you:

- a type whose fields reference another type on the boundary needs that one to derive `TS`
  as well, so the set closes on itself;
- `#[serde(skip)]` and `#[serde(flatten)]` are read by `serde-compat` and need no help.

Run `cargo check -p ipc` after each file rather than at the end: an error names the one type
you just touched instead of twenty.

- [ ] **Step 4: Run it and watch it pass**

Run: `cargo test -p ipc --test contract_shapes`
Expected: 5 passed.

- [ ] **Step 5: Run the whole suite — nothing about the wire may move**

Run: `cargo test --workspace`
Expected: green, and in particular `crates/ipc/tests/summary_shape.rs` and
`crates/ipc/tests/reasons.rs` unchanged and passing. They are the control: they assert what
the JSON looks like, and a derive that changed serialization would break them. If one goes
red, the derive changed the wire — that is a bug in this task, not an expectation to update.

- [ ] **Step 6: Commit**

```bash
git add crates/ipc/src crates/ipc/tests/contract_shapes.rs
git commit -m "feat(ipc): the view-models declare their own TypeScript"
```

---

### Task 4: the comments move to the types

**Files:**
- Modify: the same twenty files under `crates/ipc/src/`, plus `crates/core-save/src/section.rs`,
  `crates/wiki/src/model.rs`, `crates/discovery/src/lib.rs`
- Read: `ui/src/lib/ipc/types.ts` (the source of the prose)

**Interfaces:**
- Consumes: the derives of Tasks 2 and 3.
- Produces: `TS::docs()` non-empty for every type whose TypeScript mirror carries a comment
  today. Task 5's reconciliation depends on this being finished, or every one of those
  comments shows up as a difference.

`ts-rs` turns `///` into a JSDoc block: `TS::docs()` at `ts-rs-12.0.1/src/lib.rs:398`, emitted
before the declaration at `export.rs:314-322`. This is the task that turns the item from a
deduplication into a move — after it, the knowledge lives once, next to what it describes.

- [ ] **Step 1: Take an inventory**

Read `ui/src/lib/ipc/types.ts` top to bottom and sort every comment into two piles:

- **carries a fact** — "`miniZ` is not a typo", "B37 — a want, read from the other end of the
  graph", "The same identity as the UI shows it: name and icon resolved now", the note on
  `IoReason` explaining why a reason built with `format!` is not translatable. These move.
- **navigates the mirror** — "Mirrors `crates/ipc/src/reasons.rs`", "Mirrors
  `crates/app/src/error.rs`". These die with the file: after this work there is no mirror to
  point at.

- [ ] **Step 2: Move the first pile**

For each, put the text on the Rust type as `///`, verbatim where the wording still fits. Where
a sentence was written to a TypeScript reader ("the tag is `kind` in camelCase"), rewrite it
for a reader of the Rust type — the fact is what moves, not the sentence.

A type that already has a Rust doc comment keeps it, and gains the TypeScript one only if it
says something the Rust one does not. Two comments saying the same thing in one place is the
defect this whole item is about, at a smaller scale.

- [ ] **Step 3: Check nothing else moved**

Run: `cargo test --workspace`
Expected: green. Doc comments cannot change behaviour; a red suite here means something other
than a comment was edited.

- [ ] **Step 4: Commit**

```bash
git add crates/ipc/src crates/core-save/src/section.rs crates/wiki/src/model.rs crates/discovery/src/lib.rs
git commit -m "docs(ipc): the contract's knowledge moves next to the types"
```

---

### Task 5: the list, the binary, and the diff that has to reach zero

**Files:**
- Modify: `crates/ipc/src/contract.rs`
- Create: `crates/ipc/src/bin/ipc-types.rs`
- Modify: `package.json`
- Modify: `ui/src/lib/ipc/types.ts` (regenerated, not edited)

**Interfaces:**
- Consumes: `to_const_enums` (Task 1), the derives (Tasks 2 and 3), the docs (Task 4).
- Produces: `ipc::contract::render() -> String` and the command `pnpm ipc:types`.

- [ ] **Step 1: Write `render()`**

Append to `crates/ipc/src/contract.rs`:

```rust
use ts_rs::{Config, TS};

/// The header of the generated file. It names the command rather than describing the
/// situation: whoever opens this file is about to edit it by hand.
const HEADER: &str = "\
// Generated by `pnpm ipc:types` from the Rust types. Do not edit.
//
// The contract between Rust and Vue lives in the `#[serde]` attributes of `crates/ipc` and
// of the four types that cross from `core-save`, `wiki` and `discovery`. `scripts/check`
// fails when this file and those types disagree.
";

/// One declaration: its JSDoc, then `export`, then the type itself.
fn decl<T: TS + ?Sized + 'static>(cfg: &Config, out: &mut String) {
    if let Some(docs) = <T as TS>::docs() {
        out.push_str(&docs);
    }
    out.push_str("export ");
    out.push_str(&<T as TS>::decl(cfg));
    out.push_str("\n\n");
}

/// The whole contract, in the order of the file.
///
/// The list **is** the registry of what crosses the boundary: a type that is not here does
/// not travel, and one that is reachable only as another's field cannot arrive by accident.
/// An incomplete list produces a file that does not compile, and `pnpm typecheck` is already
/// in `scripts/check`.
pub fn render() -> String {
    let cfg = Config::new().with_large_int("number");
    let mut out = String::with_capacity(32 * 1024);
    out.push_str(HEADER);
    out.push('\n');

    decl::<crate::CandidateSource>(&cfg, &mut out);
    decl::<discovery::SavePrefix>(&cfg, &mut out);
    // … the rest of the list, in the order of the current `types.ts` …

    to_const_enums(out.trim_end()) + "\n"
}
```

**Build the list from the current `ui/src/lib/ipc/types.ts`, in its order.** That order is the
contract's order, and keeping it is what makes the first diff readable: anything that moves is
a real difference, not a reshuffle. Walk the file top to bottom, and for each `export const` /
`export interface` / `export type`, add the Rust type it mirrors.

- [ ] **Step 2: Write the binary**

`crates/ipc/src/bin/ipc-types.rs`:

```rust
//! Writes `ui/src/lib/ipc/types.ts` from the Rust types. Offline, run by hand through
//! `pnpm ipc:types`; the app never runs this.
//!
//! Thin on purpose, like `crates/graph/src/bin/graph-rules.rs`: paths, I/O and exit codes.
//! Everything with a return value worth checking is in `ipc::contract`.

use std::path::Path;

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let out = match std::env::var("ISAACDOME_TYPES_OUT") {
        Ok(p) => std::path::PathBuf::from(p),
        Err(_) => root.join("ui/src/lib/ipc/types.ts"),
    };
    if let Some(dir) = out.parent() {
        if let Err(e) = std::fs::create_dir_all(dir) {
            eprintln!("cannot create {}: {e}", dir.display());
            std::process::exit(1);
        }
    }
    if let Err(e) = std::fs::write(&out, ipc::contract::render()) {
        eprintln!("cannot write {}: {e}", out.display());
        std::process::exit(1);
    }
    println!("wrote {}", out.display());
}
```

`ISAACDOME_TYPES_OUT` exists for one caller: the staleness gate of Task 6, which must write
somewhere other than the working tree. A check that repairs what it is checking cannot report
on it.

- [ ] **Step 3: Add the command**

`package.json`, in `scripts`, beside `graph:rules`:

```json
    "ipc:types": "cargo run -q -p ipc --bin ipc-types && pnpm --filter ui exec prettier --write src/lib/ipc/types.ts",
```

prettier is not decoration: `pnpm format:check` is in `scripts/check` and the repo's settings
are `semi: false, singleQuote: true`, which is why the committed file has no semicolons. The
generator emits `ts-rs`'s spelling and prettier normalizes it.

- [ ] **Step 4: Generate, and read the diff**

Run: `pnpm ipc:types && git diff --stat ui/src/lib/ipc/types.ts`

**This is the heart of the task.** `types.ts` is the live contract, so every difference is a
change to the wire and has to be explained before it is accepted. Go through them one at a
time. Three kinds are expected:

- **`sections: { kind: string; count: number }[]` becomes `sections: SectionCount[]`, with
  `kind: Kind` a real union.** Accept it, and say so in the commit: this is the documented
  incident being closed by construction.
- **A comment that did not move in Task 4.** Not a wire change — go back and move it.
- **An ordering difference.** Fix the list, not the file.

Anything that is none of these — a field renamed, a type widened, an optional that became
required — **stop**. It means a Rust type and its hand-written mirror had already drifted, and
which of the two is right is a question for the owner, not a diff to accept quietly. Write down
what you found.

- [ ] **Step 5: Prove the generated file is the one the app compiles against**

Run: `pnpm typecheck && pnpm ui:test && pnpm lint && pnpm scan`
Expected: green. `pnpm scan` is the one that would catch a string union the rewriter missed;
`pnpm typecheck` is the one that would catch a type missing from the list.

- [ ] **Step 6: Commit**

```bash
git add crates/ipc/src/contract.rs crates/ipc/src/bin/ipc-types.rs package.json ui/src/lib/ipc/types.ts
git commit -m "feat(ipc): the contract is generated from the types it is about"
```

---

### Task 6: the gate

**Files:**
- Modify: `scripts/check`

**Interfaces:**
- Consumes: `pnpm ipc:types` and `ISAACDOME_TYPES_OUT` (Task 5).

- [ ] **Step 1: Add the check**

In `scripts/check`, after `run 'pnpm scan' pnpm scan` and before the summary:

```sh
# The contract is generated (B2/N7): this is what makes a Rust type and its TypeScript
# disagree loudly. It regenerates into a temp file and compares — it never writes into the
# working tree, because a check that repairs what it is checking cannot report on it.
printf '\n=== ipc types\n'
generated=$(mktemp)
if ISAACDOME_TYPES_OUT="$generated" cargo run -q -p ipc --bin ipc-types >/dev/null &&
	pnpm --filter ui exec prettier --write "$generated" >/dev/null 2>&1 &&
	diff -u ui/src/lib/ipc/types.ts "$generated"; then
	printf 'types.ts matches the Rust types\n'
else
	printf 'types.ts is stale: run `pnpm ipc:types`\n'
	failed="$failed ipc-types"
fi
rm -f "$generated"
```

Add `"$generated"` to the `trap` line's `rm -f` alongside `"$output"` and `"$declarations"`,
so an interrupted run leaves nothing behind.

**Note:** prettier needs the file to end in `.ts` to pick its TypeScript parser. `mktemp`
gives a name without an extension, so use `generated=$(mktemp --suffix=.ts)` — and if the
platform's `mktemp` rejects `--suffix`, fall back to
`generated=$(mktemp -d)/types.ts` and remove the directory instead.

- [ ] **Step 2: Prove the gate can speak before trusting its silence**

Edit a variant name in `crates/core-save/src/section.rs` — `Bestiary` to `Bestiaryy` — and run
`sh scripts/check`.
Expected: **two** failures, `cargo-test` (the pin from Task 2) and `ipc-types`. An instrument
that reports nothing proves nothing until it has been shown able to speak.

Undo the rename.

- [ ] **Step 3: Run the whole thing**

Run: `pnpm check`
Expected: every command green, and the skip count the same as before this work started. Read
the skip lines rather than the total.

- [ ] **Step 4: Commit**

```bash
git add scripts/check
git commit -m "build: a stale contract fails the checks"
```

---

### Task 7: what the documents say now

**Files:**
- Modify: `docs/STATUS.md`, `docs/IMPROVEMENTS.md`, `CLAUDE.md`

- [ ] **Step 1: Close the items**

`docs/IMPROVEMENTS.md`: tick **B2** and write, in the style of the other closed entries, what
the work actually found — including the count of differences the reconciliation of Task 5
produced and what each one was. If any difference was a drift between a Rust type and its
mirror, that is the most valuable sentence in the entry: it is a defect that existed before
the generator and that only the generator could see.

`docs/STATUS.md`: tick **N7** in *Next up*, with the same detail, and note that the order's
next item is **M4 sub-project 1** — which was the whole argument for N7 running before it.

- [ ] **Step 2: Correct `CLAUDE.md`**

The *Code rules* section describes `types.ts` as "hand-mirrored in `ui/src/lib/ipc/types.ts`",
and the *State* section says a change to a type "has to be handed on, not merely committed"
because the mirror is hand-written. Both sentences are about a file that no longer exists in
that form. Rewrite them to say what is true: the contract is generated by `pnpm ipc:types`,
`scripts/check` fails when it is stale, and what still has to be handed on is the **change to
the contract**, which is a fact about the design and not about the file.

Leave the paragraph about `core_save::Kind` crossing inside `ipc::SectionCount` — but say that
it is now closed by construction, and by which test.

- [ ] **Step 3: Commit**

```bash
git add docs/STATUS.md docs/IMPROVEMENTS.md CLAUDE.md
git commit -m "docs: the contract is generated, and B2 closes"
```

---

## Self-Review

**Spec coverage.** §1 the binary and the list → Task 5. §2 where the derive goes → Tasks 2 and
3. §3 the fieldless-enum gap → Task 1. §4 formatting → Task 5 step 3. §5 the staleness gate →
Task 6. §6 the comments → Task 4. §7 what is tested → Tasks 1, 2, 3 (the delicate five, the
rename pin, the rewriter) and Task 6 (the file-level equality, enforced by the gate rather than
by a Rust test, because the comparison has to happen after prettier). §8 out of scope → nothing
in any task touches the shape tests, the wrappers, the command surface or the fixtures. §9 done
when → Task 6 steps 2 and 3, Task 7.

**One deliberate departure from the spec's wording.** §7's first bullet reads as though the
equality of generated and committed were a Rust test. It cannot be: the committed file is
prettier's output and a Rust test cannot run prettier. It is the gate of Task 6 that enforces
it, which is the same assertion in the only place it can hold.

**Placeholders.** The one ellipsis is in Task 5 step 1, and it is instructed rather than
left blank: the list is built by walking the current `types.ts` in order, and step 4's diff is
what proves it complete. Writing all 81 lines here would be a second copy of the very file this
plan deletes — and it would be stale by Task 4.

**Type consistency.** `to_const_enums` and `pascal_case` (Task 1) are called by `render` (Task
5) under those names. `render()` is called by the binary and by nothing else.
`ISAACDOME_TYPES_OUT` is written in Task 5 and read in Task 6. `Config::new().with_large_int("number")`
appears identically in Tasks 2, 3 and 5.
