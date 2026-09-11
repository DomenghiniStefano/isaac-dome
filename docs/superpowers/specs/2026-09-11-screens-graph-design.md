# Design system, cycle 3.3a — the node, Next steps and Unlock (design)

**Date:** 2026-09-11
**Milestone:** design system (`docs/STATUS.md`), cycle 3 of 3, sub-project 3 of 7, first half
**Depends on:** 3.1 (shell) and 3.2 (Completion), `DESIGN-BRIEF.md` §4, §6, §7.1–§7.3, §7.5,
§10, question 4 of §13; `Schermate.dc.html` of the Claude Design export (Unlock, lines 610–757;
Next steps, lines 949–996; `FACETS`, `ST_META`, `unlock()`, lines 1404–1659); the design pack's
committed payloads `contracts/payload/unlock.json` and `next_steps.json`
**Status:** every decision below was taken by the author on the owner's delegation ("procedi
come credi"). Each is marked **(delegated)** so the first look can overturn it cheaply.

## Sub-project 3.3, split in two (delegated)

The decomposition of cycle 3 gives sub-project 3 three screens: Next steps, Unlock and Plan.
They share one thing — the node (§7.1) — and differ in everything else: Next steps is five
cards, Unlock is a filterable virtualized grid of 641 rows, Plan is a queue you drag, with
write commands and a repair rule. One spec for all three would be a spec for two unrelated
problems. So:

| # | half | what it brings |
|---|---|---|
| **3.3a** | **the node, Next steps, Unlock** (this document) | the node's state and its "why", TanStack Table and Virtual, the facets that have data |
| 3.3b | Plan | goals, the queue with drag and its repair, "in the queue" on the other two screens |

## What this half is

Two screens that read the unlock graph and never write. **Next steps** answers "what's worth
doing right now": at most five nodes, all unlockable now, most fan-out first. **Unlock**
answers "what's missing, filterable": every node, with four state cards, a facet drawer, a
search, three sort orders and a virtualized table. Both draw the node the same way, so a
state can't mean one thing on one screen and another on the other.

## Applicable constraints

1. **No IPC change.** `unlock()` and `nextSteps()` exist and carry everything drawn here.
2. **A `partial` node never reads as unlockable now** (§7.1, question 4). It is never gold,
   never a step, never counted among the unlockable.
3. **No facet without data** (§7.5): mode, effort shape, required ending, quality, pool and
   Steam rarity have no field, and a facet filled with guesses would lie. Only facets the
   contract can answer are drawn.
4. **Degrade, never fail**: no catalog, section 1 unread, slots beyond the catalog and a
   catalog beyond the slots are each a state on screen, from `UnlockView.diagnostics`.
5. **No game asset in the package.** Achievement and item icons are `isaac://` links; the
   development fixtures rewrite them to the design pack's files under `import.meta.env.DEV`.
6. **Numbers are fixtures of an era.** The expectations below come from the design pack's
   committed `unlock.json` (the reference profile on 2026-09-08), counted outside our code.

## Decision 1 — the node's state (delegated)

One pure function decides what a node is, for every screen:

| state | when | badge | counts as unlockable |
|---|---|---|---|
| `Done` | `done` | `Done` (tick) | — |
| `Now` | not done, `computed`, `availableNow` | `Now` (star, gold) | yes |
| `Blocked` | not done, `computed`, not `availableNow` | `Blocked` (lock), "bloccato da N" | no |
| `Partial` | not done, `partial` | **`Partial`**, "grafo parziale" | **no, ever** |

- **Done wins over the graph**: a node the save says is done is done, whatever the graph
  could interpret (three `partial` nodes on the reference profile are done).
- **An unknown achievement** (`achievement.kind: 'unknown'`, 4 on the reference profile) has a
  state like any other; its row says "Achievement sconosciuto · slot N" with the placeholder
  art (§5.5's "index known, name unknown").
- **The `Partial` badge is a new variant of the Badge primitive** (answering question 4): the
  blocked colours — it is not unlockable — with a dashed edge and the lock. Dashed says "not
  fully computed", the lock says "not yours yet"; it can't be mistaken for `Now` (gold, star),
  for `Blocked` (solid edge) or for `Unknown` (hatched, "?"). The export's cyan "parziale" is
  not used: cyan means focus (cycle 1). The Kit page shows the variant.

On the reference profile: **387 done, 119 unlockable now, 117 blocked, 18 partial** (641);
three more `partial` nodes are done. The unknown achievements are slots **638–641**: three
done, one partial. Ten distinct characters appear in some node's `missing`.

## Decision 2 — the node's "why" (delegated)

`missing` is what stands in the way, typed; the brief asks the screen to say "1 character and
2 unknown conditions", not "blocked by 3". A pure function groups a node's `missing` by kind
(`character`, `boss`, `challenge`, `item`, `gate`, `unknown`), in that order, with the names or
labels inside each group. It is drawn **in the state badge's tooltip** on both screens: the
badge says the state, the tooltip says why. `gate` and `unknown` labels stay in English, as
the file wrote them, under a heading that says we couldn't interpret them.

## Decision 3 — Next steps (delegated)

From `Schermate.dc.html`, lines 949–996:

- **Header**: title, and the export's line — at most five rows, all unlockable now, the ones
  that open the most; a node the graph can only call partial isn't a step.
- **One card per step**: the rank; the achievement art at `achievement` width on the flat
  mark paper (the drawings are dark strokes on transparency, invisible on the dark theme); the
  text; one `Tag` badge per unlocked target ("Samson · personaggio"); on the right the fan-out
  in `state-now-foreground` over the word "sblocca" — gold is allowed here because every step
  *is* unlockable now; the state badge with its tooltip.
- **Empty**: with a `noCatalog` diagnostic, an `Alert` "Nessun passo: manca il catalogo" and
  why; without it, an `EmptyCategory` "Niente è sbloccabile adesso" (everything done, or
  everything blocked). The diagnostic lives in `UnlockView`, so the screen reads both.
- The export's "already in the Plan's queue" line is 3.3b.

On the reference profile the steps are achievements **484, 488, 489, 479, 480**, fan-out
**23, 23, 23, 22, 21**.

## Decision 4 — Unlock (delegated)

From `Schermate.dc.html`, lines 610–757, keeping only what has data:

- **Header**: title, the line "every node, filterable; one filter matters more than the
  others: unlockable now", and **four state cards** (done, unlockable now, blocked, partial)
  with their counts. A card toggles its state in the State facet.
- **Facet drawer**: a `CardCollapsible`, closed by default, its title summarising the active
  filters ("2 filtri attivi"), an "Azzera" action. Four facets, each a list of values with a
  checkbox and a count:

  | facet | values | from |
  |---|---|---|
  | Stato | done, unlockable now, blocked, partial | Decision 1 |
  | Cosa sblocca | passive, active, familiar, trinket, character, boss, challenge, nothing | `unlocks[]` — a node can match several |
  | DLC di origine | Rebirth, Afterbirth, Afterbirth+, Repentance, not stated | `origin` |
  | Personaggio richiesto | the characters that appear in some node's `missing` | `missing[]` of kind `character` |

  **A value's count is over the nodes that match every other facet and the search**, so a
  count says what picking that value would leave.
- **Results card**: "N righe" in the band; a search `Input` over the achievement's text, its
  condition and the names of what it unlocks; sort as a `ToggleGroup` — **sblocca** (fan-out,
  highest first; the default), **passi mancanti** (blocked-by, lowest first, partial last),
  **nome**; a strip of active filters as removable badges.
- **The table**, virtualized, rows at `row-wide`:

  | column | content |
  |---|---|
  | art | the achievement art at `achievement-thumb` on the mark paper, or the placeholder |
  | Achievement | the text (done rows in `subtle-foreground`), and "slot N" |
  | Cosa sblocca | the first target — its item sprite at 32px when it has one — and "+N" |
  | Condizione | the `hint`, or an `EmptyValue` "nessuna condizione nel file" (354 of 637 are null, which is normal) |
  | Stato | the state badge with its "why" tooltip |
  | Sblocca | the fan-out; "—" for a done node |

- **No results**: an `EmptyCategory` "Nessuna riga con questi filtri" and an "Azzera i filtri"
  button.
- **Diagnostics** above the results: `noCatalog` and `noAchievementSection` as `Alert`s — the
  second says in words that zero rows is not zero done; `slotsBeyondCatalog` and
  `catalogBeyondSlots` as one line under the table, with their counts.

New tokens: `achievement-thumb` 48px (a 263 × 176 drawing at 48 × 32) and the table's fixed
column widths; a `grid-cols-unlock` utility composes them, like `grid-cols-matrix`.

## Decision 5 — state, pure logic and the libraries

- **`stores/graph.ts`** (Pinia): `unlock`, `steps`, status and error; `load()` reads
  `unlock()` and `nextSteps()` together, again whenever the active profile's id changes, and
  clears both first, as `stores/completion.ts` does.
- **`lib/graph/`**, pure, tested first against the committed payload:
  - `nodeState(node)`, `stateCounts(nodes)`;
  - `missingGroups(node)`;
  - `facetValues(node, facet)`, `matchesFilter(node, filter)`,
    `facetCounts(nodes, filter, facet)`, `sortNodes(nodes, sort)`, and the search.
- **TanStack Table** holds the grid's filter and sort state and gives the row model, with our
  predicates as its filter functions; **TanStack Virtual** renders the body. Versions and the
  Vue adapter's patterns are fixed in the plan, from the libraries' documentation.

## Decision 6 — fixtures (delegated)

- **`lib/ipc/fixtures/graph.ts`** answers `unlock` and `next_steps` with the design pack's
  committed payloads, imported under the development build only. Their `isaac://` links become
  the pack's image files (`images/achievement/NNNN_*.png`, the items' sprites) through a glob;
  `?art=none` answers them as `null`.
- **`?catalog=none`** answers what a machine without the game gets, as `ipc::unlock_view`
  builds it: one node per slot, each an `unknown` achievement with the save's `done`, nothing
  unlocked, no origin, nothing missing and a `partial` graph with one unknown; totals with
  `known: 0`; a `noCatalog` diagnostic; no steps.

## i18n

`graph.*` for what both screens share (states, the why's headings, target kinds, origins),
`nextSteps.*` and `unlock.*` for each screen. Achievement texts, conditions, target names and
requirement labels are data, in English.

## Testing

Vitest, test-first, expectations from the committed `unlock.json` and `next_steps.json`:

- **`nodeState`** — each row of Decision 1's table, done winning over `partial`, an unknown
  achievement; **`stateCounts`** on the payload — 387 / 119 / 117 / 18.
- **`missingGroups`** — the order of the kinds, names kept, empty for a done node with nothing
  missing.
- **Facets** — `facetValues` for a node unlocking two kinds, for one unlocking nothing, for a
  `null` origin; `matchesFilter` with values inside one facet (any) and across facets (all);
  `facetCounts` excluding its own facet; counts on the payload: 231 nodes unlock nothing known,
  274 have no origin.
- **Sort** — fan-out descending, blocked-by ascending with partial last, name ascending.
- **Search** — matches the text, the condition, an unlocked target's name; case-insensitive.
- **Fixtures** — `unlock` answers 641 nodes and `slotsBeyondCatalog { count: 4 }`; `next_steps`
  answers five; `?catalog=none` answers none with `noCatalog`.

Visual checks on the development server, with each fixture: the four cards and their counts,
a partial badge next to a blocked one, the why tooltip, facet counts moving as another facet
is picked, no results, the virtualized scroll to the last row, Next steps with and without a
catalog.

## Deviations recorded while planning

- **TanStack Table is not introduced in this half.** Read from the installed package
  (`@tanstack/vue-table` 9.2.4, whose v9 API is `useTable` with `tableFeatures`): its faceted
  unique values count a cell's value, so an array cell ("what it unlocks") would count as one
  value, while the facets need one count per kind and every count must leave out its own facet.
  The pure functions of Decision 5 already filter, count and sort; a second engine over the same
  rows would be a second source of truth. Reconsidered for Collection (3.4), whose columns are
  scalar. TanStack Virtual (3.13.37) renders the body as planned.
- **The filter lives in the Unlock screen**: leaving the tab resets it, until tabs keep their
  state (3.7).
- **The fixtures read the payloads through `import.meta.glob`**, not a JSON import: a file
  outside `src/` imported by name would leave `vue-tsc --build`'s file list.
- **Without a catalog the nodes are not empty.** `DESIGN-BRIEF.md` §7.2 says "empty nodes";
  `ipc::unlock_view` sends one `unknown`, `partial` node per slot with the save's `done`. The
  fixture follows the code, the Unlock screen draws those rows under the `noCatalog` alert,
  and the brief's line is corrected when this half lands.

## Deviations recorded while executing

- **The state is a toggle group, not four cards.** Four clickable cards would need a hand-made
  button; the `ToggleGroup` primitive already toggles several values, by keyboard too, and
  carries each state's name, count and square. The drawer holds the other three facets.
- **A state has two wordings**: "bloccato da N" on the badge, where the number follows, and a
  plain name ("bloccato") on the toggle and the chips (`graph.stateName`).
- **The fan-out sort puts done nodes last**, found by looking: the first row was a done node
  with a fan-out of 10, while what is done opens nothing more for the player.
- **The drawer's summary reads "filtri attivi: N"**, found by looking at "1 filtri attivi".
- **The progress gate waits** while the profile is still loading, instead of asking for a
  choice it doesn't know is needed — found by looking, a flaw of 3.1 the slower graph fixtures
  made visible.
- **The graph's fixture images live in `fixtures/graphArt.ts`** and load only when a screen
  asks for the graph: importing some 1,500 images on every read of the profile made each page
  load wait for seconds.
- **The Partial badge had no classes for four tasks.** Each task's typecheck was filtered with
  `grep "error TS"`, which `vue-tsc`'s coloured output never matches; `scripts/check`, which
  reads the exit code, caught it. Checks are judged by exit code since.
- **A shared `useOnActiveProfile` composable** reloads Completion, Next steps and Unlock when
  the active profile changes, instead of three copies of the same watch.

## Out of scope for this half

- **Plan and the queue**, and "in the queue" / "add to the queue" on these screens (3.3b).
- **Facets with no field** (§7.5) and Steam rarity (network).
- **A node's own page**, and a row opening its wiki page (3.5).
