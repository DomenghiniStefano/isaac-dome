# The unlock graph — M2 (design)

**Date:** 2026-09-07
**Milestone:** M2 (`docs/STATO.md`), backlog entry B4
**Status:** design approved in conversation; pending spec review

## Context

Three screens — Unlock, Next steps, Plan — carry a declared `{ kind: "stub" }` wherever
the graph doesn't know yet. The contracts were fixed on 2026-09-05 precisely so this
milestone could fill them without changing the types the frontend consumes. This spec says
how the graph is derived, where it lives, and how it is verified.

The backlog entry (B4) assumed the work was **parsing 283 English condition comments** out
of `achievements.xml` into a versioned rules file. Measurement on 2026-09-07 says
otherwise, and the design changes accordingly: the parsing is already done, by our own
`wiki` crate, and what M2 actually owes is the semantic step and the recursion.

## Applicable non-negotiable constraints

1. **Read-only on saves.** The graph reads section 1 flags through the types that already
   exist; it opens nothing.
2. **No API keys, no network at runtime.** The rules are embedded at build time, like
   `wiki::Dataset::embedded()`. Nothing in this milestone talks to the network.
3. **Degrade, never fail.** A requirement that isn't understood makes the node say so; it
   never becomes "no prerequisites", which would read as "available now".
4. **Unknown data is not zero data.** That is this whole spec in one line.
5. **No `panic!` on data read from disk.** The rules file is data read from disk even when
   embedded: a cycle or a dangling reference degrades.

## The measurement this design rests on

Run on 2026-09-07 against the wiki snapshot of 2026-09-04T17:33:31Z and the catalog built
from the game installed on this machine (Repentance+). The probe was
`crates/catalog/examples/probe_graph.rs`, throwaway, deleted with this spec.

**Coverage of the requirement references.** The wiki dataset carries, for **641 of 641
achievements**, an `infobox.requirements` field: an inline tree with typed refs, never a
free string. None is empty. 589 contain at least one typed ref. Of the 1,034 refs:

| outcome | count | |
|---|---|---|
| resolved against the user's catalog | **919** | 88.9% |
| gates — outside the catalog by nature | 77 | room 41, stage 28, pickup 4, transformation 4 |
| unresolved | 38 (18 distinct) | 15 of them one alias away: the wiki writes `Jacob and Esau`, the game `Jacob & Esau` |

The residue after that one alias is ~23 refs, and reading them shows what they are: pickups
and enemies that appear in the sentence without being prerequisites — `Red Heart`,
`Locked Chest`, `Krampus`, `Shopkeeper`, `rainbow poop`.

**Cross-check against the game's own files.** The wiki declares `unlocks` for 470
achievements; `catalog::unlocks` derives the same relation from `items.xml`,
`players.xml`, `challenges.xml` and `bossportraits.xml`. On the 404 both describe, they
**agree on 397 — 98.3%**. The 7 divergences are not errors:

- **6 are two different id spaces.** The wiki's `entity` id is the game's entity type
  (Gish = 43, Steven = 79); the catalog's `BossId` comes from `bossportraits.xml`
  (Gish = 19, Steven = 20). There is no arithmetic between them: the bridge is **the
  name**, and it resolves 468 of 488 entity refs.
- **1 is a real ambiguity**: achievement 132 unlocks The Soul, which the wiki files under
  character 17 and the catalog under item 335. Both are true of the game; the catalog wins,
  because it is what section 4 of the save is indexed by.

**What this buys.** 464 of 641 achievements have at least one prerequisite that reduces to
another achievement — 140 entirely, 324 mixed — for 2,157 edges. That is the graph.

## Decision 1 — the wiki is the source of requirements, the game is the source of unlocks

The two sources are not interchangeable, and the split is not a compromise:

- **What is needed** comes from the wiki's typed refs. The game files say it for only 283
  achievements, in prose, and say nothing at all for the other 354.
- **Who unlocks what** comes from the catalog — `Character.unlocked_by`,
  `Item.unlocked_by`, `Challenge.unlocked_by` — because that is derived from the user's own
  installation, with their edition and their DLC. The graph never invents an edge from the
  wiki; it reads the edge from the game.

The XML comments stay in the design as a **cross-check**, not as a source: they are the
independent second derivation that makes 98.3% a number worth trusting, and the test that
keeps it honest.

## Decision 2 — a prerequisite is typed by the nature of its target

```rust
// crates/graph/src/model.rs — domain, doesn't cross the IPC

/// What an achievement demands. The type is the point: it says whether the
/// prerequisite has an achievement of its own behind it (character, challenge,
/// item) or is content available from the start (Satan, Mom).
pub enum Requirement {
    Character { id: CharacterId },
    Boss      { id: BossId },
    Challenge { id: ChallengeId },
    Item      { kind: ItemKind, id: ItemId },
    /// Stage, room, mode: outside the catalog by nature. The curated table says
    /// whether it's always available, or what it sits behind. `GateId` is the
    /// target's key in that table — `kind:label`, e.g. `stage:The Void` — and
    /// not an id of the game's: these targets have none on our side.
    Gate      { gate: GateId },
    /// Declared, never dropped: the node knows what it doesn't know.
    Unknown   { label: String },
}
```

The type carries two jobs at once. It decides **where recursion is meaningful** — a
character has an achievement behind it, Satan does not — and it is **what the screen groups
by**, so a node can say "you're missing 1 character and 2 bosses" instead of "blocked by 3".

## Decision 3 — three verdicts, and no fourth

Everything that doesn't reduce to an achievement gets exactly one hand-written verdict:

- `alwaysAvailable` — Satan, Mom, Isaac: fought on night one.
- `behind` — Delirium sits behind The Void; The Beast behind the Ascent.
- `notAPrerequisite` — a Red Heart is a pickup that appears in the sentence.

A target **without** a verdict does not become "no prerequisites". It stays `Unknown`, and
the node it belongs to drops to `Partial`. That chain is what makes faking-by-inattention
impossible.

## The pipeline: generated, corrected, derived

The shape `dataset/` already proved: what a tool writes and what a person writes are
different files, and a test holds them together.

| file | written by | contains |
|---|---|---|
| `crates/graph/rules/requirements.json` | the tool, from `dataset/wiki.json` | typed requirements for the 641 achievements, plus the inventory of unreducible targets |
| `crates/graph/rules/corrections.json` | **by hand** | aliases and the three verdicts |
| the derived graph | `graph::build`, at runtime | resolved against the user's catalog |

```json
{
  "schemaVersion": 1,
  "generatedFrom": { "snapshotAt": "2026-09-04T17:33:31Z", "maxRevid": 269057 },
  "achievements": { "1": { "refs": [{ "kind": "entity", "id": 5, "label": "Red Heart" }] } },
  "targets": [{ "kind": "stage", "label": "Basement", "uses": 28 }]
}
```

`requirements.json` is generated, and carries **no id of ours**: it is independent of which
edition the user has installed. `targets` is what makes the curation sustainable — not a
list to invent but a form to fill in, 75 rows today, thirteen of which cover nearly
everything.

```json
{
  "schemaVersion": 1,
  "aliases": { "Jacob and Esau": "Jacob & Esau" },
  "verdicts": {
    "room:Boss Rush":   { "alwaysAvailable": true },
    "stage:The Void":   { "behind": { "achievement": 403 } },
    "entity:Red Heart": { "notAPrerequisite": true }
  }
}
```

The ids in that example are **illustrative**: which achievement The Void actually sits
behind is decided during curation, against the catalog, and is a task of the plan — not a
fact this spec asserts.

**The generator is a `[[bin]]` inside `crates/graph`**, not a crate of its own.
`wiki-snapshot` is separate because it carries `ureq` and the network must not reach a pure
crate; here there is no extra dependency to isolate. The logic lives in the lib
(`graph::generate`), the bin is I/O, and the script is `pnpm graph:rules`, next to
`wiki:fetch` and `wiki:build`.

**The rules are embedded at build time.** There is deliberately **no runtime path for
"rules missing"**, because there is no file that can be missing. Rules loaded at runtime
would buy updatability-without-recompiling at the cost of one more failure mode; that trade
is worth making in M4's `log-watch`, where the patterns chase the game's patches, and not
here, where the rules chase a wiki snapshot that is already inside the binary.

## Runtime

`Graph::build(&Catalog, &Rules) -> Graph` is pure and knows nothing about the save: it
resolves refs onto the user's catalog ids, applies aliases and verdicts, and produces
edges. It lives in `tauri::State` beside `CatalogState`, built once. Evaluation against a
profile — `graph.evaluate(flags)` — is per call, because the flags change with the active
profile. At 641 nodes and ~2,157 edges this is microseconds: no cache, and therefore no
invalidation to get wrong.

**The four numbers, defined so that two people compute them alike:**

| field | definition | why this one |
|---|---|---|
| `blocked_by` | direct prerequisite achievements not yet done, distinct | it's what you can act on tonight; depth is `steps_missing`'s job |
| `available_now` | zero direct prerequisites missing, **and** no `Unknown` requirement, **and** not already done | the second clause separates "I can" from "I can't tell" |
| `fan_out` | how many achievements have **this one** as a direct prerequisite | it is the ordering of Next steps: what opens the most |
| `steps_missing` | cardinality of the **transitive** set of not-done prerequisites | not path length: a node shared by two branches counts once — it's a number of runs, not of hops |

**Cycles.** The wiki and the catalog are independent derivations; nothing guarantees they
can't jointly produce a loop (the wiki says A needs B, the game says B unlocks A). The walk
is a memoized DFS with a visiting set; on a cycle the nodes involved become `Partial` with a
diagnostic listing them. Not a theoretical case: this is data read from disk, and the
no-panic rule applies to it.

## What changes in existing code

Two additions to the contract, in the spirit the 2026-09-05 spec declared ("it grows by
adding fields, not by changing them"), and one removal that the stub was designed for.

1. **`UnlockNode` gains `missing`** — the typed list of what's absent. This is where the
   grouping lives; without it the count exists but doesn't read.
2. **`GraphInfo` gains `Partial { blocked_by, fan_out, unknown }`** — a node whose
   requirements are only partly interpreted must not be able to claim `available_now`. A
   variant and not one more field on `Computed`, because a new variant **forces** the
   TypeScript `switch` to deal with it while a field is ignored in silence — which is the
   difference between declaring and faking. `Partial` deliberately carries **no
   `steps_missing`**: with some requirements uninterpreted the transitive count isn't
   knowable, and a zero there would be the exact lie the variant exists to prevent.
3. **`GraphInfo::Stub` leaves the wire.** After M2 a node saying `Stub` would be lying: the
   graph exists. The variant was declared as a placeholder for exactly this moment. It goes
   together with its TypeScript mirror and the branch that draws it — which **costs nothing
   today**, while the frontend is one verification page, and would cost later. That is an
   argument for doing M2 before the design system, not after.

`unlock_view(catalog, flags: Option<&[bool]>, …)` keeps its degradation unchanged:
unreadable section 1 still gives zero nodes and `NoAchievementSection`. `next_steps` moves
from `StepsBasis::Stub` to `StepsBasis::FanOut` without touching a type — the variant has
been in the enum since 2026-09-05, which is the fixed contract paying off.

## Tests

The graph changes with every wiki snapshot and every game patch, so pinning "419 unlockable
nodes" writes a test that goes red when the world changes without saying whether the code is
wrong. `CLAUDE.md` already answers this for the historical series — properties, not values —
and it holds here.

**1. Invariants, which survive any patch.**

- `available_now` ⟹ `blocked_by == 0`, and `blocked_by > 0` ⟹ `!available_now`. It looks
  tautological: it is the test that catches a `Partial` wearing `Computed`'s clothes.
- `steps_missing == 0` ⟺ the node is done or available now.
- No edge points outside the catalog. A target resolved to an id that doesn't exist is the
  failure mode of name resolution, and it is silent by construction.

**2. Properties over the historical series** — 31 saves across fourteen months.

- **A node that is `available_now` at time T is, at T+1, either done or still available.**
  Achievements are not lost in this game: if the graph withdraws availability, the graph is
  wrong. One sentence that crosses parser, catalog, rules and transitive closure.
- **`steps_missing` never grows**, per node, along the series. Same shape as the test that
  surfaced the 641 → 642 slot jump.

**3. Pinned numbers, with the era in the name.** 919 resolved, 77 gates, 38 unknown are a
measurement of snapshot `2026-09-04T17:33:31Z` — which is written inside
`requirements.json`, which is itself nailed byte-for-byte by the `derived` test. The pin is
not fragile: it moves when the file moves, which is when someone decided to move it.

**4. The cross-check becomes permanent.** Wiki against game files, with the **7 known
divergences in an explicit, commented list**. A new divergence after a re-snapshot is news,
and gets read before it gets added to the list.

**5. Two structural tests on the pipeline.**

- `derived` — `requirements.json` byte-for-byte equal to `generate(dataset/wiki.json)`,
  exactly what `crates/wiki` already does. Regenerating and forgetting to commit goes red.
- `every_target_has_a_verdict` — every row of `targets` has a verdict in
  `corrections.json`, and **when one is missing the test fails, listing which**. The red is
  intended: when a new snapshot introduces a new target, either it gets curated or it gets
  known. A warning instead would mean nodes silently dropping to `Partial` with nobody
  noticing.

The declared cost: **every regeneration of the wiki dataset may demand hand curation before
the suite is green again**. That is the price of having no invented data, and it is paid
once per release, not once per launch.

**6. Unit tests on tiny synthetic graphs** for the logic that doesn't need the game: five
nodes with a cycle, a gate with no verdict, an alias that resolves. Those run on any
machine; the real-data tests declare their sample through `test-support` and skip with a
note where `samples/packed` isn't there.

## Deliberately out of scope

- **Quantitative progress on counters.** ~40 conditions are thresholds — `donate 999
  pennies`, `destroy 100 tinted rocks` — and section 2 holds the matching counters,
  labelled in `reference/isaac_counters.py`. It is the only place the graph could say *how
  close* you are, and it is the natural next step: it arrives by **adding a field**, which
  is the contract's declared way of growing. It stays out because M2 already introduces one
  curated table, and two new curations in one milestone is where mistakes live.
- **M3's derived plan.** The graph gives the plan its raw material; ordering a plan is
  another milestone.
- **The transitive explanation in the UI** ("to get here: first A, then B"). The data is
  there once `steps_missing` exists; how to draw it is design's call, after the handoff.
