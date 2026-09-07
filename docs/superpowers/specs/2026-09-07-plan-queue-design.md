# The plan queue — an ordered series of achievements (design)

**Date:** 2026-09-07
**Milestone:** M3 (`docs/STATO.md`), the derived plan
**Depends on:** M2, the unlock graph — `docs/superpowers/specs/2026-09-07-unlock-graph-design.md`
**Status:** design approved in conversation, section by section; pending spec review

## What this is

One ordered list of achievements: **the things you want to unlock, in the order you intend
to do them, first one next**. The tree answers "what does this need"; the queue answers
"what do I do tonight, and then what".

The order is **yours** — you move rows where you want them — but it can never contradict
the graph: a prerequisite is never below something that needs it. When a move would break
that, the app **repairs the order instead of refusing the move**. There are no error
messages in this feature, by design.

## Applicable non-negotiable constraints

1. **Read-only on saves.** The queue lives in `isaacdome.db`; the game's files are never
   written.
2. **Degrade, never fail.** An unreadable queue is a declared empty queue, never a silent
   one and never a panic.
3. **Unknown data is not zero data.** A row the graph can't compute says so and carries no
   constraints — see "Rows the graph can't compute".
4. **Only resolved view-models cross the IPC.** The frontend receives rows with names and
   icons already resolved, and asks for moves by achievement id and index.
5. **If a return value is worth checking, it lives in a pure crate.** The repair algorithm
   is the value of this feature; it lives in `crates/plan` and is tested there, not in the
   Tauri crate and not in Vue.

## Decision 1 — a row is an achievement, and the order is an array

A row **is an achievement**, not a target. Wanting Tainted Lost and wanting the achievement
that unlocks Tainted Lost are the same wish seen from two sides; keeping one of them
removes a reconciliation nobody would enjoy.

The order is **the position in an array**, not a `seq` column. With one row per achievement
and a sequence number, the order is *derived* from a set of integers that can contradict
each other — duplicates, gaps, a half-finished move — and the apparent benefit ("I update
one row") is illusory: moving one row rewrites the sequence of everything after it, and the
repair rule below rewrites more. An array has no incoherent state because there is no way
to write one.

```jsonc
[
  { "achievement": 89,  "wanted": true,  "origins": [41, 512] },
  { "achievement": 41,  "wanted": true,  "origins": [] },
  { "achievement": 512, "wanted": true,  "origins": [] }
]
```

- **`wanted`** — you asked for this one, for itself.
- **`origins`** — every wanted row whose chain passes through here. A list, not a single
  value: two wishes can need the same step, and with one slot the second would be lost —
  visibly, on removal, when a step still needed by another wish would look orphaned.

The four combinations each mean something, and the last one is the removal rule:

| `wanted` | `origins` | what it is |
|---|---|---|
| yes | empty | you asked for it, full stop |
| no | non-empty | it arrived as a step of something else |
| yes | non-empty | you asked for it **and** it also serves another wish |
| no | empty | **doesn't exist**: an orphan, and it goes |

## Decision 2 — moving a row: the dragged row always wins

`move(achievement, to_index)` puts that row exactly at `to_index`. It never fails, never
refuses, never asks. Everything else yields around it:

- Rows that **depend on** the moved row and now sit above it are moved to just below it.
- Rows the moved row **depends on** that now sit below it are moved to just above it.
- Yielded rows keep their relative order among themselves.
- The repair is transitive: a yielded row drags its own dependents the same way.

That is the rule stated in one line: **the row you dragged lands where you dropped it, and
the constraint is satisfied by moving the others.** The alternative — clamping the move to
the nearest legal position — would put the row somewhere you didn't ask for, and the user
would have to work out why. Moving the others is visible and explains itself.

Dependency here means the **transitive** prerequisite relation from the graph, restricted
to rows present in the queue. A prerequisite not in the queue constrains nothing: it isn't
in the list to be ordered against.

**Termination and determinism.** The graph is acyclic after M2 (self-references are dropped
with a `SelfPrerequisite` diagnostic, and no real cycle remains), so the repair terminates.
With ties broken by previous relative order, the same move on the same queue always gives
the same result — which the tests pin.

## Decision 3 — rows the graph can't compute carry no constraints

A row whose `GraphInfo` is `Partial` has requirements the graph could not interpret. It
enters the queue, it is **declared** as "steps unknown" rather than showing a number, and
**no constraint binds it**: the graph doesn't know its prerequisites, so it cannot honestly
drag anything or be dragged. You move it freely.

After M2's evidence rule this is a small set — 17 nodes on a real profile, against 216
before — but the rule matters more than the count: it is the one place where the queue could
have invented an order it doesn't know.

## Decision 4 — adding a wish fills its chain in

`enqueue(achievement, wanted: true)`:

1. Ask the graph for the achievement's **missing** transitive prerequisites — not-done, and
   not already satisfied.
2. Rows already in the queue are **not duplicated**: they gain the new id in `origins` and
   keep their position, unless the position now breaks the constraint, in which case the
   repair of Decision 2 applies.
3. Rows not in the queue are appended as `wanted: false` with `origins: [achievement]`, in
   dependency order, immediately **before** the new row.
4. The new row itself goes at the **end** of the queue. A new wish is the lowest priority
   until you say otherwise; the app never guesses that what you just added is urgent.

The app states how many steps it added. That number is `steps_missing` from the graph, and
it is the thing the tree was for.

## Decision 5 — what happens when the world moves under the queue

**You complete something.** On read, rows whose achievement is done are dropped from the
returned queue, and the read reports how many went and which of them were `wanted` — so the
screen can say "you finished Tainted Lost" instead of a row quietly disappearing. Dropping
is not persisted until the next write: a read never writes.

**A wish gains new steps** (a patch, a newer wiki snapshot). The read does **not** silently
insert them: a read that writes is a surprise. Each row carries `steps_not_queued`, and a
wanted row with a non-zero count says so, with an action to add them. The queue stays what
you put in it.

**An achievement leaves the catalog.** The row stays, declared unresolvable — the same rule
`UnresolvedGoal` already applies to goals. It never vanishes and never turns into a
different row.

**The store won't open.** `store_available: false`, the queue is empty and says why. Adding
and moving answer with the existing `storeUnavailable` error.

## Decision 6 — where it lives

| piece | where | why |
|---|---|---|
| the queue model and the repair algorithm | **new pure crate `crates/plan`** | it's the value of the feature and it's worth verifying; it depends on `graph`, and knows nothing about Tauri, SQL or JSON-for-the-UI |
| persistence | `crates/store`, **migration 2** | one document, not a table: see Decision 1 |
| view-models | `crates/ipc` | names, icons and diagnostics resolved, as for every other screen |
| commands | `crates/app` | wiring only |

**Migration 2 creates the queue document empty, and seeds nothing.** It cannot: seeding
means resolving each saved `TargetKey` to the achievement that unlocks it, and `store` has
no catalog — it doesn't even depend on the crate. Importing inside the first `queue()` call
was the obvious alternative and it is worse: it makes a read write, which is the surprise
this spec rules out two decisions above.

So the import is **an explicit command**, `queue_import_goals()`, living in `app` where the
catalog is. While unimported goals exist the view carries `GoalsPending { count }`, and one
action moves them in, in `goals.seq` order. The `goals` table is **left in place and stops
being written** — dropping it would destroy data to save nothing, and keeping it makes the
step reversible.

## The contract

```rust
#[serde(rename_all = "camelCase")]
pub struct QueueView {
    pub rows: Vec<QueueRow>,
    pub diagnostics: Vec<QueueDiagnostic>,
    /// `false` when `store` won't open: the queue can't be seen or changed, and the UI
    /// says so instead of showing an empty list.
    pub store_available: bool,
}

#[serde(rename_all = "camelCase")]
pub struct QueueRow {
    /// The achievement, with its graph info and what it is missing: the same node the
    /// Unlock screen draws, so the two screens can never disagree.
    pub node: UnlockNode,
    pub wanted: bool,
    /// The wanted achievements whose chain passes through this row.
    pub origins: Vec<u32>,
    /// Prerequisites this row still needs that are **not** in the queue. Zero for a row
    /// whose chain is fully queued; unknown rows report zero and say so through `node.graph`.
    pub steps_not_queued: u32,
}

#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum QueueDiagnostic {
    StoreUnavailable { reason: String },
    /// The saved document didn't parse: the queue is empty because it couldn't be read,
    /// which is not the same as being empty.
    Unreadable,
    /// Rows dropped from this read because the profile has completed them.
    Completed { count: u32, wanted: Vec<u32> },
    /// A row whose achievement this catalog no longer knows. It stays, by id.
    Unresolved { achievement: u32 },
    /// Goals saved before the queue existed, not yet imported. One action moves them in;
    /// nothing happens on its own, because a read never writes.
    GoalsPending { count: u32 },
    NoCatalog,
}
```

Commands: `queue()`, `queue_add(achievement)`, `queue_remove(achievement)`,
`queue_move(achievement, to_index)`, `queue_import_goals()`. All four return the whole `QueueView`: the queue is
small, every mutation can reorder much of it, and returning the new truth is cheaper to
reason about than teaching the frontend to replay the repair.

## Tests

**The repair algorithm, on synthetic queues** — this is where the feature is proved:

- A move puts the row at exactly the requested index. Always, for every index.
- Moving a prerequisite below its dependent drags the dependent below it.
- Moving a dependent above its prerequisite pulls the prerequisite above it.
- Yielded rows keep their relative order.
- The repair is transitive: a chain of three moves as one block.
- A `Partial` row is dragged by nothing and drags nothing.
- **Property, over random queues and random moves**: after any move the queue satisfies the
  invariant — no prerequisite below something that needs it — and the moved row is at the
  index asked for. This is the test that matters; the cases above are its readable examples.
- Idempotence: moving a row to the index it already occupies changes nothing.

**Insertion and removal:**

- Adding a wish appends it last, with only its missing steps before it.
- A step already queued is not duplicated and gains an origin.
- Removing a wish removes only the rows left with neither `wanted` nor `origins`.
- Removing a wish whose step is also wanted, or serves another wish, leaves that step.

**Persistence:** migration 2 creates an empty document; `queue_import_goals` brings the old
goals in, in `goals.seq` order, once and not twice; a document that doesn't parse gives an
empty queue plus `Unreadable`; a queue survives reopening.

**On real data:** enqueueing a real late-game achievement produces a chain whose length
equals the graph's `steps_missing`, and every row of the chain is not-done. Skips with a
note when `samples/packed` is absent, like every other real-data test.

## Deliberately out of scope

- **Ordering suggestions.** The app never reorders on its own beyond the repair: no "sort by
  feasibility" button in this cycle. The data to do it exists (`steps_missing`, `fan_out`);
  offering it is a screen decision, and the screen doesn't exist yet.
- **Counters.** A row like "donate 999 pennies" still can't say how close you are. Same
  reason as M2: it arrives with the counter work, by adding a field.
- **Multiple queues / profiles.** One queue, global, like the active profile.
