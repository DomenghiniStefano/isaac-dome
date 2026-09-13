# B37 — Searching the unlock graph from the goal — report

**Date:** 2026-09-13
**Branch:** `feature/goals-want`, cut from `develop` at `a83a7ef`, in the worktree
`C:\Projects\isaac-dome-want`
**Spec:** `docs/superpowers/specs/2026-09-13-goals-want-design.md`
**Plan:** `docs/superpowers/plans/archive/2026-09-13-goals-want.md`
**Closes:** B37

## What landed

The graph could only be read one way: from a node to what it unlocks. Now you can name the
thing you want — an item, a character, a boss, a challenge, or an achievement by its own name
— and the answer is the ordered series of what you still have to play, with one gesture that
puts the whole series in the Plan.

- **`crates/ipc/src/want.rs`**, a pure module: it resolves the name, finds every achievement
  that grants it, and asks `Graph::missing_chain` what is missing.
- **The order is the queue's own.** `want.rs` builds an empty throwaway `plan::Queue`,
  enqueues the want with its chain, and reads the rows back — so the preview and the write are
  one computation rather than two rules to keep aligned.
- **Four states, none deduced from a length**: `done`, `availableNow`, `chain { steps,
  unknown }`, `noProfile`. An empty `missing_chain` means four different things, and telling
  them apart is what this view is for.
- **The want is a place**: `#/goals?want=item:105`, using `pageKey`, the codec wiki pages
  already travel by — so back, forward and tab restore reach it for free.
- **The Goals screen answers one question at a time**: while a want is named the
  recommendations step aside, and clearing the bar brings them back.

## What the execution measured, and what it changed

### 1. Fourteen of the 45 challenges have two ways in

The spec argued for `routes: Vec` on a structural reading of the catalog — a challenge's
`unlocked_by` is a list in the game's own file, unlike an item's or a character's, which are
single. Measured on the real catalog: **14 of 45 challenges are named by more than one
achievement**. The list shape is not a precaution; the singular `achievement_unlocking`
(`.find()`, used by the goals import) was hiding a second way on a third of the challenges.
It is now `achievements_unlocking(...) -> Vec<u32>`, with the old one rewritten over it so the
import keeps its behaviour.

### 2. The graph is flat, and the reference profile is the wrong place to test an order

The plan's real-data tests were written against `live.rep+persistentgamedata1.dat`, the sample
every other real test uses. Its vacuity guard fired immediately: **the deepest chain on that
profile is one step**. Measured across five samples, the deepest chain anywhere is
achievement 509's —

| sample | achievements done | deepest chain |
|---|---|---|
| 20240606 | 286 | 3 |
| 20250626 | 302 | 3 |
| 20260629 | 305 | 3 |
| 20260912 co-op partner | 128 | **4** |
| live (2026-08-31) | 379 | 1 |

— because a prerequisite already earned is not in the chain, so chains *shorten* as a profile
advances. The ordering test moved to the young co-op-partner profile, which is the one place
in the collection where there is an order to check at all. The guard did exactly the work it
exists for: without it the test would have compared a one-element list to a one-element list
and reported coverage that wasn't there.

This is also a finding about the product, not only about the tests: **for a player as far
along as the reference profile, "what do I have to play for X" is almost always one step.**
The screen's chain block will be rare there, and the Kit page is where its multi-step shape
can actually be looked at.

### 3. `missing_chain` is on `Graph`, not on `Eval`

The plan had `route_state` take an `Eval`. It is a method on `Graph`; the plan's own first
draft was right and the "correction" made it wrong. Caught at the first compile of Task 5,
fixed in the code, and the command passes the graph it already holds.

### 4. No graph is not a fifth state

When the graph cannot be built, `unlock_view` already writes `GraphInfo::Partial { unknown: 1 }`
for every slot — the repo's existing way of saying "the graph has nothing to say". So
`route_state` reads `done` and `availableNow` from the node itself and only needs the graph to
compute the chain: with no graph the chain comes out empty and `unknown` counts that Partial,
so the route reads "I can't tell you the series", never "nothing is missing". No new variant.

### 5. The palette's row is not reusable, and the reason is the point

The plan hoped `SearchRow.vue` would serve the bar. It cannot: that component draws a
**destination** (`SearchRow` carries a `location` built by `searchRows`), and one hit is three
destinations — a wiki page, an Unlock row, a Collection row. A want is none of them; it is a
question about what to play. The bar draws its own minimal row (sprite plus name) and the
vocabulary filter lives in `wantable()`, tested, rather than inline in the component.

## Tests

- `crates/ipc/tests/want.rs` — 14 tests on synthetic catalogs: the three resolution cases, the
  four states each from its own cause, the two properties (no empty chain that claims nothing
  is missing; the `noProfile` banner iff every route is `noProfile`), and the wire shape.
- `crates/ipc/tests/want_real.rs` — 3 tests, each with a vacuity guard, on the real catalog and
  the young profile; two of them print their measurement.
- `crates/ipc/tests/queue.rs` — the two-route case, and that `achievement_unlocking` keeps its
  contract.
- `ui/`: `wantLocation` (5), `wantBlocks` (7), `useWant` (3). The late-answer guard was checked
  by breaking it: without `if (target.value === asked)` the test fails, which is the only way
  to know it can.
- `pnpm check`: green. 7 skips, all pre-existing (samples not in this collection).

## What this did not do

- No facet in Unlock — the alternative, refused in the spec for the reason that a sortable
  table cannot hold an order that matters.
- No hand-written vocabulary of modes and events. *Greed Mode* is reached by naming the
  achievement *Greedier!*, which is measured data, not curation.
- `PlanExpansion::Computed` is still a stub: a want is not a stored goal.
- The one visible gap: an `UnlockTarget::Boss` want cannot be named from the bar unless the
  dataset has a page for that boss, because the vocabulary is `pageKey`'s. That is the same
  boundary B36 describes, and it is deliberate.
