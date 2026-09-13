# B37 — Searching the unlock graph from the goal (design)

**Date:** 2026-09-13
**Backlog:** B37 in `docs/BACKLOG.md`, logged the same day
**Branch:** not cut yet. The owner asked for **spec and plan only**; execution starts later,
from `develop`, once 3.6 (`feature/screens-goals-detail`) is merged — B32 and B35 are closed
there and this design stands on both.
**Depends on:** 3.3a (a node's state and its why, `NodeStateBadge`, `missingGroups`), 3.3b (the
plan queue: `enqueue`, `GraphDeps`, `queue_add`), 3.5a (a page is a tab location: `pageKey`,
`parsePageKey`, `pageLocation`), 3.5b (the `search` command and the palette's row), 3.5d (a
blocker carries its page), 3.6 (the Goals screen, `GoalCard.vue`, `AchievementRef::condition`,
`UnlockTarget::page`), the tab history (back and forward inside a tab)
**Owner's request (2026-09-13):** "anche ricerca di unlock partendo da obiettivo, quindi voglio
giocare greedome, che cosa devo giocare e così via"
**Status:** four choices below are marked **(owner)** — they were answered in the brainstorm.
Everything else is the author's and marked **(delegated)**, so the first look can overturn it
cheaply.

## What this sub-project is

The app reads the graph one way: from a node to what it unlocks. Unlock's four facets — state,
what it unlocks *by kind*, origin, character — are all properties of a node, and none of them
is "this particular thing I want". A player does not arrive with an achievement id. They arrive
with a want: *Greedier*, *The Forgotten*, the *D6*.

This sub-project adds the other direction: **name a thing, get the ordered series of what you
still have to play for it**, with one gesture that puts the whole series in the Plan.

The traversal itself already exists — `graph::Eval::missing_chain` — and so does half the
reverse lookup (`ipc::achievement_unlocking`). What is missing is the way in, the ordering, and
an honest account of the ways the answer can be "I can't say".

## Applicable constraints

1. **Only resolved view-models cross the IPC.** The frontend holds no edges and no ids it has
   to interpret: it names a `Target` and receives nodes it already knows how to draw.
2. **`Partial` never reads as "nothing in the way".** The graph's own rule (`GraphInfo::Partial`
   carries no `steps_missing` on purpose) extends to a chain: a chain containing uninterpreted
   requirements says so, in a number.
3. **An empty list is not an answer.** `missing_chain` returns an empty vector for four
   different situations — done, available now, not knowable, not in the graph — and this view
   must tell them apart from `NodeInfo`, never from the length of a list.
4. **Degrade, never fail.** No catalog, no profile, no dataset: the view still answers with what
   it knows. Without a profile it still names the achievement that grants the thing; it just
   does not claim where you stand.
5. **Exhaustiveness is mandatory.** Every state is a variant, so a TypeScript `switch` is forced
   to deal with it.
6. **The frontend rules hold** (`docs/frontend-conventions.md`, `pnpm scan`): no `<style>`, no
   visual constants, no `invoke()` in components, no raw `<input>`, no string unions, every
   visible string through `t()` in both `it.ts` and `en.ts`.
7. **The IPC contract is live.** The new type is handed on in `DESIGN-BRIEF.md`, not merely
   committed.

## Decision 1 — the answer lives in the Goals screen, not in Unlock (owner)

The want is a second entry point on the Goals screen (`RouteName.Goals`, the screen 3.6
renamed), above the recommended sections. It is **not** a facet in Unlock.

The reason is the shape of the answer. A want produces a *chain* — an ordered series where the
order is the meaning — and Unlock is a sortable, virtualized table whose every column reorders
the rows. A table that can be re-sorted cannot hold an order that matters. The Goals screen
already draws ordered cards, and 3.6 gave them what a card needs to say.

**While a want is active it replaces the recommended sections**, rather than sitting beside
them: the screen answers one question at a time. Clearing the bar brings the recommendations
back.

## Decision 2 — the vocabulary is what a node unlocks, plus the achievements themselves (owner)

You can name any of the four things a node unlocks — item (trinkets included), character, boss,
challenge — **and any of the 637 achievements**, by the name the game file gives them.

The second half is what makes the request that opened B37 answerable. *Greed Mode* is not a
node and neither is *Greedier*: the mode is not a thing the catalog models, so the achievement
that grants it (*Greedier!*) unlocks "nothing the catalog knows" — one of the 231 such nodes.
Naming the achievement reaches it, and costs no curation: the names come from
`achievements.xml`, not from a list we wrote.

**Refused, and refused in writing:** a hand-written table of modes and events (Greed Mode, Boss
Rush, Hush…) mapping a player's words onto nodes. It is the curation `docs/STATUS.md` keeps
declining for the twelve mark columns (B36), and it would be a second place where a name means
a thing.

## Decision 3 — one pure function in `ipc`, one thin command (owner)

`crates/ipc/src/want.rs`, a new file — `graph.rs` is past 900 lines and this is a question of
its own. The command in `crates/app/src/commands/` is wiring only.

**The input is a `wiki::Target`**, not an `ipc::TargetKey`. Three reasons: it already crosses
the wire and is already mirrored in TypeScript; it is exactly what `SearchHit` carries, so the
typeahead builds nothing; and it is the only one of the two that can say `Achievement { id }`,
which Decision 2 needs. Turning a `Target` into a `TargetKey` (an item's kind, a boss's entity
triple) stays in Rust, where the catalog can answer — the frontend never assembles a key.

*The alternative, refused:* computing the chain on the frontend over the graph store. It is not
possible today — `UnlockNode` carries requirements resolved to names and the counters, never
the prerequisite ids — and making it possible means putting the graph's edges on the wire, a
larger contract change than this one, to duplicate in TypeScript a traversal that in Rust
already handles cycles.

## Decision 4 — the shape of the answer (delegated)

```rust
pub struct WantView {
    /// What you named, drawn back.
    pub wanted: WantedView,
    /// One way per achievement that grants it. Usually one; never silently one.
    pub routes: Vec<WantRoute>,
    pub diagnostics: Vec<WantDiagnostic>,
}

/// A want is one of two things, and `UnlockTarget` can only be one of them: it has four
/// variants and none of them is an achievement. Decision 2 lets you name *Greedier!*, so the
/// view needs both — and a third case for a name the catalog no longer resolves.
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum WantedView {
    Target { target: UnlockTarget },
    Achievement { achievement: AchievementRef },
    Unresolved,
}

pub struct WantRoute {
    pub node: UnlockNode,
    pub state: WantState,
}

#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum WantState {
    /// Already yours.
    Done,
    /// Nothing in the way: play it.
    AvailableNow,
    /// The prerequisites, in the order the Plan would play them, the wanted node excluded.
    /// `unknown` counts the steps — the final node included — whose requirements the graph
    /// only partly interprets.
    Chain { steps: Vec<UnlockNode>, unknown: u32 },
    /// Section 1 was not read: the route is named, where you stand is not claimed.
    NoProfile,
}

#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum WantDiagnostic {
    NoCatalog,
    NoProfile,
    /// `routes` is empty, and this is why: the graph has no node granting this.
    NothingUnlocks,
    /// A stage, a room, a pickup, a transformation: not a thing you unlock.
    NotUnlockable,
}
```

`routes` is a list because two achievements granting the same thing is two ways, and picking
one is a wrong answer wearing a right one's clothes. An empty list always travels with the
diagnostic that says which empty it is.

## Decision 5 — the reverse lookup reads the same `unlocks` the table draws (delegated)

`ipc::achievement_unlocking(catalog, &TargetKey) -> Option<u32>` exists
(`crates/ipc/src/queue.rs:64`) and is a `.find()`: it takes the **first** achievement granting
the target and never says there were others. For importing old goals into the queue that was
enough; for a view that promises to say *how* you get a thing, a hidden second way is a wrong
answer.

It becomes `achievements_unlocking(...) -> Vec<u32>`, and today's function is rewritten over it
(`.first().copied()`) so `queue_import_goals` keeps its behaviour, unchanged and still tested.

The lookup reads the catalog's `unlocks` — the same list `target_of` reads and the same one
Unlock's "Cosa sblocca" column draws. No second table maps a thing to the node that grants it.

## Decision 6 — the order is the queue's, asked rather than reinvented (delegated)

`missing_chain` returns a **set in ascending id order**. As a sequence to play it means
nothing: id order is not dependency order.

`want.rs` builds an **empty throwaway `plan::Queue`**, enqueues the wanted achievement with its
chain and `GraphDeps::new(graph, flags, ids)`, and reads `rows()`. `Queue::enqueue` appends the
chain, then the wish, then runs the repair that pulls prerequisites above it — so the rows come
out in the order the Plan would show.

The gain is not the saved code. It is that **the preview and the write are one computation**:
the "metti nel Piano" button calls the existing `queue_add`, and there is no second ordering
rule to keep aligned by hand. The property is checkable — enqueueing the same want into an
empty queue yields exactly the preview's rows — and in a queue that already holds rows it
narrows to the *relative* order of that want's own rows.

## Decision 7 — four states, none deduced from a length (delegated)

| state | when | read from |
|---|---|---|
| `Done` | already yours | `node.done` |
| `AvailableNow` | nothing in the way | `GraphInfo::Computed { available_now: true }` |
| `Chain { steps, unknown }` | this series is missing | a non-empty chain, or a `Partial` node with computable steps |
| `NoProfile` | where you stand is unknown | the section 1 flags are absent |

Two properties the implementation must keep, because they are the two ways this view would lie:

- **No route is a `Chain` with no steps and `unknown` zero.** That case is either
  `AvailableNow` or a bug: it means something read an empty list as "nothing missing".
- **The `NoProfile` diagnostic is emitted if and only if every route is `NoProfile`.** The
  banner and the rows are two readings of one fact and must not be able to disagree; the screen
  takes the banner from the diagnostic without scanning rows.

## Decision 8 — the want is in the URL, with the codec that already exists (delegated)

`RouteName.Goals` grows the query parameter `want`, whose value is `pageKey(target)` —
`#/goals?want=character:41`. `ui/src/lib/wiki/pageKey.ts` already encodes and parses a `Target`
in both directions, is tested, and returns `null` for exactly the four kinds that are not
things you unlock. No second codec.

Three things come free: back and forward inside the tab (the history that just landed), tab
restore, and a link from anywhere — the wiki page of a locked thing can point at "how do I get
this" without the Goals screen knowing who called. A `want` that does not parse is not an
error: the screen draws the recommendations, as if there were none.

## Decision 9 — the bar reuses the search that exists (delegated)

The "voglio…" field calls the `search` command (debounced), and keeps the hits whose `pageKey`
is not `null`. No second index, and no ranking on the frontend — the backend's order is kept,
as `searchRows` already keeps it.

The row reuses the palette's. If it turns out not to be reusable, the report says why rather
than quietly drawing a second kind of row.

## Decision 10 — what the answer draws (delegated)

Under the bar, one block per route — two ways means two blocks, each with its own button and a
header that says there are two. Inside a block: the cards of 3.6 in order, numbered, the last
one marked as the want itself; every row opens the page of what it names (B35 put `page` on
`UnlockTarget`); an `unknown` above zero adds one line saying how many requirements the app
could not read. One button puts the whole chain in the Plan through `queue_add`, and afterwards
says it is there.

`Done`, `AvailableNow` and `NoProfile` each draw as themselves — one card, with the sentence
that fits — and never as an empty chain.

## What this sub-project does not do

- **No facet in Unlock.** It was the alternative; it stays refused, for Decision 1's reason.
- **No history of past wants, and no comparing two.** One question at a time.
- **No hand-written vocabulary of modes and events.** Decision 2.
- **No new write path.** The only write is the existing `queue_add`.
- **It does not fill `PlanExpansion::Computed`.** That stub is the Plan's own question, and a
  want is not a stored goal.

## Risks, and what the plan must actually check

- **The real catalog may have no target with two routes.** Then the list-not-option shape is
  untested on real data. The plan measures it and the report *states the number*, rather than
  leaving a property that cannot fail (the vacuity rule).
- **The four states may not all occur** on the reference profile. Same treatment: the test names
  which one is absent and why, instead of pretending to cover it.
- **`Chain`'s order depends on `plan`'s repair.** If `enqueue`'s behaviour changes, this view
  changes with it — which is the point, and is why the property is written as "equals what the
  queue does", not as a pinned sequence of ids.

## Tests

**Rust, pure, on synthetic catalogs** (`crates/ipc/tests/want.rs`):
- the reverse lookup in its three cases: one node, several, none;
- `achievement_unlocking` rewritten over `achievements_unlocking` keeps its contract, so
  `queue_import_goals` is unchanged;
- each of the four states from its own cause;
- `WantedView` in its three cases: a target, an achievement named directly, a name the catalog
  no longer resolves;
- the two properties of Decision 7;
- a non-unlockable target: no routes, the `NotUnlockable` diagnostic;
- the JSON shape — camelCase, tagged variants, fields renamed inside struct variants. The
  `summary_shape.rs` case: a rename that crosses the wire with the suite green.

**Rust, on real data** (`crates/ipc/tests/want_real.rs`), every one with its vacuity guard:
- the reference catalog holds a want whose chain is at least three long — otherwise the order
  property proves nothing;
- the order equals the queue's after enqueueing the same want;
- how many targets have more than one route, stated as a number.

**Frontend** (Vitest): the `want` parameter's round trip and the Goals location it builds; the
typeahead's filter; the card model per state, the `unknown` line included; the "already in the
Plan" state.

## Handing on the contract

`WantView` and its enums go into `DESIGN-BRIEF.md` beside the other graph view-models, and
`ui/src/lib/ipc/types.ts` is hand-mirrored in the same commit as the Rust type.
