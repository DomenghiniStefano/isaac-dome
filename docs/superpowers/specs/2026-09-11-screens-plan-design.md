# Design system, cycle 3.3b — Plan and the queue (design)

**Date:** 2026-09-11
**Milestone:** design system (`docs/STATUS.md`), cycle 3 of 3, sub-project 3 of 7, second half
**Depends on:** 3.3a (the node, Next steps, Unlock — `2026-09-11-screens-graph-design.md`);
`DESIGN-BRIEF.md` §7.4–§7.6; the plan queue's spec and report (`2026-09-07-plan-queue-*`);
`Schermate.dc.html` of the Claude Design export (Piano, lines 483–640; the queue's logic,
`QUEUE` and `qPrereq`…`qDown`, lines 1422–1607; the view model, lines 2533–2675; "in coda" on
Unlock, line 721, and on Next steps, line 985); the design pack's committed payloads
`contracts/payload/queue.with_rows.json` and `queue.empty.json`
**Status:** every decision below was taken by the author on the owner's delegation ("procedi
come credi"). Each is marked **(delegated)** so the first look can overturn it cheaply.

## What this half is

The one screen of sub-project 3 that **writes**. The Plan is an order, not a set (§7.6): a
queue of achievements you mean to do, the prerequisites they dragged in, and a drag that
reorders it without ever failing. Next steps and Unlock gain the other end of it — a row says
it is already queued, and one action puts it there.

## Applicable constraints

1. **A move repairs, it never fails** (§7.6). No rejected drop, no error for an illegal
   position: there is none. The backend repairs; the screen draws the order it answers with
   and never replays the repair itself.
2. **Every mutation answers with the whole `QueueView`**, and the screen replaces what it
   shows with it. No optimistic reorder.
3. **A row never disappears without a word**: `completed` and `unresolved` are drawn.
4. **Degrade, never fail**: no database, an unreadable queue, no catalog, goals not yet
   imported — each is a state on screen, from `QueueView.diagnostics`.
5. **No game asset in the package**: the queue's nodes are the Unlock fixture's nodes, whose
   links the development fixtures already rewrite to the design pack's files.
6. **Numbers are fixtures of an era**: the three rows of `queue.with_rows.json` (480 Samson
   pulled in by 55; 55 Blood Penny wanted; 69 Platinum God wanted, partial, one step outside
   the queue) and its `completed { count: 1, wanted: [1] }`.

## Decision 1 — a move names a row, not an index (delegated; changes the IPC)

Building the drop on `queue_move(achievement, to)` surfaced two defects, both silent:

- **The screen's indices are not the file's.** `ipc::queue_view` leaves completed and
  unresolved rows out of `rows`, and nothing removes them from the saved document. On the
  reference payload one row is completed, so every index the screen could send is off by one
  from the first row on.
- **`to` is applied after the dragged rows are taken out.** `move_row` removes the moved row
  and its dependents, then clamps `to` against what is left. Queue `[A, B, C, D]` with B
  requiring A: dropping A below C sends `to = 2` and lands `[C, D, A, B]` instead of
  `[C, A, B, D]`. The existing tests never moved a row past one of its own dependents.

The fix names the drop the way the screen sees it — **"right below this row"**:

- **`plan::Queue::move_after(achievement, after: Option<u32>, deps)`**. `None` is the top.
  The target is the position just below `after` among the rows that remain once the moved
  row is out; `move_row` does the rest. An `after` that isn't queued, or is the moved row
  itself, leaves the queue as it is (the screen's view was stale; the answer is the truth).
- **`move_row` counts the dragged rows before `to` out of the target**, so an index means
  the same thing whether or not dependents sit in between. Its doc comment, which still
  describes the gathered-prerequisites rule the report retired, is rewritten.
- **`queue_move(achievement, after: Option<u32>)`** in `crates/app`; on the TypeScript side
  `queueMove(achievement: number, after: number | null)`. `QueueView` does not change, and
  neither does the saved document: no migration.

## Decision 2 — the Plan screen (delegated)

From `Schermate.dc.html`, lines 520–640, keeping what has data:

- **Header**: title "Piano", the export's line — the goals are what you want, the queue the
  order you mean to do them in; a drag repairs around the constraint — and the summary
  "righe: N · chieste: M · tirate dentro: K" (counts after a colon: `useMessages` has no
  plural forms, as in 3.3a).
- **Two columns**: the queue card, fluid; an aside at `plan-aside` (248px) with the proposal.
  Below the `lg` breakpoint the aside goes under the queue.
- **The queue card**: a band with "La coda" and the hint (Decision 3); the rows (Decision 4);
  under them, when there are any, the `completed` and `unresolved` lines (Decision 5).
- **The aside, "Prossimi passi"**: the export's "five rows unlockable now, ordered by how much
  they open. Not your queue: the proposal." Each step is the achievement thumb, its text, its
  fan-out, and either "Aggiungi" (`queueAdd`) or "in coda". Read from the graph store's
  `steps`; empty says "Nessuna proposta adesso".
- **Empty queue**: an `EmptyCategory` "La coda è vuota" and the way in: add from the proposal
  beside it, or from Unlock.

Not taken from the export: the saved goals as chips (the queue stands for them, §7.5; they
reach it through the import, Decision 5), the per-row detail page (everything it says is on
the row or in the state badge's tooltip), and the queue card's resize handle.

## Decision 3 — dragging (delegated)

- **A grip handle per row** (`GripVerticalIcon`, a `Button` of the icon size, "Sposta"), not
  the whole row: the row also holds a remove button and text worth selecting.
- **Pointer events, as `TabStrip`**: a press becomes a drag past a threshold, the pointer is
  captured only then, the rows' rects are read once; the dragged row fades and a line in
  `primary` marks the gap the drop would take. The half of the row under the pointer decides
  before or after, and a pure function turns (order, from, target, side) into **an anchor or
  no move** — dropping a row next to itself sends nothing.
- **Keyboard**: `Alt+↑` / `Alt+↓` on a focused handle moves the row one place, through the
  same anchor.
- **Every row can be moved by hand, `partial` ones included.** The export disables the handle
  on a partial row; but a row the graph can't compute carries no constraint, which makes it
  the freest row to move, not a locked one. It is never dragged along and never walls — that
  is the backend's rule and needs nothing from the screen.
- **The hint says what happened.** Idle: "trascina per riordinare — una mossa ripara, non
  fallisce". Dragging: "rilascia dove vuoi: la coda si ripara". After a move that stopped
  short of the drop: **"si è fermata sotto «X»: è un prerequisito"**, X being the row right
  above where it landed. The export previews the wall while dragging, from a guess matching
  names; the screen doesn't know the graph, so it says it afterwards, from the answer.
  A move can stop short only on the way up — dependents are dragged, never jumped — so the
  check is: the moved row is not right below the anchor.
- **While a mutation is in flight** the handles and the row buttons are disabled.

## Decision 4 — a queue row (delegated)

| part | content |
|---|---|
| grip | Decision 3 |
| position | 1-based, `subtle-foreground`, tabular |
| art | the achievement drawing at `achievement-thumb` on the mark paper |
| text | the achievement's text; under it the game's hint as "indizio del gioco: …" when there is one |
| badges | **"chiesta"** when `wanted` (a new `Wanted` badge variant: a square tag with the primary edge); **"serve «X»"** for each origin, X the origin row's text or "achievement N" when it isn't visible; "sblocca X · kind" for the first unlocked target |
| right | the state badge with its "why" tooltip (3.3a); "sblocca: N" when the fan-out is above zero; **"fuori dalla coda: N passi"** when `stepsNotQueued` is above zero |
| remove | a ghost icon button "Togli dalla coda", **on wanted rows only**: a pulled-in step leaves when the wishes that need it do, which the backend already decides |

## Decision 5 — diagnostics and degraded states (delegated)

| diagnostic | on screen |
|---|---|
| `storeUnavailable { reason }` | a destructive `Alert` "Il piano non è disponibile" with the reason; no queue card, no aside actions — not an empty list |
| `unreadable` | an `Alert` "La coda salvata non si legge con questa versione": it is left as it is, never overwritten; no queue card |
| `noCatalog` | an `Alert` "Serve il gioco installato per leggere la coda"; no queue card |
| `goalsPending { count }` | an `Alert` "Obiettivi salvati da importare: N" with a button "Importa nella coda" (`queueImportGoals`) |
| `completed { count, wanted }` | a line under the rows: "chiuse giocando: N", and when `wanted` isn't empty "fra quelle chieste: …" with their texts from the Unlock view, or "achievement N" |
| `unresolved { achievement }` | a line per id, "achievement N non è più nel catalogo", with "Togli" (`queueRemove`) |

A mutation that answers with an error (`catalogUnavailable`, `storeUnavailable`) leaves the
view as it was and shows a destructive `Alert` with the message above the queue until the
next successful mutation. Its text comes from the same mapping `ProfileError` uses, moved to a
composable both share.

## Decision 6 — "in the queue" on Next steps and Unlock (delegated)

- **Next steps, a card**: "già nella coda del Piano" in `state-done-foreground` when queued
  (the export's line 985); otherwise an outline button "Aggiungi alla coda".
- **Unlock, a row**: " · in coda" after "slot N" on the second line, in the same micro text —
  the export's cyan badge next to the name doesn't fit a 40px row with two lines, and cyan is
  focus. A new narrow column at the end, `unlock-queue` (40px), holds a ghost icon button
  "Aggiungi alla coda" (`ListPlusIcon`) on rows that can be added: not done, a known
  achievement, not already queued.
- **Both read the queue store** and load it with the graph when the active profile changes.
  A queue that can't be read hides the affordances instead of blocking the screen: without a
  view, or with `storeAvailable: false`, nothing says "in coda" and nothing offers to add.
- A mutation error shows the Decision 5 alert on these screens too.

## Decision 7 — state and pure logic

- **`stores/queue.ts`** (Pinia): `view`, `status`, `error` for the read; `busy` and
  `mutationError` for the writes; `load()`, `add(id)`, `remove(id)`, `move(id, after)`,
  `importGoals()`, each replacing `view` with the answer. `load()` clears the view first, as
  the graph store does.
- **`lib/plan/`**, pure, tested first:
  - `queuedIds(view)` — the ids of the rows shown;
  - `queueSummary(rows)` — rows, wanted, pulled in;
  - `originRows(row, rows)` — each origin with its visible row or none;
  - `dropAnchor(ids, from, target, side)` and `stepAnchor(ids, index, direction)` — an
    anchor (`{ after: number | null }`) or no move;
  - `stoppedUnder(rows, achievement, after)` — the row the moved one stopped under, or none;
  - `canQueue(node, queued)` — Decision 6's rule.

## Decision 8 — fixtures (delegated)

- **`lib/ipc/fixtures/queue.ts`** keeps a queue in memory for the page's life, like the chosen
  profile: the stored order and each row's `wanted` and `origins`. It is seeded with the
  payload's three rows **plus achievement 1 done at the top** — the hidden row Decision 1 is
  about, present on the development server. Rows are built from the Unlock fixture's nodes, so
  `?art=none` and the pack's images apply; `stepsNotQueued` is the payload's for the seeded
  rows and 0 for added ones; `completed` is computed from the done rows.
- **The repair is a development port** of `move_after` with the same readable cases as the
  Rust tests, over a dependency relation read from the nodes: a row requires another when a
  `character` it's missing is a character the other unlocks (55 requires 480, which unlocks
  Samson). It approximates the graph for looking at the screen; the app's order is the
  backend's.
- `queue_add` queues a wish with no chain; `queue_remove` follows the backend's rule (a step
  goes when nothing keeps it); `queue_import_goals` seeds the rows and clears `goalsPending`.
- **`?queue=`**: `empty` (no rows, `goalsPending { count: 3 }`), `unavailable`
  (`storeUnavailable`, "database da una versione più recente (3 > 2)"), `unreadable`. With
  `?catalog=none` the view is `noCatalog` and every mutation answers `catalogUnavailable`.

## i18n

`plan.*` for the screen, its rows, hints, diagnostics and the aside; `queue.*` for what the
three screens share (in the queue, add, remove, the mutation error's title). Achievement texts
and hints are data, in English.

## Testing

Rust, test-first, in `crates/plan/tests/order.rs`:

- **The dependents defect**: `[1, 2, 3, 4]`, 2 requires 1, `move_row(1, 2)` gives
  `[3, 1, 2, 4]`.
- **`move_after`**: below a row; `None` to the top; an anchor that is a dependent of the moved
  row (the row takes the dependent's place, the dependent follows); an anchor not queued and
  the row itself leave the queue unchanged; stopping under a prerequisite when rising.
- **The property over 500 random sequences** runs on `move_after` too.

Vitest, test-first:

- **`lib/plan`** — each function above on the payload's rows: summary 3 / 2 / 1; 480's origin
  is 55's row; anchors for a drop above, below, next to itself, at both ends, and for a step at
  each end; `stoppedUnder` for a row walled by 480 and for a free move; `canQueue` for a done
  node, an unknown achievement, a queued one and a free one.
- **The repair port** — the Rust cases, including the dependents defect.
- **The fixture** — the seeded view (3 rows, `completed { count: 1, wanted: [1] }`); moving 480
  below 69 gives 69, 480, 55; moving 55 to the top stops under 480; adding 484; removing 55
  takes 480 with it; `empty` then import; `unavailable`; `?catalog=none`.

Visual checks on the development server: the three row kinds and their badges, a drag down
dragging 55 with 480, a drag up stopping under 480 with its hint, `Alt+↑`, removing and adding
from the aside, "in coda" and the add button on Next steps and Unlock, each `?queue=` scenario
and `?catalog=none`.

## Deviations recorded while planning

- **The fixture's `storeUnavailable` reason is the app's own text**, `store_reason` in
  `crates/app/src/lib.rs` — "database from a newer version (3 > 2)" — not the Italian wording
  quoted in Decision 8.

## Deviations recorded while executing

- **The pack's images are indexed once.** The store's test timed out, and a measurement found
  why: every graph answer with art took 1,728 ms, because `packIconUrl` scanned all 1,500 image
  paths for each achievement and item link. That cost has been on every Unlock, Next steps and
  (now) queue command of the development server since 3.3a. Indexed by prefix: 2 ms.
- **The store's test imports the graph fixture up front**: the first command otherwise pays for
  loading the 1,500 images inside its five seconds.
- **The proposal names what a step unlocks** ("The Lost"), falling back to the achievement's
  text, found by looking: at `plan-aside` every achievement text truncated to "You …". The
  fan-out moves to a second line and the add button to an icon.
- **The grip's label has no arrow glyphs**: the scanner refused ↑ and ↓, which the Determination
  font doesn't draw; it says "Alt e freccia su o giù".
- **`placeholder.graph` is gone**: no route arrives with it any more.

## Out of scope for this half

- **Saved goals as a set**: showing, adding and removing `PlanView.goals`. The queue stands
  for them; the import brings old ones in.
- **Ordering suggestions** (sort the queue by fan-out or closeness) and **counters** — the
  report's open items.
- **A row's detail page** and opening a row's wiki page (3.5).
- **Opening the Plan from an "in coda" mark** on another screen.
