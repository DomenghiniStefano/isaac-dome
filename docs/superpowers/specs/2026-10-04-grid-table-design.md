# The lists' table — a cell never leaves its column

Card #100. Design agreed with the owner on 2026-10-04.

## What it is for

In UAT of #53 (3.13c), in the real window, the owner found Challenges' state pill —
*"requisiti mancanti: 1"* — running into the next column and out of the table. The cause is not
the screen: the five list tables (Challenges, Unlock, Collection, Runs, Live's "what this run
could open") are each a hand-written CSS grid, the state track is a fixed 8.5rem (136px), and
`Badge` is `whitespace-nowrap shrink-0`, so the Italian label with its icon and padding is wider
than its track **at every window width**, not only at 640 × 480. Unlock has the same track and
the same text.

The owner's decision: **fix the component, not the screen**. There is no component to fix — each
table repeats the same frame (pinned header band, stripes, hover, a wide and a narrow template
kept in step with the `hidden` classes by hand) and none of them has a contract for its cells. So
one is made, and the five tables move onto it.

Asked in the same review, and part of the same component: **an "Actions" column, always, in every
table**, carrying the button that puts the row in the Plan's queue — and takes it out again.

## What it is not

- Not a merge of the tables' contents. N3 kept `UnlockTable` and `CollectionTable` two files
  because their columns are different things; that stands. Each screen still declares its own
  columns and draws its own cells — the component is the frame and the cell contract.
- Not the completion matrix, Floor's grid, or a wiki fact sheet: those are grids, not lists of
  rows, and stay as they are.
- Not a change to the queue's rules. What can be added and what can leave are the Plan's existing
  rules (`canQueue`, and "only a row you asked for can leave", `GoalRow.vue`).

## Rows are flex, not grid

The column list cannot become a `grid-template-columns` in TypeScript: `pnpm scan` forbids a token
read from a string (`'var(--spacing-…)'`), and rightly. So a row is a **flex** row and each cell
carries its own width:

- a **fixed** column carries its token's class (`w-challenge-state`), a full literal so Tailwind
  sees it;
- a **grow** column carries its weight as a CSS variable bound from the template
  (`--cell-grow: 1.4`, the sanctioned shape) and one utility, `cell-grow`, reads it:
  `flex: var(--cell-grow) 1 0`. With `min-w-0` that is exactly `minmax(0, 1.4fr)`.

Header and rows are built from the same declarations at the same width, so the columns line up.
A column that folds at compact is a cell that becomes `hidden`, and the others take its room:
**there is no narrow template any more**, so the mistake the narrow templates guarded against —
a track dropped while its cell stays — cannot be made.

## The component

`ui/src/components/ui/grid-table/` — `GridTable.vue`, `columns.ts`, `index.ts`.

```ts
export const ColumnFold = { Never: 'never', Compact: 'compact' } as const
export const ColumnAlign = { Start: 'start', Center: 'center', End: 'end' } as const
export const ColumnWidthKind = { Fixed: 'fixed', Grow: 'grow' } as const

export type ColumnWidth =
  | { kind: typeof ColumnWidthKind.Fixed; class: string } // 'w-challenge-state'
  | { kind: typeof ColumnWidthKind.Grow; weight: number } // 1.4

export interface GridColumn {
  key: string // the slot is #cell-<key>
  header: Message | null // null: a column with no title (a sprite)
  width: ColumnWidth
  fold?: ColumnFold // default Never
  align?: ColumnAlign // default Start
}
```

```vue
<GridTable
  :columns="challengeColumns"
  :rows="rows"
  :row-key="(row) => row.number"
  :virtual="false"
  @row-click="(row, event) => …"
>
  <template #cell-state="{ row }"><ChallengeStateBadge :state="row.state" /></template>
  <template #actions="{ row }"><QueueActionButton … /></template>
</GridTable>
```

Generic over the row type (`generic="T"`), like `VirtualRows`. What it guarantees:

- **Containment.** Every cell is `min-w-0 overflow-hidden px-2`. Nothing leaves its track, by
  construction. Truncating a long name stays the screen's choice, as today.
- **A badge wraps inside a cell.** `Badge` gains an in-cell form, set by the cell and not by every
  caller: `whitespace-normal max-w-full line-clamp-2`. A pill too wide for its track goes onto two
  lines; two caption lines fit the 40px row (`h-row-wide`), so the row height — which the
  virtualizer counts on — does not move. Outside a table a badge is unchanged.
- **The frame.** The header band pinned to the page box's top (`sticky top-0 z-raised-header
  bg-muted`), rows `h-row-wide`, stripes, hover, the hairline under each row.
- **Actions, always.** The last column is always *Actions* (`table.actions`), fixed at
  `--spacing-actions` (2.5rem), added by the component; a screen cannot declare it or leave it
  out. The `#actions` slot is required by `defineSlots`, so a table that does not fill it does not
  typecheck.
- **Virtualization** with `:virtual`, through the existing `VirtualRows` and `rowWidePx`. Unlock,
  Collection and Runs use it; Challenges (45 rows) and Live do not.
- **A row click**, optional (`@row-click`), for Runs. The row is not a `<button>`: a button inside
  a button is invalid HTML, and the Actions cell holds one.

## Actions and the queue

**The button's state is a pure function**, `ui/src/lib/plan/queueAction.ts`:

```ts
export const QueueAction = {
  Add: 'add', // not queued and queueable — "Add to plan"
  Remove: 'remove', // queued because you asked for it — "Remove from plan"
  Unavailable: 'unavailable', // dragged in, done, no achievement, or the store can't write
} as const

export interface QueueMembership {
  wanted: Set<number>
  steps: Set<number>
}

export const membershipOf = (view: QueueView | null): QueueMembership
export const queueAction = (
  achievement: number | null,
  done: boolean,
  membership: QueueMembership,
  canWrite: boolean,
): QueueAction
```

A row dragged in as a prerequisite is `Unavailable`: it leaves when its wish does, which is the
Plan's rule. A disabled button says nothing about why — the owner's choice; its tooltip says
what a click does, and only when there is a click to do.

`components/plan/QueueActionButton.vue` draws it: an icon button, `ListPlusIcon` to add,
`ListMinusIcon` to remove, the add icon disabled for `Unavailable`. It reads the queue store,
calls `add`/`remove`, and is disabled while the store is `busy`. `QueueMembership` replaces the
`queued` set and array Challenges and Unlock pass today.

| table | achievement | done |
|---|---|---|
| Challenges | the first reward's, `queueableReward`'s rule | state `done` |
| Unlock | `knownId(node)` | `node.done` |
| Collection | `lock.achievement` when `lock.kind` is `locked`, otherwise `null` | false |
| Live | `refNumber` of the row's achievement, carried onto `OpenRow` | false: Live lists only what is missing |
| Runs | — | — |

In Runs the Actions cell holds **"Open run"** (`runs.open`) instead: the same thing a click on the
row does, made a button that Tab reaches.

Removing from Challenges removes the reward's achievement — the one the button adds. A challenge
with several rewards queues one, as today.

## Migration

One atomic commit per table, in this order: Challenges (the bug), Unlock, Collection, Live, Runs.
Before them, the component, `queueAction`, `QueueActionButton` and the badge's in-cell form.

**New:** `components/ui/grid-table/`; the `cell-grow` utility; the `--spacing-actions` token;
`lib/plan/queueAction.ts`; `components/plan/QueueActionButton.vue`; the i18n keys
`table.actions`, `queue.remove`, `runs.open` (it and en).

**Changes:** each `*Table.vue` becomes its column array and a `<GridTable>` with its slots. The
`*Row.vue` fragments (Challenges, Unlock, Collection, Runs) move into the slots: a cell with logic
of its own becomes a small component, a trivial one stays inline, and a Row left empty is deleted
in the same commit.

**Goes:** the ten `@utility grid-cols-{challenges,unlock,collection,runs,live-opens}[-narrow]`; the
`-queue` tokens; `lib/design/tables.ts` and `tables.test.ts`, which held the template pairs
together; the scan rule *narrow grid template with no column hidden*, its two fixtures, its row in
`docs/frontend-conventions.md`, and that document's paragraph on the `@utility` pair — rewritten:
a list is a `GridTable`, and a column that folds says so with `fold`.

**The new rule arrives with its check**, in the same commit: a scan rule, *a list grid outside
GridTable*, failing on a `grid-cols-*` class or a pinned header band outside
`components/ui/grid-table/`. It is seen to fail on a fixture first, and its fixtures try the
forms this repo writes (`@max-compact/page:grid-cols-…`, a class bound with `cn(…)`). The
permanent exceptions — the matrix, Floor, the fact sheet, the facet drawer — go in `EXEMPTIONS`
with their reasons. What it cannot see goes in the script's header and in the conventions table.

**Kit:** `WidthsSection` shows the five tables at the three widths, and Challenges with the
longest state the fixtures carry, so the case this card was opened for stays on the page.

## Tests

Test-first, expected values from this document.

- `queueAction`, one case per row of the table above: not queued → `Add`; wanted → `Remove`;
  dragged in → `Unavailable`; done → `Unavailable`; `null` → `Unavailable`; store can't write →
  `Unavailable` whatever the rest says.
- `membershipOf`: wanted and steps split, no achievement in both, `null` view → both empty.
- `GridTable` (Vitest, `@vue/test-utils`): the header has one cell per column and then Actions,
  in the declared order; every cell carries the containment classes; a `Compact` column carries
  `@max-compact/page:hidden` in the header and in the row; a grow column binds `--cell-grow`, a
  fixed one carries its class.
- Each table's columns: a fixed width names a `w-*` class whose `--spacing-*` token exists in
  `spacing.css`, read from the CSS the way `tables.test.ts` reads `utilities.css` today — a
  renamed token breaks a test instead of a layout.

`tables.test.ts` loses its three tests and the suite gains more; `check-test-count.mjs` will name
the file, and the commit says why.

## What a machine cannot see

"Nothing runs past the page" is held by review. Measured in the browser with the snippet in
`docs/frontend-conventions.md` at 640 × 480 and at 1324 before the card goes to UAT; then the
real window at 100% and 150%, which is the owner's.
