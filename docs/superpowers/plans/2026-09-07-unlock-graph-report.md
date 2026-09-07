# The unlock graph (M2) — execution report

**Date:** 2026-09-07
**Spec:** `docs/superpowers/specs/2026-09-07-unlock-graph-design.md`
**Plan:** `docs/superpowers/plans/2026-09-07-unlock-graph.md`
**Branch:** `feature/unlock-graph`, 13 commits, `pnpm check` green.

## The shape of the result

`crates/graph`, pure, no I/O: 42 tests. 637 nodes, **459 prerequisite edges**, 1,247
requirements resolved from the wiki's typed refs onto the user's own catalog.

| outcome | count | |
|---|---|---|
| resolved | 1,215 | 97.4% — boss 493, character 396, challenge 47, item 46, judged as gating nothing 189 |
| uninterpreted | 32 | 2.6%, and **every one of them is an explicit `unknown` verdict**: nothing is uninterpreted by inattention |

The uninterpreted 32 are eight labels, each judged and each recorded with its reason:
`Guppy` and `Beelzebub` (a transformation is three items, and the model can't say "N of
these"), `ending` (`all endings` is a set), `Collect`/`collect`/`collection` (the
collection page is a set), `Bestiary`, and `tainted character` (any of the 17, each behind
its own achievement: a disjunction).

## What execution found that the plan didn't know

**1. The spec's "2,157 edges" was wrong, and it was mine.** The probe that produced it
summed, over every ref, how many achievements the *wiki* says unlock that thing — a
different relation, without dedup. The real graph has 459 edges, and that is correct:
493 of 1,247 requirements are bosses, and Satan or Mom aren't unlocked by any achievement,
so they rightly produce no edge. The plan's `edges > 1500` assertion was calibrated on the
wrong number and was replaced by what the design actually promises: a resolution rate above
90%, asserted as `unknown * 10 < total`.

**2. The inventory is 101 targets, not 75** — and it cannot be otherwise. The 75 came from
a probe that resolved names against the catalog first; the generator has no catalog, by the
design decision that keeps `requirements.json` independent of the user's edition. The 26 of
difference are entities that resolve to known bosses, for which a verdict would never be
read.

*Decision taken:* `TargetRow` gained `verdictRequired`. It is false for entities only —
whether an entity resolves is a runtime fact — and true for stage, room, pickup and
transformation, which can never resolve. The file-level test covers the 63 mandatory rows
and runs on any machine; the stronger check (nothing left uninterpreted against the real
catalog) lives in the real-data tests. The honesty chain doesn't depend on the flag: an
entity that neither resolves nor has a verdict still ends up `Unknown`.

**3. A fourth verdict was needed, and the spec said "three, and no fourth".** Curation hit
`transformation:Guppy`: genuinely gated, but by three items rather than one achievement.
`notAPrerequisite` would claim it gates nothing, `alwaysAvailable` that it's reachable on
night one, and leaving it out would make it indistinguishable from a row nobody has looked
at yet.

*Decision taken:* `unknown { reason }` — **judged, and the answer is that the model can't
say it**. At runtime it behaves exactly like no verdict (the node drops to `Partial`); what
it adds is the record that a person looked. Without it, curation has to lie in one of two
directions.

**4. The generated file's schema needed one more thought about churn.** `targets` is sorted
(it comes out of a `BTreeMap`) precisely so the `derived` test means something: an unsorted
inventory would rewrite the file on every run and the byte-for-byte comparison would be
noise.

**5. `design-export` also calls `unlock_view`,** which the plan never mentioned. Rather than
pass `None`, it now builds the graph: a design drawn against stubs would be a design of the
placeholder. The checked-in `design-export/isaacdome-design-pack/` was **not** regenerated —
that is a separate decision, and it would add 423 changed files to this branch.

**6. A test passed while running nothing.** `dated_series` takes the suffix without a
leading dot; with `".rep+…"` it matched no file, and all four invariants went green having
evaluated an empty series. `test-support`'s declared `skip:` line is the only reason this
was visible — which is the rule the crate exists for, demonstrated on itself.

**7. The workspace needed no edit.** `members = ["crates/*"]` already covers a new crate;
the plan's step to add it was unnecessary.

## Decisions inside the contract

`GraphInfo::Stub` **left the wire**, as the spec planned: after M2 a node claiming "the
graph doesn't exist" would be lying. `StepsBasis::Stub` went with it, for the same reason;
`StepsBasis` keeps one variant because the ordering is a decision the screen reads, and the
next basis has to arrive as a value rather than a rename. `PlanExpansion::Stub` **stays** —
that's M3, and it's a different field.

`next_steps` changed meaning, and this is the change most visible on screen: it used to be
"the first five not-done, in slot order", and it is now "what is unlockable **now**, most
fan-out first, ties by id". A blocked node isn't a step — tonight can't touch it — and a
`Partial` node isn't one either, because recommending what the graph can't vouch for is a
guess. Consequence, asserted by a test: **without a catalog, Next Steps is empty**, and the
view's `NoCatalog` diagnostic is what says why.

## Verification

- `pnpm check` green: `cargo fmt`, `cargo clippy --all-targets -D warnings`,
  `cargo test --workspace`, `pnpm typecheck`, `pnpm lint`, `pnpm format:check`, `pnpm scan`.
- **1 skip** in the whole suite, pre-existing and unrelated: the Python cross-check, on a
  machine with no Python. 503 real files touched.
- **The historical series runs**: 31 dated saves, from 302 achievements done in June 2025 to
  384 in September 2026. Both properties hold across every consecutive pair — an unlockable
  node is never taken away, and `steps_missing` never grows — plus a guard that the series
  actually progresses, so the two properties can't pass vacuously.
- **The cross-check is permanent**: 397 of 404 achievements described by both the wiki and
  the game's own files agree, with the 7 known divergences listed and explained in the test.
- The Tauri app builds and starts with the graph wired in. **The on-screen numbers were not
  checked by eye** — the same chain is asserted on the live profile by `ipc`'s real-data
  test instead.

## What stays open

- **Quantitative progress on counters** (~40 threshold conditions): deliberately out, per
  the spec. It arrives by adding a field.
- **15 challenge disjunctions**: a challenge unlocked by two to four achievements is "either
  of these", and the model has no OR. Each one is a `Disjunction` diagnostic and its
  requirement is `Unknown` — wrong-free, but it does demote those nodes to `Partial`.
- **Regenerating the design package** with the real graph data now available.
