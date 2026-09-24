# The rules CLAUDE.md states become checks

**Status:** design agreed in conversation on 2026-09-24. Card #81, first block (C1–C9). Three
decisions are the owner's and are marked as such; the rest follow from them or from what the
checks already do.

The code review of 2026-09-24 measured one thing more sharply than anything else: **what a script
checks is 100% clean, and what lives only in `CLAUDE.md` is broken in every area.** No `<style>`,
no `invoke()` in a component, no `Result<_, String>`, no `unwrap()` on disk data — all checked.
Meanwhile two fieldless enums are tagged, `_ =>` sits on closed enums, nine examples open
`samples/` by hand, seven command files `use crate::state::*`, and `lib/` imports from
`components/` under a comment that says "checked". A rule holds because something fails when it
is broken; this cycle gives nine rules that something.

---

## 1. Decisions

- **A check lands with the fix for what it finds** (owner). Each branch turns one check on, sees
  it fail on the real repository, records what it listed as a comment on card #81, closes those
  violations, and merges green. `EXEMPTIONS` holds permanent exceptions with a reason, never
  debt waiting for a later branch — a list of postponed fixes is a backlog wearing a gate's name.
- **Clippy's lints are declared once for the workspace, in all sixteen crates** (owner):
  `wildcard_imports` and `wildcard_enum_match_arm`, with a reasoned `#[allow]` where a wildcard
  is legitimate.
- **`for_tests` goes behind a `test-api` feature per crate** (owner), enabled through
  `[dev-dependencies]`, with a gate that the release graph never carries it.
- **Every check is seen failing before it is trusted.** On a fixture first, where the check has
  fixtures (`scan-conventions.mjs` already does this with `FIXTURES`), then on the repository.
  An instrument that never spoke proves nothing (CLAUDE.md, *Measuring on real data*).

## 2. The branches

Eight branches, one at a time, each in its own worktree and merged `--no-ff` into `develop` from
the main worktree. Smallest first, to prove the shape; the one touching every crate second,
before the others add code for it to lint; the three UI ones last, independent of each other.

| # | Branch | Check | Violations closed |
|---|---|---|---|
| 1 | `feature/fieldless-enums` | C1 | `WantDiagnostic`, `ChallengesDiagnostic` become bare camelCase strings |
| 2 | `feature/workspace-lints` | C3 | V4 (`_ =>` on closed enums), V10's Rust half (`use crate::state::*`) |
| 3 | `feature/samples-gate` | C2 | V5: nine examples, three tests reading `live.rep+…` |
| 4 | `feature/no-serialize-paths` | C7 | whatever stops compiling |
| 5 | `feature/test-api-feature` | C8 | test-only API moved to `pub(crate)` or behind the feature |
| 6 | `feature/ui-layering` | C4 | V7 |
| 7 | `feature/ui-tokens` | C5 | V8 |
| 8 | `feature/ui-expressions` | C6 | V10's UI half |

C9 has no branch of its own: every branch writes, for the rule it touches, what checks it; the
last one adds the list of rules that cannot be checked.

**Out of this block**: V1, V2, V3, V6, V9, V11, V12. None of them has a check to turn on; they are
the card's next block.

## 3. The checks

### C1 · A fieldless enum is never tagged

A test in `crates/ipc/tests/contract_shapes.rs` over `ipc::contract::render()` — the generated
`types.ts`, not a list of names, so a new offender is found the day it is written. It fails on
every `export type X =` whose union members are **all** exactly `{ kind: '…' }`. A union with even
one data-carrying member is a tagged enum by right (CLAUDE.md: the tag exists to distinguish
variants that carry different data) and passes.

The recognizer is a pure function in `ipc::contract`, so the fixture that shows it failing is a
string handed to that function. Closing the two violations changes the wire: the Rust enums lose
`tag = "kind"`, `pnpm ipc:types` rewrites `types.ts` into the `const` pair, and the UI's
consumers switch on the value instead of `.kind`.

### C2 · `samples/` is opened from `test-support` only

`scripts/check-samples-access.mjs`, a gate in `scripts/check`. Over the tracked `.rs` files outside
`crates/test-support/`, with `//` comments blanked first (the repo writes "samples/ only from
test-support" everywhere, and a rule that trips on its own explanation trains people to reword
it — B59), it flags:

- a file holding both `CARGO_MANIFEST_DIR` and a string literal naming `samples`;
- any string literal starting `"live.` — a sample is named by its date.

Examples are included: that is where nine of the twelve live today. Exemptions are an array at the
top of the script, file and reason, like `scan-conventions.mjs`.

### C3 · Workspace lints

```toml
[workspace.lints.clippy]
wildcard_imports = "warn"
wildcard_enum_match_arm = "warn"
```

`warn` is enough: `scripts/check` runs clippy with `-D warnings`. Every crate gets
`[lints] workspace = true`. `wildcard_imports` already lets `use super::*` through in
`#[cfg(test)]` modules.

**The branch's first step is a measurement**: of the 64 `_ =>` in `src/`, how many the lint
flags, and how many more it finds in test targets. A legitimate wildcard — a match on a string,
a foreign `#[non_exhaustive]` enum, a `_ => panic!` inside an assertion — gets `#[allow]` with the
reason beside it; if test files hold many of those, the allow goes at the top of the file, once.
The numbers and the decision go on the card.

### C4 · UI import direction

A rule in `pnpm scan`:

| from | may not import |
|---|---|
| `lib/`, `stores/`, `composables/` | `@/components`, `@/screens` |
| `components/` | `@/screens` |
| `router/` | `@/components` — it imports `@/screens`, it is what mounts them |

Imports with `?raw` are excluded: they read a file's source as text, not its code
(`lib/scale/virtualRows.test.ts`). Twelve findings today.

### C5 · Visual tokens

Four rules in `pnpm scan`:

- a `:style` binding that **reads** a token (`var(--…)` as a value) — setting `'--x': value` stays
  the sanctioned way to pass a computed value;
- a quarter-step class (`-N.25`, `-N.75`); half steps stay allowed, as the conventions say;
- `z-<number>`;
- `shadow-<name>` unless `--shadow-<name>` is declared. `--shadow-*` is `initial`, so an
  undeclared `shadow-sm` draws nothing and says nothing.

### C6 · UI expressions

Two rules in `pnpm scan`:

- `await` right after `??`, `||`, `&&` or a ternary `?` — a conditional await;
- `case '…'` in a `switch` whose subject is not a `.kind` access. On the tag the literal is the
  sanctioned exception to frontend rule 5; on a value it is the `const` (`IoReason.NotFound`).
  There are 243 `case '` today; the branch measures how many remain once `.kind` switches are
  set aside before fixing the rule's shape.

### C7 · No `Serialize` on a type carrying a path or an offset

The derive leaves `SteamInstall`, `GameInstall`, `SaveCandidate`, `SaveSource` (discovery),
`Save` and `Section` (core-save), `Entry` and `ExtractReport` (unpack). From then on the compiler
is the check. To see it speak, a throwaway `serde_json::to_string(&candidate)` must fail to
compile, and is deleted.

### C8 · `for_tests` stays out of the release

`[features] test-api = []` in `catalog`, `discovery`, `graph`, `ipc`, `store`, `unpack`, `wiki`,
and `#[cfg(feature = "test-api")]` on the module. A crate's own tests enable it through a
self-reference in `[dev-dependencies]`; a crate using another's `for_tests` does the same; an
example that needs it declares `required-features`. With resolver 2, dev-dependency features do
not reach `app`'s build. The gate in `scripts/check`:

```sh
cargo tree -p app -e normal,build -f '{p} {f}'   # must not print test-api
```

Seen failing by adding the feature to a normal dependency for one run. Test-only API that is not
`for_tests` (`SearchIndex::len`, `counter_index`, `CHARACTER_KEYS`, …) moves to `pub(crate)` where
that is enough, behind the feature where it is not.

### C9 · What is checked, and what only holds because it is read

In `CLAUDE.md` and `docs/frontend-conventions.md`, each checked rule names what checks it. A new
list, *rules that hold only because they are read*, names the rest: logic with a return value
outside `app`, a section named from a guess, `Debug` across the IPC, and whatever the branches
find that no regex can see.

## 4. What the checks cannot see

The `pnpm scan` rules are regexes over `.ts` and `.vue`, like the ones already there. Each new
rule writes its limit beside its code, the way the comment pre-pass does ("what it cannot do,
said here rather than found later"). Known now: C6 reads a `switch` subject textually, so a tag
aliased to a local (`const k = x.kind; switch (k)`) reads as a value; C2 reads a file, so a
samples path assembled across two files is invisible to it.

## 5. Done means

- all eight branches merged into `develop`, each with `pnpm check` green;
- every new check was seen failing on a real violation first, and the card says what it listed;
- C1–C9 ticked on card #81, and the card in `UAT`, with nothing past it — none of this is visible
  in a window, so the open question for the owner is only whether the rules chosen are the right
  ones.

---

## 6. The second block: the violations no check can find (V1, V2, V3, V6, V9, V11, V12)

**Status:** decomposition agreed in conversation on 2026-09-24. The first block landed on
`develop` at `0ec9c9b`. These seven have no check to turn on — each is a rule CLAUDE.md now lists
under *"holds only because it is read"*, or a fix to a boundary type — so a branch here closes
what the review of 2026-09-24 named, and its tests are what keep it closed.

Same method as the first block, by the owner's decision on 2026-09-24: branches stacked one on
another in `isaac-dome-checks`, targeted tests while working, **one** `pnpm check` at the top of
the stack, one review of the whole stack, then the merges and the push.

| # | Branch | Items | What changes |
|---|---|---|---|
| 1 | `feature/docs-and-vacuity` | V11, V6 | documents and tests only |
| 2 | `feature/ipc-purity` | V2 | `ipc` stops reading the clock, the pid and the embedded dataset |
| 3 | `feature/logic-out-of-app` | V1 | six functions with a return value move from `app` to `ipc`, with tests |
| 4 | `feature/typed-wire` | V3 | three wire shapes lose a free string or an empty struct |
| 5 | `feature/typed-ids` | V12 | `AchievementId` through `graph` and `plan` |
| 6 | `feature/i18n-sentences` | V9 | one message per sentence, dates formatted in one place |

### 6.1 Decisions

- **V11.** The `run` row of the module table and `run/src/lib.rs` stop crediting `Tail` with "a
  shorter file is a relaunch": `run::resume` says that. `store` building an `IpcError` in
  `degrade.rs` **stays**, and the table says why: `store` is the one crate whose every failure is
  a reason the frontend shows — and the reason it lives there is the dependency: `store` depends on
  `ipc` for `Goal` and `GoalId`, so it is the only crate that sees both halves (the header of
  `degrade.rs` already said so). A second error type mapped one-to-one would be the same enum
  written twice.
- **V6.** A test on real data that walks something counts what it checked and asserts
  `checked > 0`, beside the skip it declares when there is nothing to walk. A silent
  `let … else { continue }` over samples becomes `test_support::skip(…)`.
  `test_support::sample()` tests `is_file()` like its siblings.
- **V2.** `GoalId::new(created_unix: i64, nonce: u64)` — the id is still the hash it is today,
  of values `app` now hands in (the clock, the pid mixed with an allocation's address, as
  `roll`'s seed already arrives). `impl Default for GoalId` goes: a default that generates an id
  is a clock read nobody sees. `target_sprite` takes the dataset as a parameter instead of
  `wiki::Dataset::embedded()` behind a `OnceLock`.
  **Deferred to card #82, S3, on 2026-09-24 (executor's ruling).** The boss keys the dataset
  feeds are read through `wiki_target::boss`, `resolve_target`, `want`, `search` and
  `missing_view` — some thirty callers — and S3 is the item that computes `boss_keys` once per
  catalog and passes it. Threading a parameter now would rewrite the same lines twice. The
  dataset is a compile-time constant with no I/O, so what stays is a hidden dependency, not an
  impure read. V2 is therefore **half closed**: the `GoalId` half landed, this half did not.
- **V1.** Each of the six moves as a pure function in `ipc` over plain data — rows, ids,
  names, never a `tauri::State` — and `app` keeps the lock, the call and nothing else. The
  function names and their modules are the plan's; the rule is that every one gets a test in
  `crates/ipc/tests/` written from what the command does today, **read from the code as the
  spec of its behaviour**, because this is a move and not a change.
- **V3.**
  - `FloorDiagnostic::RulesUnreadable` loses `reason: String`: the rules are embedded at build
    time, so the text of a `serde_json` error is a developer's message, and it is not
    translatable. The variant carries nothing.
  - `DrawnTargetView::Mark { column: String }`, the boss's English name, becomes
    `{ column: MarkColumnView }`, which the UI already translates everywhere else.
  - `Greedier {}` becomes `Greedier`. Tagged, because its enum has a variant with data.
  - `types.ts` is regenerated and the UI follows it.
- **V12.** `catalog::AchievementId` replaces `u32` in the public signatures of `graph` and
  `plan` — `build`, `model`, `rules`, `evaluate`, `plan::model` — and `resolve.rs` stops
  throwing the type away. `ipc` converts to `u32` where the view-model is built, and only
  there. The plan documents (`plan`'s JSON in `store`) keep their on-disk shape: `AchievementId`
  serializes as the `u32` it wraps, which is pinned by a test before the change.
- **V9.** A sentence is one message with named interpolation, never pieces joined in code. The
  `«»` quotes live inside the messages, not in `GoalRow.vue`. One `achievementLabel(id)`. Dates
  go through a `formatDate` in `lib/` beside `formatCount`, the only reader of
  `i18n.global.locale` for formatting; the seven SFCs that read it for that stop.

### 6.2 Done means

- the six branches merged into `develop`, with `pnpm check` green once at the top of the stack
  and one review of the whole stack;
- V1, V2, V3, V6, V9, V11, V12 ticked on card #81, and the card in `UAT` with `NEEDS WINDOW` —
  V9 and V3 change text the user reads, and the C5 roundings from the first block are still
  unseen.
