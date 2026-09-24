# C1 · A fieldless enum is never tagged — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task (inline, per the owner's standing preference). Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** a test that reads the generated IPC contract and fails on any union whose every member is a bare `{ kind }`, landed together with the fix for the two enums it finds today.

**Architecture:** a pure recognizer in `ipc::contract` over the raw `render()` output (the same
text the generator writes before prettier), pinned by fixture tests; one contract test runs it
on the real contract. `WantDiagnostic` and `ChallengesDiagnostic` lose `tag = "kind"`, the
contract regenerates into `const` pairs, and the UI moves from `d.kind` to the value.

**Tech Stack:** Rust (`ts-rs`, `serde`), Vue 3 + TypeScript, Vitest.

**Spec:** `docs/superpowers/specs/2026-09-24-rules-become-checks-design.md` §3 C1, branch 1 of §2.

## Global Constraints

- Work in `C:\Projects\isaac-dome-checks` on `feature/fieldless-enums`; the main worktree stays on `develop` and only takes the merge.
- A check lands with the fix for what it finds; `EXEMPTIONS`-style lists hold no postponed debt.
- The check is seen failing on the real repository before the violations are closed, and what it listed goes on card #81 as a comment.
- `types.ts` is never edited by hand: `pnpm ipc:types`.
- Frontend rule 5: compare against the `const` (`WantDiagnostic.NothingUnlocks`), never a literal, except a tagged union's tag.
- Commits: Conventional Commits, English, no `Co-Authored-By`, staged by explicit path.

## Review Focus

- A fieldless enum whose variants carry doc comments: `ts-rs` must still render it on one line, or the line-based recognizer misses it. Pinned: `WantDiagnostic` has a doc on `NothingUnlocks` and today renders on one line (measured 2026-09-24); the render test would go silent, not red, if that changed — the fixture for a multi-line declaration in Task 1 states the limit.
- A union with a nested `|` inside a data member (`iconUrl: string | null`): must not be flagged. Fixture in Task 1.
- A struct variant with no fields (`Greedier {}` renders `{ "kind": "greedier", }`): carries nothing, so it counts as bare. Fixture in Task 1.
- A single-variant tagged enum: flagged like any other. Fixture in Task 1.
- A UI site still reading `.kind` on the now-bare value: `pnpm typecheck` refuses it (`Property 'kind' does not exist on type string`), so Task 2's typecheck step is the net.

---

### Task 1: The recognizer, and its fixtures

**Files:**
- Modify: `crates/ipc/src/contract.rs` (after `rewrite_line`, before `use ts_rs::…`)
- Test: `crates/ipc/tests/contract.rs`

**Interfaces:**
- Produces: `pub fn tagged_fieldless_unions(file: &str) -> Vec<&str>` — the names, in file order.

- [ ] **Step 0: Put `samples/` into the worktree**

The suite on real data skips silently without it (CLAUDE.md, *second worktree*).

The worktree's `samples/` holds only the tracked `.gitkeep`. Replace the empty folder with a
junction to the main worktree's, which holds the same `.gitkeep` plus the git-ignored saves:

```powershell
$s = 'C:\Projects\isaac-dome-checks\samples'
Remove-Item "$s\.gitkeep"
[System.IO.Directory]::Delete($s, $false)
New-Item -ItemType Junction -Path $s -Target C:\Projects\isaac-dome\samples
(Get-ChildItem C:\Projects\isaac-dome\samples -Recurse -File | Measure-Object).Count   # note it
git -C C:\Projects\isaac-dome-checks status --short
```

Expected: `git status` clean, `samples\packed` resolves. **Before the worktree is ever removed,
the junction goes first** with `[System.IO.Directory]::Delete($s, $false)`, and the count above
is checked again on the target.

- [ ] **Step 1: Write the failing fixture tests**

Append to `crates/ipc/tests/contract.rs` (and add `tagged_fieldless_unions` to its `use ipc::contract::{…}` line):

```rust
#[test]
fn a_union_of_bare_tags_is_named() {
    let file = r#"export type WantDiagnostic = { "kind": "noCatalog" } | { "kind": "noProfile" };"#;

    assert_eq!(tagged_fieldless_unions(file), vec!["WantDiagnostic"]);
}

#[test]
fn one_member_with_data_makes_the_tag_legitimate() {
    let file = r#"export type SaveReason = { "kind": "tooShort" } | { "kind": "io", reason: IoReason, };"#;

    assert!(tagged_fieldless_unions(file).is_empty());
}

#[test]
fn an_empty_struct_variant_carries_nothing_and_counts_as_bare() {
    let file = r#"export type Drawn = { "kind": "greedier", } | { "kind": "other" };"#;

    assert_eq!(tagged_fieldless_unions(file), vec!["Drawn"]);
}

#[test]
fn a_single_bare_variant_is_named_too() {
    let file = r#"export type Only = { "kind": "one" };"#;

    assert_eq!(tagged_fieldless_unions(file), vec!["Only"]);
}

#[test]
fn a_pipe_inside_a_field_is_not_a_member_boundary_that_hides_data() {
    let file = r#"export type Lock = { "kind": "free" } | { "kind": "unlocked", text: string | null, };"#;

    assert!(tagged_fieldless_unions(file).is_empty());
}

#[test]
fn string_unions_and_the_const_pair_are_not_tagged() {
    let file = r#"export type MissingReason = "steamNotFound" | "noSaves";
export const X = {
  A: "a",
} as const;
export type X = (typeof X)[keyof typeof X];"#;

    assert!(tagged_fieldless_unions(file).is_empty());
}

#[test]
fn a_declaration_split_across_lines_is_not_read() {
    // The limit, stated: the recognizer reads one line per declaration, as `ts-rs` writes a
    // union whose members carry no field docs. A multi-line union has fields, so this is not a
    // hole today — the test exists so that the day it is, the reason is already written here.
    let file = "export type Split = { \"kind\": \"a\" }\n  | { \"kind\": \"b\" };";

    assert!(tagged_fieldless_unions(file).is_empty());
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p ipc --test contract`
Expected: FAIL to compile — `cannot find function tagged_fieldless_unions in module ipc::contract`.

- [ ] **Step 3: Implement**

In `crates/ipc/src/contract.rs`, after `rewrite_line`:

```rust
/// The unions whose every member is a bare tag, `{ "kind": "…" }` and nothing else.
///
/// That is a fieldless enum serialized with `tag = "kind"`, which CLAUDE.md calls a bug and
/// not a style: the value is a string, and a tag around it hides that. **By form, like
/// `to_const_enums`**, so a new one is found the day it is written and no list can forget it.
/// A union with one data-carrying member is a tagged enum by right and is not named.
///
/// Reads one line per declaration, which is how `ts-rs` writes a union without field docs;
/// a union spread over lines has fields, so it cannot be one of these today.
pub fn tagged_fieldless_unions(file: &str) -> Vec<&str> {
    file.lines().filter_map(tagged_fieldless_union).collect()
}

fn tagged_fieldless_union(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("export type ")?;
    let (name, body) = rest.split_once(" = ")?;
    let body = body.strip_suffix(';')?;
    body.split('|').all(is_bare_tag).then_some(name)
}

/// `{ "kind": "x" }`, or `{ "kind": "x", }` for a struct variant with no fields. A `|` inside a
/// field splits a member into fragments, and a fragment is never a bare tag.
fn is_bare_tag(member: &str) -> bool {
    let Some(inner) = member.trim().strip_prefix('{').and_then(|m| m.strip_suffix('}')) else {
        return false;
    };
    inner
        .trim()
        .trim_end_matches(',')
        .trim_end()
        .strip_prefix(r#""kind": ""#)
        .and_then(|v| v.strip_suffix('"'))
        .is_some_and(|v| !v.contains('"'))
}
```

- [ ] **Step 4: Run them to see them pass**

Run: `cargo test -p ipc --test contract`
Expected: PASS, the seven new tests plus the existing ones.

- [ ] **Step 5: Commit**

```bash
git add crates/ipc/src/contract.rs crates/ipc/tests/contract.rs
git commit -m "test(ipc): recognize a tagged union whose every member is a bare tag"
```

---

### Task 2: The check on the real contract, and the two enums it names

**Files:**
- Test: `crates/ipc/tests/contract_shapes.rs`, `crates/ipc/tests/want.rs`, `crates/ipc/tests/challenges.rs`
- Modify: `crates/ipc/src/want.rs:71-86`, `crates/ipc/src/challenges.rs:84-91`
- Regenerate: `ui/src/lib/ipc/types.ts`
- Modify: `ui/src/lib/diagnostics/spec.ts`, `ui/src/lib/diagnostics/search.ts`, `ui/src/lib/diagnostics/challenges.ts`, `ui/src/screens/goals/WantAnswer.vue`, `ui/src/kit/sections/app/WantAnswerSection.vue`, `ui/src/composables/useWant.test.ts`, `ui/src/lib/graph/wantBlocks.test.ts`, `ui/src/lib/ipc/fixtures/challenges.ts`, `ui/src/lib/ipc/fixtures/graph.ts`

**Interfaces:**
- Consumes: `ipc::contract::tagged_fieldless_unions(file: &str) -> Vec<&str>` (Task 1), `ipc::contract::render() -> String`.
- Produces: `entriesFromValues<D extends string>(diagnostics: D[], table: Record<D, DiagnosticRow | null>): DiagnosticEntry[]` in `ui/src/lib/diagnostics/spec.ts`; `WantDiagnostic` and `ChallengesDiagnostic` as `const` pairs in `types.ts`.

- [ ] **Step 1: Write the contract test**

Append to `crates/ipc/tests/contract_shapes.rs`:

```rust
#[test]
fn no_fieldless_enum_crosses_as_a_tagged_object() {
    // CLAUDE.md: a fieldless enum is a bare camelCase string, and a tagged unit enum is a bug.
    // Read from the generated contract, not a list of names, so a new one fails the day it is
    // written. The recognizer's own cases are in `tests/contract.rs`.
    let contract = ipc::contract::render();

    assert_eq!(
        ipc::contract::tagged_fieldless_unions(&contract),
        Vec::<&str>::new()
    );
}
```

- [ ] **Step 2: Run it and see it name the two**

Run: `cargo test -p ipc --test contract_shapes no_fieldless_enum`
Expected: FAIL with `left: ["WantDiagnostic", "ChallengesDiagnostic"]`, `right: []`.

Post the failure on card #81 (`6ab53ea8a6a97737b4d03984`) as a comment: the test name, the two names it listed, the date.

- [ ] **Step 3: Pin the wire shape the fix must produce**

In `crates/ipc/tests/want.rs`, add:

```rust
#[test]
fn a_want_diagnostic_is_a_bare_string_on_the_wire() {
    use serde_json::{json, to_value};

    assert_eq!(
        to_value(WantDiagnostic::NothingUnlocks).expect("serializes"),
        json!("nothingUnlocks")
    );
}
```

In `crates/ipc/tests/challenges.rs` (it already imports `json` and `to_value`):

```rust
#[test]
fn a_challenges_diagnostic_is_a_bare_string_on_the_wire() {
    assert_eq!(
        to_value(ChallengesDiagnostic::NoChallengesSection).expect("serializes"),
        json!("noChallengesSection")
    );
}
```

Run: `cargo test -p ipc --test want --test challenges bare_string`
Expected: FAIL, `left: Object {"kind": String("nothingUnlocks")}`.

- [ ] **Step 4: Untag the two enums**

`crates/ipc/src/want.rs`, replacing the derive and serde lines above `pub enum WantDiagnostic`:

```rust
/// Why the answer is empty or partial. Fieldless: a bare string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum WantDiagnostic {
```

`crates/ipc/src/challenges.rs`:

```rust
/// What the list could not take into account. Fieldless: a bare string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum ChallengesDiagnostic {
```

Run: `cargo test -p ipc --test want --test challenges --test contract_shapes`
Expected: PASS. If `Copy` makes clippy flag a `.clone()` somewhere, remove the clone.

- [ ] **Step 5: Regenerate the contract**

Run: `pnpm ipc:types`
Expected: `ui/src/lib/ipc/types.ts` now holds

```ts
export const WantDiagnostic = {
  NoCatalog: 'noCatalog',
  NoProfile: 'noProfile',
  NothingUnlocks: 'nothingUnlocks',
  NotUnlockable: 'notUnlockable',
} as const
```

and the same pair for `ChallengesDiagnostic` (`NoCatalog`, `NoChallengesSection`, `NoAchievementSection`, `NoWiki`). `git diff --stat ui/src/lib/ipc/types.ts` shows only those two declarations changed.

- [ ] **Step 6: One helper for value diagnostics**

`ui/src/lib/diagnostics/spec.ts`, after `entriesFrom`:

```ts
// A screen whose diagnostics carry no values receives bare strings — the Rust rule for a
// fieldless enum — and this gives them the shape `entriesFrom` reads.
export const entriesFromValues = <D extends string>(
  diagnostics: D[],
  table: Record<D, DiagnosticRow | null>,
): DiagnosticEntry[] =>
  entriesFrom(
    diagnostics.map((kind) => ({ kind })),
    table,
  )
```

`ui/src/lib/diagnostics/search.ts`: import `entriesFromValues` instead of `entriesFrom`, delete the comment "The only screen whose diagnostic is a bare string…" (no longer true), and make the export

```ts
export const searchEntries = (
  diagnostics: SearchDiagnostic[],
): DiagnosticEntry[] => entriesFromValues(diagnostics, table)
```

- [ ] **Step 7: The Challenges table reads the value**

`ui/src/lib/diagnostics/challenges.ts`, whole file:

```ts
import { ChallengesDiagnostic } from '@/lib/ipc/types'
import { entriesFromValues, Severity } from './spec'
import type { DiagnosticEntry, DiagnosticRow } from './spec'

// Without the game there is no list at all, so that one is an alarm and not a note — the
// screen is empty and has to say why. An unread section 7 takes the states away and leaves the
// rows; a missing wiki takes the conditions away and leaves both.
const table: Record<ChallengesDiagnostic, DiagnosticRow> = {
  [ChallengesDiagnostic.NoCatalog]: {
    severity: Severity.Warning,
    title: 'challenges.diagnostics.noCatalogTitle',
    body: 'challenges.diagnostics.noCatalog',
  },
  [ChallengesDiagnostic.NoChallengesSection]: {
    severity: Severity.Warning,
    title: 'challenges.diagnostics.noChallengesSectionTitle',
    body: 'challenges.diagnostics.noChallengesSection',
  },
  [ChallengesDiagnostic.NoAchievementSection]: {
    severity: Severity.Warning,
    title: 'challenges.diagnostics.noAchievementSectionTitle',
    body: 'challenges.diagnostics.noAchievementSection',
  },
  [ChallengesDiagnostic.NoWiki]: {
    severity: Severity.Note,
    body: 'challenges.diagnostics.noWiki',
  },
}

export const challengeEntries = (
  diagnostics: ChallengesDiagnostic[],
): DiagnosticEntry[] => entriesFromValues(diagnostics, table)
```

`ui/src/lib/ipc/fixtures/challenges.ts:110`: `diagnostics: [ChallengesDiagnostic.NoChallengesSection],`, and add `ChallengesDiagnostic` to the value import on line 1: `import { ChallengesDiagnostic, Style } from '../types'`.

- [ ] **Step 8: The Want banner reads the value**

`ui/src/screens/goals/WantAnswer.vue`: change `import type { WantDiagnostic }` to `import { WantDiagnostic } from '@/lib/ipc/types'`, and

```ts
const bannerText: Record<WantDiagnostic, Message> = {
  [WantDiagnostic.NoCatalog]: 'want.diagnostics.noCatalog',
  [WantDiagnostic.NoProfile]: 'want.diagnostics.noProfile',
  [WantDiagnostic.NothingUnlocks]: 'want.diagnostics.nothingUnlocks',
  [WantDiagnostic.NotUnlockable]: 'want.diagnostics.notUnlockable',
}
```

and in the template `{{ t(bannerText[banner]) }}`.

`ui/src/kit/sections/app/WantAnswerSection.vue`: add `import { WantDiagnostic } from '@/lib/ipc/types'` and bind `:banner="WantDiagnostic.NothingUnlocks"`.

`ui/src/lib/ipc/fixtures/graph.ts:262`: `diagnostics: [WantDiagnostic.NothingUnlocks],`, with `WantDiagnostic` imported as a value from `'../types'` (a separate `import { WantDiagnostic } from '../types'` line beside the existing `import type`).

`ui/src/composables/useWant.test.ts:10`: `diagnostics: [WantDiagnostic.NothingUnlocks],`, importing `WantDiagnostic` from `@/lib/ipc/types`.

`ui/src/lib/graph/wantBlocks.test.ts:86-88`:

```ts
        diagnostics: [WantDiagnostic.NothingUnlocks],
      }),
    ).toBe(WantDiagnostic.NothingUnlocks)
```

importing `WantDiagnostic` from `@/lib/ipc/types`.

- [ ] **Step 9: Let the compiler find what the grep missed**

Run: `pnpm typecheck; pnpm ui:test; pnpm lint; pnpm scan`
Expected: all green. A `Property 'kind' does not exist on type` error names a site this plan missed: fix it the same way (value, not `.kind`).

- [ ] **Step 10: Commit the check with its fix**

```bash
git add crates/ipc/tests/contract_shapes.rs crates/ipc/tests/want.rs crates/ipc/tests/challenges.rs \
  crates/ipc/src/want.rs crates/ipc/src/challenges.rs ui/src/lib/ipc/types.ts \
  ui/src/lib/diagnostics/spec.ts ui/src/lib/diagnostics/search.ts ui/src/lib/diagnostics/challenges.ts \
  ui/src/screens/goals/WantAnswer.vue ui/src/kit/sections/app/WantAnswerSection.vue \
  ui/src/composables/useWant.test.ts ui/src/lib/graph/wantBlocks.test.ts \
  ui/src/lib/ipc/fixtures/challenges.ts ui/src/lib/ipc/fixtures/graph.ts
git commit -m "fix(ipc): no fieldless enum crosses as a tagged object, and a test that keeps it so"
```

(plus any file Step 9 added, by path).

---

### Task 3: Say what checks the rule, then merge

**Files:**
- Modify: `CLAUDE.md` (the "One of our fieldless enums isn't tagged" bullet, around line 170-181)

- [ ] **Step 1: CLAUDE.md names the check**

In the bullet, add `WantDiagnostic`, `ChallengesDiagnostic`, `SearchDiagnostic` to the zero-exceptions list, and after "A tagged unit enum showing up again is a bug, not an alternative style — …" append:

> **Checked since 2026-09-24** (card #81, C1) by `no_fieldless_enum_crosses_as_a_tagged_object` in
> `crates/ipc/tests/contract_shapes.rs`, which reads the generated contract: two had come back,
> `WantDiagnostic` and `ChallengesDiagnostic`, and nothing had noticed.

- [ ] **Step 2: Full check**

Run: `pnpm check`
Expected: `all green`; `real files touched` non-zero (the junction works); `test count` passes with more tests than the merge base.

- [ ] **Step 3: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: the fieldless-enum rule names the test that checks it"
```

- [ ] **Step 4: Merge from the main worktree**

```bash
git -C C:/Projects/isaac-dome branch --show-current        # expect develop
git -C C:/Projects/isaac-dome pull --ff-only
git -C C:/Projects/isaac-dome merge --no-ff feature/fieldless-enums -m "merge: no fieldless enum crosses as a tagged object, into develop"
git -C C:/Projects/isaac-dome push origin develop
```

Then park the worktree for the next branch and close this one:

```bash
git -C C:/Projects/isaac-dome-checks switch --detach develop
git -C C:/Projects/isaac-dome rev-list --count develop..feature/fieldless-enums   # expect 0
git -C C:/Projects/isaac-dome branch -d feature/fieldless-enums
```

- [ ] **Step 5: The card**

Tick C1 on card #81; comment with the merge hash. The card stays `In Progress` — seven branches remain.
