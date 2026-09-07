# The plan queue — execution report

**Date:** 2026-09-08
**Spec:** `docs/superpowers/specs/2026-09-07-plan-queue-design.md`
**Plan:** `docs/superpowers/plans/2026-09-07-plan-queue.md`
**Branch:** `feature/plan-queue`, `pnpm check` green.

## What was built

`crates/plan`, pure, 26 tests: the queue as one ordered document, the move-and-repair
algorithm, and the bookkeeping that keeps a step alive while any wish still needs it.
`store` gained migration 2, `ipc` the view-model, `app` five commands, `ui` the mirrored
types and typed wrappers.

## The finding that changes what the screen should show

**The unlock graph of this game is almost flat.** Measured across the whole historical
series, the depth of the missing chain per node:

| save | done | chain lengths |
|---|---:|---|
| 2025-06-26 | 302 | `{0: 464, 1: 171, 2: 1, 3: 1}` |
| 2026-09-05 | 384 | `{0: 519, 1: 118}` |

Even at 302 of 637 done — the least advanced era in `samples/` — **exactly one node has a
chain of 3 and one has 2**. Everything else needs zero or one step.

That is not a defect and it isn't the evidence rule flattening things: it is the shape of
the game. The typical requirement is "beat X with character Y", one gate deep, and the
character's own achievement is usually already reachable.

**What it means for the feature.** The automatic fill-in of a wish's chain will almost
always add **zero or one row**. The tree the queue was meant to expand does not, in this
game, go deep. The queue's value is therefore in **ordering many wishes**, not in unrolling
deep trees — and a screen built around "look how deep this goes" would be designing for a
shape the data doesn't have.

## What execution found that the plan didn't know

**1. "Downward moves are never clamped" was wrong, and symmetrically so.** If 3 must stay
below 1, then 1 cannot reach the last row: 3 would have to sit below the end of the list.
Caught by the first test that moved a prerequisite to the bottom.

**2. Gathering prerequisites into a block above the moved row was wrong, and this one
changes behaviour.** The spec said rows the moved one depends on are "gathered into a block
immediately above it". Doing that **moves rows the user had already arranged**: queue
`[7, 41]`, add 512 which needs 7, and 7 gets hauled down below 41 to be adjacent — despite
already being above 512, which is all the constraint asked.

*The rule is asymmetric*, and it is what was described in conversation rather than what the
spec generalised it to:

- **Dependents are dragged.** Move a prerequisite down and what needs it follows, gathered
  below it.
- **Prerequisites are a wall, not cargo.** They never move. They only stop the row from
  rising past them.
- **Everything else stays put.**

A move now changes the minimum it can, and when a row stops short of where it was dropped,
the reason is the row immediately above it.

**3. `missing_chain` had to move from Task 6 to Task 5.** `ipc::queue_view` needs it to
compute `steps_not_queued`, and Task 5 could not compile without it. The plan had the two
in the wrong order.

**4. `IpcError::StoreUnavailable` carries a reason**, which the plan's sample code forgot;
every construction had to name what went wrong, which is the point of the variant.

**5. Migration 2 needed a test the plan didn't ask for: the upgrade path.** A fresh
database exercises "create both tables at once", and every existing install exercises
something else — a version-1 file gaining the queue without losing its goals. That test
builds a version-1 file by hand and opens it with this binary.

**6. A doc comment ended up on the wrong struct.** Inserting `GraphState` in the M2 work
left `ResourcesState`'s doc attached to it, and `ResourcesState` with none. Found while
reading the file to add the queue commands, fixed there.

## Decisions kept from the spec, and worth restating

- **The order is an array, not a `seq` column.** There is no way to write an order that
  contradicts itself, and no half-applied reorder to recover from: a move is one write.
- **`wanted` and `origins` are independent.** A row can be both asked-for and a step of
  something else; a row that is neither is an orphan and goes. That is the whole removal
  rule.
- **A read never writes.** The goals import is its own command, `queue_import_goals`, with
  `GoalsPending { count }` on the view until it is run. Migration 2 seeds nothing, because
  `store` has no catalog and cannot resolve a target to an achievement.
- **An unreadable document is never edited.** The mutating commands refuse rather than
  replace it with a modified empty one: a plan written by a version that knew more is not
  something to overwrite.

## Verification

- `pnpm check` green: `cargo fmt`, `cargo clippy --all-targets -D warnings`,
  `cargo test --workspace`, `pnpm typecheck`, `pnpm lint`, `pnpm format:check`, `pnpm scan`.
- **1 skip** in the whole suite, pre-existing and unrelated: the Python cross-check.
- The repair algorithm is pinned by nine readable cases **and a property over 500
  pseudo-random move sequences**: after any sequence, no prerequisite sits below something
  that needs it, and no row is lost or duplicated.
- On real data: the deepest chain on the current profile, both wishes sharing a step, and
  the check that removing one of them leaves the shared step alone.
- The app builds and starts with the five commands registered.

## What stays open

- **The screen.** There is no UI for the queue: this is the backend and the contract. The
  drag-and-drop, and what a row looks like, belong to the design system.
- **Counters.** A row like "donate 999 pennies" still can't say how close you are.
- **Ordering suggestions.** The app never reorders on its own beyond the repair. Given
  finding 1 above — a flat graph, most wishes independent — a "sort by fan-out" or
  "sort by closeness" action would be worth more here than anywhere else, and it is one
  call to the graph away.
