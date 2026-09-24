# Card #81, second block — Implementation Plan

> **For agentic workers:** executed inline (superpowers:executing-plans), branches stacked in
> `C:\Projects\isaac-dome-checks`, targeted tests per task, **one** `pnpm check` at the top of the
> stack (owner, 2026-09-24), one review of the whole stack, then merges and push.

**Goal:** close V1, V2, V3, V6, V9, V11, V12 of card #81.

**Spec:** `docs/superpowers/specs/2026-09-24-rules-become-checks-design.md` §6.

## Global Constraints

- Each branch starts from the previous one; the first from `develop` at `0ec9c9b`.
- Test-first where a value changes (V2, V3, V12, V9); V1 is a move, its tests read the current
  behaviour from `app` as the spec of the move.
- `types.ts` only through `pnpm ipc:types`. No `Co-Authored-By`. Stage by explicit path.
- A plan's on-disk document (`store`'s JSON) never changes shape.

## Review Focus

- V12: a `plan` document saved before the change reads back after it (serde shape of
  `AchievementId` is the bare `u32`) — pinned by a round-trip test before the type moves.
- V2: two goals created in the same millisecond still get different ids.
- V3: a Roll document saved with the old `column: "Mom's Heart"` string — is it stored? If the
  store keeps the draw, the old shape must still read (or degrade with a diagnostic).
- V9: every language file carries every new key (`entries.test.ts`-style key check).
- V1: a Tainted name that matches two forms stays `AmbiguousCharacter`, not the first match.

---

### Branch 1 · `feature/docs-and-vacuity` (V11, V6)

**Task 1.1 — V11.** `CLAUDE.md` module table row `run`, and `crates/run/src/lib.rs:5`: `Tail`
turns bytes into lines; recognizing a relaunch is `run::resume`. `store` row: one sentence on why
`degrade.rs` builds `IpcError` (spec §6.1). Verify: `node scripts/check-doc-refs.mjs` no `NEW`.

**Task 1.2 — V6, vacuity guards.** In `graph/tests/real_data.rs:9-54,151-186` and
`graph/tests/series.rs:9-69`, `ipc/tests/collection_real.rs:28-45`,
`test-support/tests/logs.rs:32`: count what the loop checked and assert `> 0`; for
`collection_real`, `assert!(!v.items.is_empty())`. Run each test with `--nocapture`, confirm it
still runs on real data (`sample:` lines) and passes.

**Task 1.3 — V6, declared skips.** `graph/tests/support/mod.rs:72-75`
(`let Ok(s) = Save::open … else { continue }` → `test_support::skip`), `test-support/tests/logs.rs:7`
(skip declared twice → once), `test_support::sample()` uses `is_file()`. Run `cargo test -p graph
-p test-support`.

### Branch 2 · `feature/ipc-purity` (V2)

**Task 2.1 — GoalId.** Test first in `crates/ipc/tests/goals.rs`: `GoalId::new(1, 2) ==
GoalId::new(1, 2)`, `!= GoalId::new(1, 3)`, `!= GoalId::new(2, 2)`, 32 hex chars. Then
`GoalId::new(created_unix: i64, nonce: u64)` hashing the two; remove `impl Default`; `app` passes
`now_unix()` and a nonce from `process::id()` mixed with an allocation address and a counter.
Fix every caller (`cargo check --workspace --all-targets`).

**Task 2.2 — target_sprite dataset.** `target_sprite.rs:168-187`: the function takes
`Option<&wiki::Dataset>`; callers in `app` pass the one they hold. Tests updated to pass
`Some(&Dataset::embedded())` or a fixture.

### Branch 3 · `feature/logic-out-of-app` (V1)

One task per site, each: read the command, write the `ipc` function over plain inputs, move the
body, write the tests in `crates/ipc/tests/` from the current behaviour, make `app` call it.

- 3.1 `commands/runs.rs:23-62` sources → `Vec<(RunSource, runs)>` + unreadable count;
- 3.2 `commands/runs.rs:117-131` character match by id/name, Tainted ambiguity;
- 3.3 `commands/runs.rs:136-157` matrix rows for live marks;
- 3.4 `commands/queue.rs:51-61` pending goals;
- 3.5 `commands/roll.rs:89-119` building the draw;
- 3.6 `commands/profile.rs:196-222` `WriteIgnored`/`WriteRefused` decision.

Verify: `cargo test -p ipc`, `cargo clippy -p app -- -D warnings`.

### Branch 4 · `feature/typed-wire` (V3)

**Task 4.1** JSON-shape tests first (`ipc/tests/floor_shape.rs`, `ipc/tests/roll.rs`):
`RulesUnreadable` → `{"kind":"rulesUnreadable"}`; `Mark` → `{"kind":"mark","column":{…MarkColumnView}}`;
`Greedier` → `{"kind":"greedier"}`. Then the types, `pnpm ipc:types`, UI consumers
(`pnpm typecheck` names them), i18n for the column through the existing `MarkColumnView` labels.

### Branch 5 · `feature/typed-ids` (V12)

**Task 5.1** Round-trip test first: a `plan` document serialized today reads back identical after
the change (fixture string in `plan/tests/`). **Task 5.2** `AchievementId` in `graph/build.rs:14,18,28-51`,
`model.rs:36`, `rules.rs:103,289`, `evaluate.rs:77,257`, `plan/model.rs:22,28`, `resolve.rs:170`;
`.0` only in `ipc`. Verify: `cargo test -p graph -p plan -p ipc -p store`.

### Branch 6 · `feature/i18n-sentences` (V9)

**Task 6.1** `formatDate` in `lib/format/` beside `formatCount`, with a Vitest test; replace
`Intl.DateTimeFormat` in `WikiLanding.vue:43-53` and the seven `i18n.global.locale.value` reads.
**Task 6.2** one message per sentence: `QueueCard.vue:117-118`, `QueueFootnotes.vue:18,29-31`,
`ChainCard.vue:37`, `SectionsCard.vue:38-40`, `NothingFound.vue:32-34`, `ActiveProfileCard.vue:35-39`,
`GoalRow.vue:172-174` (quotes into the messages), `achievementLabel(id)`. New keys in both `it.ts`
and `en.ts`. Verify: `pnpm typecheck`, `pnpm ui:test`, `pnpm scan`.

### Close

`pnpm check` once on the top; review of the stack; merges `--no-ff` in order from the main
worktree; push; branches deleted; card: V items ticked, `UAT` + `NEEDS WINDOW`.
