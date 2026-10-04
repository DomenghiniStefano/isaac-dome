# The lists' table — a cell never leaves its column

Card #100. Design agreed with the owner on 2026-10-04.

## What it is for

In UAT of #53 (3.13c), in the real window, the owner found Challenges' state pill —
*"requisiti mancanti: 1"* — running into the next column and out of the table. The cause is not
the screen: the list tables are each written by hand, the state track is a fixed 8.5rem (136px),
and `Badge` is `whitespace-nowrap shrink-0`, so the Italian label with its icon and padding is
wider than its track **at every window width**, not only at 640 × 480. Unlock has the same track
and the same text.

The owner's decision: **fix the component, not the screen**. There is no component to fix — each
table repeats the same frame (pinned header band, stripes, hover, a wide and a narrow template
kept in step with the `hidden` classes by hand) and none of them has a contract for its cells. So
one is made, and **six tables** move onto it: Challenges, Unlock, Collection, Runs, Live's "what
this run could open", and the wiki lists' table view (`WikiTable`).

Asked in the same review, and part of the same component: **an "Actions" column, always, in every
table**, carrying the button that puts the row in the Plan's queue — and takes it out again.

## What it is not

- Not a merge of the tables' contents. N3 kept `UnlockTable` and `CollectionTable` two files
  because their columns are different things; that stands. Each screen still declares its own
  columns and draws its own cells — the component is the frame and the cell contract.
- Not the completion matrix, Floor's grid, a wiki fact sheet, or Search's results: the first three
  are grids, not lists of rows, and Search's results are a list with no columns.
- Not a change to the queue's rules. What can be added and what can leave are the Plan's existing
  rules (`canQueue`, and "only a row you asked for can leave", `GoalRow.vue`).
- Not a component test harness. The repo has no `@vue/test-utils` and tests no component: its
  logic lives in `lib/` and its presentation is looked at on the Kit page. The component's
  decisions are pure functions in `lib/table/`, tested there.

## Rows are flex, not grid

The column list cannot become a `grid-template-columns` in TypeScript: `pnpm scan` forbids a token
read from a string (`'var(--spacing-…)'`), and rightly. So a row is a **flex** row and each cell
carries its own width — the shape `WikiTable` already has:

- a **fixed** column carries its token's class (`w-challenge-state`), a full literal so Tailwind
  sees it;
- a **grow** column carries its weight as a CSS variable bound from the template
  (`--cell-grow: 1.4`, the sanctioned shape) and one utility, `cell-grow`, reads it:
  `flex: var(--cell-grow) 1 0`. With `min-w-0` that is exactly `minmax(0, 1.4fr)`.

Header and rows are built from the same declarations at the same width, so the columns line up.
A column that folds at compact is a cell that becomes `hidden` — out of the accessibility tree,
which is why the old templates never collapsed a track to zero — and the others take its room:
**there is no narrow template any more**, so the mistake the narrow templates guarded against —
a track dropped while its cell stays — cannot be made.

## The component

Types and decisions in `ui/src/lib/table/gridColumn.ts` (pure, tested); the component in
`ui/src/components/ui/grid-table/GridTable.vue`.

```ts
export const ColumnFold = { Never: 'never', Compact: 'compact' } as const
export const ColumnAlign = { Start: 'start', Center: 'center', End: 'end' } as const
export const ColumnWidthKind = { Fixed: 'fixed', Grow: 'grow' } as const
export const RowHeight = { Wide: 'wide', Wiki: 'wiki' } as const

export type ColumnWidth =
  | { kind: typeof ColumnWidthKind.Fixed; class: string } // 'w-challenge-state'
  | { kind: typeof ColumnWidthKind.Grow; weight: number } // 1.4

export interface GridColumn {
  key: string // the cell slot is #cell-<key>, the optional header slot #head-<key>
  header: Message | null // null: a column with no title (a sprite)
  width: ColumnWidth
  fold: ColumnFold
  align: ColumnAlign
}
```

```vue
<GridTable
  :columns="challengeColumns"
  :rows="rows"
  :row-key="(row) => row.number"
  :row-height="RowHeight.Wide"
  :virtual="false"
  @row-click="(row, event) => …"
>
  <template #cell-state="{ row }"><ChallengeStateBadge :state="row.state" /></template>
  <template #actions="{ row }"><QueueActionButton … /></template>
</GridTable>
```

Generic over the row type (`generic="T"`), like `VirtualRows`. What it guarantees:

- **Containment.** Every cell is `flex min-w-0 items-center px-2` plus the `grid-cell` utility:
  `overflow: clip` with `overflow-clip-margin: var(--spacing)` — nothing leaves its track, and the
  2px focus outline at its 2px offset still shows — and `min-width: 0` on its children, so a
  `truncate` inside a cell truncates. Truncating a long name stays the screen's choice.
- **A badge wraps inside a cell.** The same utility gives a `[data-slot=badge]` inside a cell
  `white-space: normal; max-width: 100%`. A pill too wide for its track goes onto two lines; two
  caption lines fit the 40px row (`h-row-wide`), so the row height — which the virtualizer counts
  on — does not move. `Badge` itself is unchanged.
- **The frame.** The leather band (`bg-band text-band-foreground`, the owner's choice — the
  design system's table header, already `TableHeader`'s and `WikiTable`'s) pinned to the page
  box's top (`sticky top-0 z-raised-header`); rows of `RowHeight` (`h-row-wide` 40px,
  `h-row-wiki` 72px); stripes; hover; the hairline under each row.
- **Headers.** A column's header is its message; a `#head-<key>` slot replaces it, which is how
  `WikiTable` draws its sort arrow.
- **Actions, always.** The last column is always *Actions* (`table.actions`), fixed at
  `--spacing-actions` (4rem: the header's word must fit), added by `withActions` and not
  declarable by a screen. vue-tsc does not report a missing slot — measured: a `<GridTable>` with
  no `#actions` typechecks — so `pnpm scan` does: *a GridTable without #actions*.
- **Virtualization** with `:virtual`, through the existing `VirtualRows` and the row height's
  `rowPx`. Unlock, Collection, Runs and the wiki use it; Challenges (45 rows) and Live do not. It
  passes `offset`/`offsetChange` through (the wiki keeps its list's position) and exposes
  `scrollToIndex` (the Collection's find bar).
- **A row click**, optional (`@row-click`), for Runs and the wiki. The row is not a `<button>`:
  a button inside a button is invalid HTML, and the Actions cell holds one. The keyboard reaches
  the same place through a button in the row — Runs' "Open run" in Actions, the wiki's title.

## Actions and the queue

**The button's state is a pure function**, `ui/src/lib/plan/queueAction.ts`:

```ts
export const QueueAction = {
  Add: 'add', // not queued and queueable
  Remove: 'remove', // queued because you asked for it
  Unavailable: 'unavailable', // dragged in, done, no achievement, or the store can't write
} as const

export interface QueueMembership {
  wanted: Set<number>
  steps: Set<number>
}

export const membershipOf = (view: QueueView | null): QueueMembership
export const queueAction = (
  target: QueueTarget | null, // null: the row has no achievement to queue
  membership: QueueMembership,
  canWrite: boolean,
): QueueAction

export interface QueueTarget {
  achievement: number
  done: boolean
}
```

A row dragged in as a prerequisite is `Unavailable`: it leaves when its wish does, which is the
Plan's rule. A disabled button says nothing about why — the owner's choice; its tooltip and label
say what a click does (`queue.add`, `queue.remove`, the keys that exist).

`components/plan/QueueActionButton.vue` draws it: an icon button, `ListPlusIcon` to add,
`ListMinusIcon` to remove, the add icon disabled for `Unavailable`. It reads the queue through
`useQueueOffer`, calls `add`/`remove`, and is disabled while the store is `busy`. `useQueueOffer`
gains `membership`; the `queued` props Challenges and Unlock pass today go.

| table | `QueueTarget` |
|---|---|
| Challenges | the first reward's achievement (`queueableReward`'s rule), done when the challenge is |
| Unlock | `knownId(node)`, `node.done` |
| Collection | `lock.achievement` when `lock.kind` is `locked`; otherwise none |
| Live | `refNumber` of the row's achievement, carried onto `OpenRow`; not done — Live lists what is missing |
| Wiki | an achievement page: its id, and `done` from its progress; an item or unlockable page not yet unlocked: its `unlockedBy`; otherwise none |
| Runs | — |

In Runs the Actions cell holds **"Open run"** (`runs.open`) instead: what a click on the row
does, made a button that Tab reaches.

Removing from Challenges removes the reward's achievement — the one the button adds. A challenge
with several rewards queues one, as today.

## Migration

The pieces first — `gridColumn.ts` and its utilities, `queueAction.ts`, `GridTable`,
`QueueActionButton` — then one atomic commit per table: Challenges (the bug), Unlock, Collection,
Live, Runs, Wiki. Then the scan rule and the documents.

**New:** `lib/table/gridColumn.ts`; `components/ui/grid-table/`; the `cell-grow` and `grid-cell`
utilities; the `--spacing-actions` token; `lib/plan/queueAction.ts`;
`components/plan/QueueActionButton.vue`; the i18n keys `table.actions` and `runs.open` (it and en).

**Changes:** each table becomes its column array (in `lib/table/columns.ts`, where a test reads
them) and a `<GridTable>` with its slots. The `*Row.vue` fragments (Challenges, Unlock,
Collection, Runs) move into the slots: a cell with logic of its own becomes a small component, a
trivial one stays inline, and a Row left empty is deleted in the same commit.

**Goes:** the ten `@utility grid-cols-{challenges,unlock,collection,runs,live-opens}[-narrow]`;
the `-queue` tokens; `lib/design/tables.ts` and `tables.test.ts`, which held the template pairs
together; the scan rule *narrow grid template with no column hidden*, its two fixtures, its row in
`docs/frontend-conventions.md`, and that document's §"A table drops columns by priority" —
rewritten: a list is a `GridTable`, and a column that folds says so with `fold`.

**The new rule arrives with its check**, in the same commit: a scan rule, *a striped list outside
GridTable*, failing on a row stripe (`bg-row-alt`) outside `components/ui/grid-table/` and
`components/ui/table/`. It reads the form — every hand-rolled list stripes its rows — not a list
of names. It is seen to fail on a fixture first, and its fixtures try the forms this repo writes
(`'bg-row-alt'` in a `cn(…)`, in a ternary, `even:bg-row-alt`). The permanent exceptions —
the completion matrix (`MarksGrid`) and Search's results, which stripe without being tables —
go in `EXEMPTIONS` with their reasons. What it cannot see — a list that does not stripe — goes in
the script's header and in the conventions table.

A second rule, *a GridTable without #actions*, fails on a template that opens `<GridTable` and
never fills `#actions`. What it cannot see: a file with two tables, one of which fills it.

**Kit:** `WidthsSection` shows the tables at the three widths — Live added — and Challenges with a
state of twelve missing requirements, so the case this card was opened for stays on the page.

## Tests

Test-first, expected values from this document.

- `queueAction`: not queued → `Add`; wanted → `Remove`; dragged in → `Unavailable`; done →
  `Unavailable`; no target → `Unavailable`; store can't write → `Unavailable` whatever the rest
  says.
- `membershipOf`: wanted and steps split, no achievement in both, `null` view → both empty.
- `gridColumn.ts`: `withActions` appends Actions last and once; `cellClass` carries the
  containment, the width class or `cell-grow`, `@max-compact/page:hidden` for a `Compact` column
  and nothing for `Never`, and the alignment; `cellStyle` binds `--cell-grow` for a grow column
  and nothing for a fixed one.
- `columns.ts`, every table: keys unique and none named `actions`; a fixed width names a `w-*`
  class whose `--spacing-*` token exists in `spacing.css`, read from the CSS the way
  `tables.test.ts` reads `utilities.css` today — a renamed token breaks a test instead of a
  layout; and every table but Live folds something (3.13b's rule: at compact a table drops what
  explains a row).
- The queue targets of Collection, Live and the wiki: one case per row of the table above.

`tables.test.ts` loses its three tests and the suite gains more; `check-test-count.mjs` will name
the file, and the commit says why.

## What a machine cannot see

"Nothing runs past the page" is held by review. Measured in the browser with the snippet in
`docs/frontend-conventions.md` at 640 × 480 and at 1324 before the card goes to UAT; then the
real window at 100% and 150%, which is the owner's.
