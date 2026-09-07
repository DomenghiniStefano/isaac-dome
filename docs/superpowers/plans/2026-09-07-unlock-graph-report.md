# The unlock graph (M2) — execution report

**Date:** 2026-09-07
**Spec:** `docs/superpowers/specs/2026-09-07-unlock-graph-design.md`
**Plan:** `docs/superpowers/plans/2026-09-07-unlock-graph.md`
**Branch:** `feature/unlock-graph`, 17 commits, `pnpm check` green.

## The shape of the result

`crates/graph`, pure, no I/O: 45 tests. 637 nodes, **459 prerequisite edges**, 1,209
requirements read from the wiki's typed refs against the user's own catalog.

| outcome | count | |
|---|---|---|
| resolved | 999 | 82.6% — character 396, judged as gating nothing 492, challenge 47, item 46, boss 33 |
| judged inexpressible | 210 | 17.4%, and **every one traces back to a verdict somebody wrote**: nothing is uninterpreted by inattention |

The 210 are fourteen labels, each judged and each recorded with its reason. Six are the
late-game bosses gated by run progress rather than by an achievement — Hush, It Lives!,
Delirium, Mother, The Beast, Ultra Greedier, 178 requirements between them (see finding 8).
The rest:
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
wrong number and was replaced — first by a resolution-rate floor, then, after finding 8,
by the assertion that nothing is uninterpreted without somebody having judged it.

**2. The inventory is 101 targets, not 75** — and it cannot be otherwise. The 75 came from
a probe that resolved names against the catalog first; the generator has no catalog, by the
design decision that keeps `requirements.json` independent of the user's edition. The 26 of
difference are entities that resolve to known bosses, for which a verdict would never be
read.

*Decision taken:* `TargetRow` gained `verdictRequired`, at first false for entities — the
reasoning being that an entity usually resolves to a boss and never reaches the table.
**Finding 8 showed that reasoning was wrong**, and the flag is now true for every row: a
handful of rows that are never read costs less than one silent hole. The file-level test
covers all 101 and runs on any machine, without a game installed.

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

**8. A silent hole, found by the question "are you sure it's only 459 edges?".** The edge
count was arithmetically consistent — 464 requirements naming something with an unlocker,
5 duplicates collapsed, 459 edges — and semantically incomplete. A requirement resolving to
a boss **stopped there**, and 76 of 103 bosses carry no `achievement=` in
`bossportraits.xml`: Hush, Delirium, Mother, The Beast, Mega Satan, Ultra Greed among them.
Those produced no edge *and no unknown*, so a node behind Delirium read as "nothing in the
way" — the exact failure the spec's `behind` verdict was written to prevent, and which that
verdict could never prevent, because name resolution succeeded first and short-circuited
the table.

*Fixed:* an entity only short-circuits the verdict table when it resolves to a boss **the
game itself gates by an achievement**; every other boss has to be judged.
`verdict_required` is now true for every inventory row, and 22 boss targets were curated —
13 `alwaysAvailable`, 3 `behind` (Gish, Steven, C.H.A.D., which the game gates by the
floor-clear achievements), and 6 `unknown`: Hush, It Lives!, Delirium, Mother, The Beast
and Ultra Greedier are gated by **run progress, not by an achievement** — 11 Mom's Heart
kills, The Void, the Ascent, 500 coins donated — which the model has no way to state.

*What it cost:* uninterpreted requirements went from 32 to **210 of 1,209 (17%)**, 178 of
them those six bosses. The edge count did not move: they never produced edges. What moved
is honesty — the nodes behind late-game content now say `Partial` instead of claiming to be
unlockable.

*What it changed in the tests:* the "under 10% uninterpreted" assertion was replaced,
because a ratio conflates two different things — *we identified what the ref points at*
(Delirium: yes) and *we can state its prerequisite* (Delirium: no). The design promises the
first. The test now asserts the thing that actually matters: **every uninterpreted
requirement traces back to a target somebody judged**, so nothing is unknown by inattention.

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
