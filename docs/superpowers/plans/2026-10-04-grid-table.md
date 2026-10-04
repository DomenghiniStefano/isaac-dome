# The lists' table Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task (the owner runs TDD plans inline). Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** One `GridTable` component for the six list tables, whose cells can never leave their column, with an Actions column on every table that adds a row to the Plan's queue or takes it out.

**Architecture:** Rows become flex rows; each cell carries its width (a token class, or a grow weight bound as `--cell-grow`) and a containment utility. The decisions — classes per column, the Actions column, the queue button's state, each table's queue target — are pure functions in `ui/src/lib/`, tested with Vitest; `GridTable.vue` / `GridRow.vue` only render them. Each table migrates in its own commit.

**Tech Stack:** Vue 3 + TypeScript, Tailwind v4 (`@utility`, `@theme` tokens), Vitest, Pinia, `@lucide/vue`, Reka-based primitives in `ui/src/components/ui/`.

**Spec:** `docs/superpowers/specs/2026-10-04-grid-table-design.md`

## Global Constraints

- Work in the worktree `C:\Projects\isaac-dome-grid-table`, branch `feature/grid-table`. Never commit in `C:\Projects\isaac-dome` (that is `develop`, the owner's).
- Stage by explicit path, never `git add -A`. Commit messages: Conventional Commits, scope `ui` (or none for repo-wide docs), English, **no `Co-Authored-By` or any attribution trailer**.
- No string unions: `const X = { … } as const` plus `type X = (typeof X)[keyof typeof X]`; a `Record` keyed by those constants uses computed keys (`[ColumnFold.Never]: …`).
- No `<style>` in SFCs; no hardcoded visual constants (`w-[48px]`, `opacity-50`, `:size="16"`); no raw `<button>`/`<input>`; no visible string in a template — everything through `t(…)`.
- A token is never read from a TypeScript string (`'var(--…)'`); setting a `'--name'` variable is the allowed shape.
- `lib/`, `stores/`, `composables/` import no component; `components/` import no screen.
- No `_ =>`-style fallthrough on a closed set: use `Record<…>` maps or `assertNever`.
- A comment says what and why — no card numbers, dates or "it used to".
- Dead code leaves in the commit that makes it dead (Row components, utilities, tokens, i18n keys).
- Machine rules: never kill processes by image name; do not run `pnpm dev` (it evicts other sessions' servers) — look in the browser through `pnpm ui:dev` on a port of its own (`pnpm ui:dev -- --port 1430`) and stop it by its PID; nothing written under `samples/`.
- Targeted checks during the work (`pnpm --filter ui exec vitest run <file>`, `pnpm typecheck`, `pnpm scan`, `pnpm lint`); the full `pnpm check` once, at the end (Task 12).

## Review Focus

- **A click inside a clickable row** (Runs, Wiki): the Actions button or the wiki's title button must not also fire the row's click — a Ctrl-click would open two tabs. `GridRow` stops propagation on the Actions cell; the wiki title stops its own. Checked in the browser in Tasks 9 and 10.
- **The queue never loaded on a screen that never needed it** (Collection, Live, Wiki): every button would sit disabled with nothing failing. Each of those tasks adds `queue.load()` to the screen's mount and a browser check that a button is enabled.
- **The focus outline inside a clipped cell**: `overflow: clip` would cut the 2px outline at 2px offset; `overflow-clip-margin: var(--spacing)` keeps it. Checked by tabbing onto a title link and an Actions button in Task 5.
- **"Azioni"/"Actions" in its own header cell**: the header clips like any cell, so the column must be wide enough for the word — `--spacing-actions` is 4rem. Checked at 640 × 480 in Task 5.
- **A fold that hides the header cell but not the row cell**: both come from the one `cellClass(column)`, pinned by its test in Task 1.

---

### Task 1: Column model and cell utilities

**Files:**
- Create: `ui/src/lib/table/gridColumn.ts`
- Test: `ui/src/lib/table/gridColumn.test.ts`
- Modify: `ui/src/assets/utilities.css` (add two utilities at the end of the table section)
- Modify: `ui/src/assets/theme/spacing.css` (add `--spacing-actions`)
- Modify: `ui/src/i18n/messages/it.ts`, `ui/src/i18n/messages/en.ts` (add `table.actions`)

**Interfaces:**
- Produces: `ColumnFold`, `ColumnAlign`, `ColumnWidthKind`, `RowHeight` (as-const objects and types); `ColumnWidth`, `GridColumn`; `ACTIONS_KEY = 'actions'`; `withActions(columns: readonly GridColumn[]): GridColumn[]`; `cellClass(column: GridColumn): string[]`; `cellStyle(column: GridColumn): Record<string, string> | undefined`; `rowHeightClass: Record<RowHeight, string>`; `rowHeightPx: Record<RowHeight, (percent: number) => number>`; helpers `fixed(cls: string)`, `grow(weight: number)` returning `ColumnWidth`.

- [ ] **Step 1: Write the failing test**

```ts
// ui/src/lib/table/gridColumn.test.ts
import { describe, expect, it } from 'vitest'
import { rowWidePx, rowWikiPx } from '@/lib/scale/rows'
import {
  ACTIONS_KEY,
  ColumnAlign,
  ColumnFold,
  RowHeight,
  cellClass,
  cellStyle,
  fixed,
  grow,
  rowHeightClass,
  rowHeightPx,
  withActions,
} from './gridColumn'
import type { GridColumn } from './gridColumn'

const name: GridColumn = {
  key: 'name',
  header: 'challenges.columns.challenge',
  width: grow(1.4),
  fold: ColumnFold.Never,
  align: ColumnAlign.Start,
}
const state: GridColumn = {
  key: 'state',
  header: 'challenges.columns.state',
  width: fixed('w-challenge-state'),
  fold: ColumnFold.Compact,
  align: ColumnAlign.End,
}

describe('the Actions column', () => {
  it('is appended last, once', () => {
    const all = withActions([name, state])
    expect(all.map((c) => c.key)).toEqual(['name', 'state', ACTIONS_KEY])
  })

  it('is a fixed column that never folds', () => {
    const actions = withActions([])[0]
    expect(actions.fold).toBe(ColumnFold.Never)
    expect(actions.width).toEqual(fixed('w-actions'))
    expect(actions.header).toBe('table.actions')
  })
})

describe('a cell', () => {
  it('always carries the containment', () => {
    for (const column of [name, state])
      expect(cellClass(column)).toEqual(
        expect.arrayContaining(['grid-cell', 'min-w-0']),
      )
  })

  it('takes its fixed width from the token class, and does not shrink', () => {
    expect(cellClass(state)).toEqual(
      expect.arrayContaining(['w-challenge-state', 'shrink-0']),
    )
    expect(cellStyle(state)).toBeUndefined()
  })

  it('grows by its weight through a variable', () => {
    expect(cellClass(name)).toContain('cell-grow')
    expect(cellStyle(name)).toEqual({ '--cell-grow': '1.4' })
  })

  it('folds at compact only when its column says so', () => {
    expect(cellClass(state)).toContain('@max-compact/page:hidden')
    expect(cellClass(name)).not.toContain('@max-compact/page:hidden')
  })

  it('aligns as its column says', () => {
    expect(cellClass(state)).toContain('justify-end')
    expect(cellClass(name)).toContain('justify-start')
  })
})

describe('a row height', () => {
  it('names the token and the number the virtualizer counts with', () => {
    expect(rowHeightClass[RowHeight.Wide]).toBe('h-row-wide')
    expect(rowHeightClass[RowHeight.Wiki]).toBe('h-row-wiki')
    expect(rowHeightPx[RowHeight.Wide]).toBe(rowWidePx)
    expect(rowHeightPx[RowHeight.Wiki]).toBe(rowWikiPx)
  })
})
```

- [ ] **Step 2: Run it to see it fail**

Run: `pnpm --filter ui exec vitest run src/lib/table/gridColumn.test.ts`
Expected: FAIL — `Failed to resolve import "./gridColumn"`.

- [ ] **Step 3: Write the module**

```ts
// ui/src/lib/table/gridColumn.ts
import type { Message } from '@/i18n/message'
import { rowWidePx, rowWikiPx } from '@/lib/scale/rows'

// A list table's columns, declared once: the header and every row are drawn from the same
// declaration, so a column that folds at compact folds in both, and a cell's width is the
// same in every row. Rows are flex, not grid: a grid template would have to be assembled from
// token names in TypeScript, which `pnpm scan` refuses, while a flex cell carries its own width.

export const ColumnFold = { Never: 'never', Compact: 'compact' } as const
export type ColumnFold = (typeof ColumnFold)[keyof typeof ColumnFold]

export const ColumnAlign = { Start: 'start', Center: 'center', End: 'end' } as const
export type ColumnAlign = (typeof ColumnAlign)[keyof typeof ColumnAlign]

export const ColumnWidthKind = { Fixed: 'fixed', Grow: 'grow' } as const
export type ColumnWidthKind =
  (typeof ColumnWidthKind)[keyof typeof ColumnWidthKind]

// A fixed column names its token's class in full, so Tailwind finds it in this source; a grow
// column shares what is left by weight, the way `minmax(0, <weight>fr)` did.
export type ColumnWidth =
  | { kind: typeof ColumnWidthKind.Fixed; class: string }
  | { kind: typeof ColumnWidthKind.Grow; weight: number }

export const fixed = (cls: string): ColumnWidth => ({
  kind: ColumnWidthKind.Fixed,
  class: cls,
})
export const grow = (weight: number): ColumnWidth => ({
  kind: ColumnWidthKind.Grow,
  weight,
})

export interface GridColumn {
  /** The cell slot is `#cell-<key>`, the optional header slot `#head-<key>`. */
  key: string
  /** `null` for a column with no title, such as a sprite. */
  header: Message | null
  width: ColumnWidth
  fold: ColumnFold
  align: ColumnAlign
}

export const RowHeight = { Wide: 'wide', Wiki: 'wiki' } as const
export type RowHeight = (typeof RowHeight)[keyof typeof RowHeight]

// The class draws the row; the number positions it in a virtual list. Both derive from the same
// rem value (`lib/scale/rows.ts`), so they cannot disagree at any scale.
export const rowHeightClass: Record<RowHeight, string> = {
  [RowHeight.Wide]: 'h-row-wide',
  [RowHeight.Wiki]: 'h-row-wiki',
}
export const rowHeightPx: Record<RowHeight, (percent: number) => number> = {
  [RowHeight.Wide]: rowWidePx,
  [RowHeight.Wiki]: rowWikiPx,
}

export const ACTIONS_KEY = 'actions'

// Every table ends with Actions, and no screen declares it: a table that forgot it would be a
// table whose rows cannot be acted on at all.
const actionsColumn: GridColumn = {
  key: ACTIONS_KEY,
  header: 'table.actions',
  width: fixed('w-actions'),
  fold: ColumnFold.Never,
  align: ColumnAlign.Center,
}

export const withActions = (columns: readonly GridColumn[]): GridColumn[] => [
  ...columns,
  actionsColumn,
]

const alignClass: Record<ColumnAlign, string> = {
  [ColumnAlign.Start]: 'justify-start text-left',
  [ColumnAlign.Center]: 'justify-center text-center',
  [ColumnAlign.End]: 'justify-end text-right',
}

const foldClass: Record<ColumnFold, string | null> = {
  [ColumnFold.Never]: null,
  [ColumnFold.Compact]: '@max-compact/page:hidden',
}

const widthClass = (width: ColumnWidth): string[] =>
  width.kind === ColumnWidthKind.Fixed
    ? [width.class, 'shrink-0']
    : ['cell-grow']

// `grid-cell` is the containment (`utilities.css`): nothing a cell holds leaves its track.
export const cellClass = (column: GridColumn): string[] =>
  [
    'grid-cell',
    'flex',
    'min-w-0',
    'items-center',
    'px-2',
    ...widthClass(column.width),
    alignClass[column.align],
    foldClass[column.fold],
  ].filter((c): c is string => c !== null)

export const cellStyle = (
  column: GridColumn,
): Record<string, string> | undefined =>
  column.width.kind === ColumnWidthKind.Grow
    ? { '--cell-grow': String(column.width.weight) }
    : undefined
```

- [ ] **Step 4: Add the token, the utilities and the message**

In `ui/src/assets/theme/spacing.css`, beside the other table tokens (after `--spacing-challenge-queue`):

```css
  /* The Actions column every list table ends with: one icon button, and the header's word
     ("Azioni", "Actions") at label size, which is the wider of the two. */
  --spacing-actions: 4rem;
```

At the end of the table utilities in `ui/src/assets/utilities.css` (after `grid-cols-live-opens`):

```css
/* A list table's grow column: its share of what the fixed columns leave, by the weight its
   declaration binds as --cell-grow (`lib/table/gridColumn.ts`). With min-w-0 this is
   minmax(0, <weight>fr). */
@utility cell-grow {
  flex: var(--cell-grow) 1 0;
}

/* A list table's cell: nothing it holds leaves its track. `clip` rather than `hidden`, with a
   margin as wide as the focus outline and its offset (base.css, 2px + 2px), so a focused link
   or button inside still shows its ring. Its children may shrink, so a `truncate` inside
   truncates; a badge may wrap to a second line instead of running out — two caption lines fit
   a 40px row. */
@utility grid-cell {
  overflow: clip;
  overflow-clip-margin: var(--spacing);

  & > * {
    min-width: 0;
  }

  & [data-slot='badge'] {
    white-space: normal;
    max-width: 100%;
  }
}
```

In `ui/src/i18n/messages/it.ts` and `en.ts`, a new top-level block next to `queue` (alphabetical order is not kept in these files; put it right before `queue:`):

```ts
  table: {
    actions: 'Azioni',
  },
```

```ts
  table: {
    actions: 'Actions',
  },
```

- [ ] **Step 5: Run the test and the checks that read these files**

Run: `pnpm --filter ui exec vitest run src/lib/table/gridColumn.test.ts src/i18n`
Expected: PASS (the i18n tests hold `it` and `en` to the same keys).
Run: `pnpm typecheck && pnpm scan`
Expected: no errors, `0 violations`.

- [ ] **Step 6: Commit**

```bash
git add ui/src/lib/table/gridColumn.ts ui/src/lib/table/gridColumn.test.ts ui/src/assets/utilities.css ui/src/assets/theme/spacing.css ui/src/i18n/messages/it.ts ui/src/i18n/messages/en.ts
git commit -m "feat(ui): a list table's columns, declared once, and a cell that keeps to its track"
```

---

### Task 2: The queue button's state

**Files:**
- Create: `ui/src/lib/plan/queueAction.ts`
- Test: `ui/src/lib/plan/queueAction.test.ts`
- Modify: `ui/src/composables/useQueueOffer.ts`

**Interfaces:**
- Consumes: `rowId(row: QueueRow): number` from `@/lib/plan/queueRows`.
- Produces: `QueueAction` (`Add`/`Remove`/`Unavailable`); `QueueMembership { wanted: Set<number>; steps: Set<number> }`; `QueueTarget { achievement: number; done: boolean }`; `membershipOf(view: QueueView | null): QueueMembership`; `queueAction(target: QueueTarget | null, membership: QueueMembership, canWrite: boolean): QueueAction`. `useQueueOffer()` returns `{ queue, queued, membership, canWrite }`.

- [ ] **Step 1: Write the failing test**

```ts
// ui/src/lib/plan/queueAction.test.ts
import { describe, expect, it } from 'vitest'
import type { QueueRow, QueueView } from '@/lib/ipc/types'
import { QueueAction, membershipOf, queueAction } from './queueAction'

const row = (id: number, wanted: boolean): QueueRow => ({
  node: {
    achievement: { kind: 'known', id, text: null, condition: null, iconUrl: null },
    done: false,
    unlocks: [],
    origin: null,
    missing: [],
    graph: {
      kind: 'computed',
      availableNow: true,
      blockedBy: 0,
      fanOut: 0,
      stepsMissing: 0,
    },
  },
  wanted,
  origins: wanted ? [] : [1],
  stepsNotQueued: 0,
})

const view = (rows: QueueRow[]): QueueView =>
  ({ rows, storeAvailable: true }) as unknown as QueueView

const membership = membershipOf(view([row(1, true), row(2, false)]))

describe('membershipOf', () => {
  it('splits what you asked for from what was dragged in', () => {
    expect([...membership.wanted]).toEqual([1])
    expect([...membership.steps]).toEqual([2])
  })

  it('reads nothing from a queue that is not there', () => {
    const none = membershipOf(null)
    expect(none.wanted.size).toBe(0)
    expect(none.steps.size).toBe(0)
  })
})

describe('queueAction', () => {
  it('offers to add what is not queued', () => {
    expect(queueAction({ achievement: 3, done: false }, membership, true)).toBe(
      QueueAction.Add,
    )
  })

  it('offers to remove what you asked for', () => {
    expect(queueAction({ achievement: 1, done: false }, membership, true)).toBe(
      QueueAction.Remove,
    )
  })

  it('offers nothing for a row dragged in as a prerequisite', () => {
    expect(queueAction({ achievement: 2, done: false }, membership, true)).toBe(
      QueueAction.Unavailable,
    )
  })

  it('offers nothing for an achievement already earned', () => {
    expect(queueAction({ achievement: 3, done: true }, membership, true)).toBe(
      QueueAction.Unavailable,
    )
  })

  it('offers nothing for a row with no achievement', () => {
    expect(queueAction(null, membership, true)).toBe(QueueAction.Unavailable)
  })

  it('offers nothing when the queue cannot be written, whatever the row', () => {
    for (const target of [
      { achievement: 1, done: false },
      { achievement: 3, done: false },
    ])
      expect(queueAction(target, membership, false)).toBe(
        QueueAction.Unavailable,
      )
  })
})
```

Before writing the `view` helper, open `QueueView` in `ui/src/lib/ipc/types.ts` and build a complete object instead of the cast if its fields are few; the cast is only acceptable if the type carries fields this test has nothing to say about.

- [ ] **Step 2: Run it to see it fail**

Run: `pnpm --filter ui exec vitest run src/lib/plan/queueAction.test.ts`
Expected: FAIL — cannot resolve `./queueAction`.

- [ ] **Step 3: Write the module**

```ts
// ui/src/lib/plan/queueAction.ts
import type { QueueView } from '@/lib/ipc/types'
import { rowId } from './queueRows'

// What the Actions button of a list row does. The rules are the Plan's own: an achievement that
// is neither earned nor queued can be added; only a row you asked for can leave — one dragged
// in as a prerequisite goes when its wish does (`GoalRow.vue`).
export const QueueAction = {
  Add: 'add',
  Remove: 'remove',
  Unavailable: 'unavailable',
} as const
export type QueueAction = (typeof QueueAction)[keyof typeof QueueAction]

export interface QueueMembership {
  wanted: Set<number>
  steps: Set<number>
}

/** The achievement a row would put in the queue, and whether it is already earned. */
export interface QueueTarget {
  achievement: number
  done: boolean
}

export const membershipOf = (view: QueueView | null): QueueMembership => {
  const rows = view?.rows ?? []
  return {
    wanted: new Set(rows.filter((r) => r.wanted).map(rowId)),
    steps: new Set(rows.filter((r) => !r.wanted).map(rowId)),
  }
}

export const queueAction = (
  target: QueueTarget | null,
  membership: QueueMembership,
  canWrite: boolean,
): QueueAction => {
  if (!canWrite || target === null) return QueueAction.Unavailable
  if (membership.wanted.has(target.achievement)) return QueueAction.Remove
  if (target.done || membership.steps.has(target.achievement))
    return QueueAction.Unavailable
  return QueueAction.Add
}
```

- [ ] **Step 4: Give `useQueueOffer` the membership**

```ts
// ui/src/composables/useQueueOffer.ts
import { computed } from 'vue'
import { membershipOf } from '@/lib/plan/queueAction'
import { queuedIds } from '@/lib/plan/queueRows'
import { useQueueStore } from '@/stores/queue'

// What a list beside the Plan may offer: which of its rows are already in the queue, which of
// those you asked for, and whether the queue can be written at all. A queue that couldn't be
// read or saved offers nothing — the rows still show, with every Actions button disabled.
export const useQueueOffer = () => {
  const queue = useQueueStore()
  const queued = computed(() => queuedIds(queue.view))
  const membership = computed(() => membershipOf(queue.view))
  const canWrite = computed(() => queue.view?.storeAvailable === true)
  return { queue, queued, membership, canWrite }
}
```

- [ ] **Step 5: Run the test**

Run: `pnpm --filter ui exec vitest run src/lib/plan`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add ui/src/lib/plan/queueAction.ts ui/src/lib/plan/queueAction.test.ts ui/src/composables/useQueueOffer.ts
git commit -m "feat(ui): a row's queue action, add or remove by the Plan's own rules"
```

---

### Task 3: `GridTable`, `GridRow` and `QueueActionButton`

**Files:**
- Create: `ui/src/components/ui/grid-table/GridTable.vue`
- Create: `ui/src/components/ui/grid-table/GridRow.vue`
- Create: `ui/src/components/ui/grid-table/index.ts`
- Create: `ui/src/components/plan/QueueActionButton.vue`

**Interfaces:**
- Consumes: Task 1's `gridColumn.ts`; Task 2's `queueAction.ts` and `useQueueOffer`; `VirtualRows` (`@/components/ui/virtual`), `ScrollOffset` (`@/lib/scale/scrollOffset`).
- Produces: `<GridTable generic="T">` props `columns: readonly GridColumn[]`, `rows: T[]`, `rowKey: (row: T) => string | number`, `rowHeight?: RowHeight` (default `Wide`), `virtual?: boolean` (default `false`), `clickable?: boolean` (default `false`), `offset?: ScrollOffset | null`; emits `rowClick: [row: T, event: MouseEvent]`, `offsetChange: [ScrollOffset]`; slots `#cell-<key>="{ row, index }"`, `#head-<key>`, `#actions="{ row }"`; exposes `scrollToIndex(index: number)`. `<QueueActionButton :target="QueueTarget | null" />`.

No unit test: the repo tests no component (spec, "What it is not"); every decision these render is tested in Tasks 1–2, and the presentation is looked at in Task 5's browser step.

- [ ] **Step 1: `GridRow.vue` — one row, the cells from the columns**

```vue
<!-- ui/src/components/ui/grid-table/GridRow.vue -->
<script setup lang="ts">
import { cn } from '@/lib/cn'
import { ACTIONS_KEY, cellClass, cellStyle } from '@/lib/table/gridColumn'
import type { GridColumn } from '@/lib/table/gridColumn'

// One row of a `GridTable`: a cell per column, each wrapped in the containment, the content
// handed down from the table's own slots. The Actions cell stops the click: in a clickable row
// a button there must not also open the row — a Ctrl-click would open two tabs.
defineProps<{
  columns: GridColumn[]
  rowClass: string
}>()
const emit = defineEmits<{ click: [event: MouseEvent] }>()
</script>

<template>
  <div :class="cn('flex', rowClass)" @click="emit('click', $event)">
    <span
      v-for="column in columns"
      :key="column.key"
      :class="cellClass(column)"
      :style="cellStyle(column)"
      @click="column.key === ACTIONS_KEY && $event.stopPropagation()"
    >
      <slot :name="column.key" />
    </span>
  </div>
</template>
```

If `pnpm scan` flags the expression in `@click` as a side effect inside an expression, replace it with a method `stopOnActions(column, event)` in the script.

- [ ] **Step 2: `GridTable.vue` — the header band, the rows, the virtual list**

```vue
<!-- ui/src/components/ui/grid-table/GridTable.vue -->
<script setup lang="ts" generic="T">
import { computed, ref } from 'vue'
import { VirtualRows } from '@/components/ui/virtual'
import { useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import type { ScrollOffset } from '@/lib/scale/scrollOffset'
import {
  ACTIONS_KEY,
  RowHeight,
  cellClass,
  cellStyle,
  rowHeightClass,
  rowHeightPx,
  withActions,
} from '@/lib/table/gridColumn'
import type { GridColumn } from '@/lib/table/gridColumn'
import GridRow from './GridRow.vue'

// The one way a list is drawn as a table: the leather band pinned to the page box's top, a row
// per item, stripes, hover, and a cell per declared column that keeps to its track whatever it
// holds. The columns are the screen's (`lib/table/columns.ts`); what is in a cell is the
// screen's too, through `#cell-<key>`. Every table ends with Actions, filled by `#actions`.
const props = withDefaults(
  defineProps<{
    columns: readonly GridColumn[]
    rows: T[]
    rowKey: (row: T) => string | number
    rowHeight?: RowHeight
    virtual?: boolean
    clickable?: boolean
    offset?: ScrollOffset | null
  }>(),
  {
    rowHeight: RowHeight.Wide,
    virtual: false,
    clickable: false,
    offset: null,
  },
)
const emit = defineEmits<{
  rowClick: [row: T, event: MouseEvent]
  offsetChange: [ScrollOffset]
}>()
defineSlots<
  {
    actions(props: { row: T }): unknown
  } & {
    [cell: `cell-${string}`]: (props: { row: T; index: number }) => unknown
  } & {
    [head: `head-${string}`]: (props: Record<string, never>) => unknown
  }
>()
const { t } = useMessages()

const all = computed(() => withActions(props.columns))

const rowClass = (index: number): string =>
  cn(
    'border-b border-hairline hover:bg-row-hover',
    rowHeightClass[props.rowHeight],
    index % 2 === 1 && 'bg-row-alt',
    props.clickable && 'cursor-pointer',
  )

const click = (row: T, event: MouseEvent): void => {
  if (props.clickable) emit('rowClick', row, event)
}

const slotName = (key: string): string =>
  key === ACTIONS_KEY ? ACTIONS_KEY : `cell-${key}`

// The find bar moves to a row by index; only the virtualizer can make that row exist.
const list = ref<{ scrollToIndex: (index: number) => void } | null>(null)
defineExpose({
  scrollToIndex: (index: number) => list.value?.scrollToIndex(index),
})
</script>

<template>
  <div class="flex flex-col">
    <div
      class="sticky top-0 z-raised-header flex border-b border-hairline bg-band text-label text-band-foreground"
    >
      <span
        v-for="column in all"
        :key="column.key"
        :class="cn(cellClass(column), 'py-1.5')"
        :style="cellStyle(column)"
      >
        <slot :name="`head-${column.key}`">
          <span v-if="column.header" class="truncate">{{
            t(column.header)
          }}</span>
        </slot>
      </span>
    </div>
    <VirtualRows
      v-if="virtual"
      ref="list"
      v-slot="{ visible }"
      :rows="rows"
      :row-px="rowHeightPx[rowHeight]"
      :offset="offset"
      @offset-change="emit('offsetChange', $event)"
    >
      <GridRow
        v-for="{ index, style, row } in visible"
        :key="rowKey(row)"
        :style="style"
        :columns="all"
        :row-class="
          cn(rowClass(index), 'absolute inset-x-0 top-0 translate-y-(--row-start)')
        "
        @click="click(row, $event)"
      >
        <template v-for="column in all" :key="column.key" #[column.key]>
          <slot :name="slotName(column.key)" :row="row" :index="index" />
        </template>
      </GridRow>
    </VirtualRows>
    <template v-else>
      <GridRow
        v-for="(row, index) in rows"
        :key="rowKey(row)"
        :columns="all"
        :row-class="rowClass(index)"
        @click="click(row, $event)"
      >
        <template v-for="column in all" :key="column.key" #[column.key]>
          <slot :name="slotName(column.key)" :row="row" :index="index" />
        </template>
      </GridRow>
    </template>
  </div>
</template>
```

`index.ts`:

```ts
export { default as GridTable } from './GridTable.vue'
```

- [ ] **Step 3: Check whether `defineSlots` enforces `#actions`**

Run: `pnpm typecheck`
Expected: no errors in the new files. Then, temporarily, mount `<GridTable :columns="[]" :rows="[]" :row-key="() => 0" />` with no `#actions` slot in any `.vue` under `ui/src/kit/` and run `pnpm typecheck` again. If vue-tsc reports the missing slot, the spec's "does not typecheck" holds; remove the probe. **If it stays silent**, remove the probe and change the spec's sentence to "a table that does not fill it shows an empty Actions cell, held by review" in the same commit — and add that line to the review-held list in `CLAUDE.md` (*What is checked, and what holds only because it is read*). Report which it was.

- [ ] **Step 4: `QueueActionButton.vue`**

```vue
<!-- ui/src/components/plan/QueueActionButton.vue -->
<script setup lang="ts">
import { ListMinusIcon, ListPlusIcon } from '@lucide/vue'
import type { Component } from 'vue'
import { computed } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { useQueueOffer } from '@/composables/useQueueOffer'
import { useMessages } from '@/i18n'
import type { Message } from '@/i18n/message'
import { QueueAction, queueAction } from '@/lib/plan/queueAction'
import type { QueueTarget } from '@/lib/plan/queueAction'

// The Actions button of a list row: adds the row's achievement to the Plan's queue, or takes
// it out when you asked for it. Disabled says nothing about why — the label says what a click
// does, and a disabled one does nothing.
const props = defineProps<{ target: QueueTarget | null }>()
const { t } = useMessages()
const { queue, membership, canWrite } = useQueueOffer()

const action = computed(() =>
  queueAction(props.target, membership.value, canWrite.value),
)

const label: Record<QueueAction, Message> = {
  [QueueAction.Add]: 'queue.add',
  [QueueAction.Remove]: 'queue.remove',
  [QueueAction.Unavailable]: 'queue.add',
}
const icon: Record<QueueAction, Component> = {
  [QueueAction.Add]: ListPlusIcon,
  [QueueAction.Remove]: ListMinusIcon,
  [QueueAction.Unavailable]: ListPlusIcon,
}

const act = async (): Promise<void> => {
  const target = props.target
  if (target === null) return
  if (action.value === QueueAction.Add) await queue.add(target.achievement)
  if (action.value === QueueAction.Remove)
    await queue.remove(target.achievement)
}
</script>

<template>
  <Tooltip :disabled="action === QueueAction.Unavailable">
    <TooltipTrigger as-child>
      <Button
        :variant="ButtonVariant.Ghost"
        :size="ButtonSize.IconCompact"
        :aria-label="t(label[action])"
        :disabled="action === QueueAction.Unavailable || queue.busy"
        @click="act"
      >
        <component :is="icon[action]" />
      </Button>
    </TooltipTrigger>
    <TooltipContent>{{ t(label[action]) }}</TooltipContent>
  </Tooltip>
</template>
```

Check `ui/src/components/ui/tooltip/index.ts` exports these three names (KpiTile uses them) and that `Tooltip` takes `disabled`; adjust the import if the names differ.

- [ ] **Step 5: Run the checks**

Run: `pnpm typecheck && pnpm scan && pnpm lint`
Expected: clean.

- [ ] **Step 6: Commit**

```bash
git add ui/src/components/ui/grid-table ui/src/components/plan/QueueActionButton.vue
git commit -m "feat(ui): GridTable, the one way a list is drawn as a table, with its Actions button"
```

---

### Task 4: The columns' file and its contract test

**Files:**
- Create: `ui/src/lib/table/columns.ts`
- Test: `ui/src/lib/table/columns.test.ts`

**Interfaces:**
- Consumes: Task 1.
- Produces: `TableColumns: Record<ListTable, readonly GridColumn[]>` and `ListTable` (as-const, one member per migrated table); `challengeColumns` (this task). Later tasks add `unlockColumns`, `collectionColumns`, `liveColumns`, `runsColumns`, and `wikiColumns(facts, hasId)` and register each in `TableColumns` (the wiki's set is registered with a representative category, see Task 10).

- [ ] **Step 1: Write the failing test**

```ts
// ui/src/lib/table/columns.test.ts
import { describe, expect, it } from 'vitest'
import spacing from '@/assets/theme/spacing.css?raw'
import { ListTable, TableColumns } from './columns'
import { ACTIONS_KEY, ColumnFold, ColumnWidthKind } from './gridColumn'

// The tables that keep every column at compact, each with its reason.
const KEEPS_EVERY_COLUMN: ListTable[] = [
  // Live sits beside the run being played and is read at full width; its four columns are what
  // the row is.
]

describe('every list table', () => {
  for (const [name, columns] of Object.entries(TableColumns)) {
    describe(name, () => {
      it('names each column once, and never Actions — the table adds it', () => {
        const keys = columns.map((c) => c.key)
        expect(new Set(keys).size).toBe(keys.length)
        expect(keys).not.toContain(ACTIONS_KEY)
      })

      it('sizes every fixed column with a token that exists', () => {
        for (const column of columns) {
          if (column.width.kind !== ColumnWidthKind.Fixed) continue
          const token = column.width.class.replace(/^w-/, '--spacing-')
          expect(spacing).toContain(`${token}:`)
        }
      })

      it('folds something at compact, unless it is declared to keep all', () => {
        if (KEEPS_EVERY_COLUMN.includes(name as ListTable)) return
        expect(columns.some((c) => c.fold === ColumnFold.Compact)).toBe(true)
      })
    })
  }

  it('is a table with at least one column to look at', () => {
    expect(Object.keys(TableColumns).length).toBeGreaterThan(0)
  })
})
```

- [ ] **Step 2: Run it to see it fail**

Run: `pnpm --filter ui exec vitest run src/lib/table/columns.test.ts`
Expected: FAIL — cannot resolve `./columns`.

- [ ] **Step 3: Write `columns.ts` with Challenges' columns**

```ts
// ui/src/lib/table/columns.ts
import {
  ColumnAlign,
  ColumnFold,
  fixed,
  grow,
} from './gridColumn'
import type { GridColumn } from './gridColumn'

// Each list table's columns, in the order they are drawn. What folds at compact is 3.13b's rule:
// who the row is and how it is doing stay; what explains it, and what is derived from it, go.
// The weights are the `fr` of the grid templates these replaced.

export const ListTable = {
  Challenges: 'challenges',
} as const
export type ListTable = (typeof ListTable)[keyof typeof ListTable]

// The number, the name, the character it forces, the goal, the state.
export const challengeColumns: readonly GridColumn[] = [
  {
    key: 'number',
    header: 'challenges.columns.number',
    width: fixed('w-challenge-number'),
    fold: ColumnFold.Never,
    align: ColumnAlign.End,
  },
  {
    key: 'name',
    header: 'challenges.columns.challenge',
    width: grow(1.4),
    fold: ColumnFold.Never,
    align: ColumnAlign.Start,
  },
  {
    key: 'character',
    header: 'challenges.columns.character',
    width: grow(1),
    fold: ColumnFold.Compact,
    align: ColumnAlign.Start,
  },
  {
    key: 'goal',
    header: 'challenges.columns.goal',
    width: grow(1),
    fold: ColumnFold.Compact,
    align: ColumnAlign.Start,
  },
  {
    key: 'state',
    header: 'challenges.columns.state',
    width: fixed('w-challenge-state'),
    fold: ColumnFold.Never,
    align: ColumnAlign.Start,
  },
]

export const TableColumns: Record<ListTable, readonly GridColumn[]> = {
  [ListTable.Challenges]: challengeColumns,
}
```

- [ ] **Step 4: Run the test**

Run: `pnpm --filter ui exec vitest run src/lib/table`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add ui/src/lib/table/columns.ts ui/src/lib/table/columns.test.ts
git commit -m "feat(ui): the list tables' columns, held to their tokens by a test"
```

---

### Task 5: Challenges on `GridTable` — the bug

**Files:**
- Modify: `ui/src/lib/challenges/challengeQueue.ts`, `ui/src/lib/challenges/challengeQueue.test.ts`
- Modify: `ui/src/screens/challenges/ChallengesTable.vue`
- Delete: `ui/src/screens/challenges/ChallengeRow.vue`
- Create: `ui/src/screens/challenges/ChallengeNameCell.vue`, `ui/src/screens/challenges/ChallengeGoalCell.vue`
- Modify: `ui/src/screens/ChallengesScreen.vue`
- Modify: `ui/src/kit/sections/app/WidthsSection.vue`
- Modify: `ui/src/assets/utilities.css` (remove `grid-cols-challenges`, `grid-cols-challenges-narrow`), `ui/src/assets/theme/spacing.css` (remove `--spacing-challenge-queue`)
- Modify: `ui/src/lib/design/tables.ts` (remove `Challenges` from `FoldingTable` and `TableTracks`)

**Interfaces:**
- Consumes: `GridTable`, `QueueActionButton`, `challengeColumns`, `QueueTarget`, `useQueueOffer().queued`.
- Produces: `challengeQueueTarget(row: ChallengeRow): QueueTarget | null` replacing `queueableReward`.

- [ ] **Step 1: Rewrite the queue rule's test first**

Open `ui/src/lib/challenges/challengeQueue.test.ts`. Its cases describe `queueableReward(row, queued)`. Rewrite them for the new rule, which no longer looks at the queue (the button does, through `queueAction`):

```ts
import { describe, expect, it } from 'vitest'
import { challengeQueueTarget } from './challengeQueue'
// keep the file's existing row builder; it is named `row` or similar — reuse it as is

describe('challengeQueueTarget', () => {
  it('is the first reward, with whether it is earned', () => {
    // a row whose rewards are [{ achievement: 10, done: false }, { achievement: 11, done: false }]
    expect(challengeQueueTarget(rowWith([reward(10, false), reward(11, false)]))).toEqual({
      achievement: 10,
      done: false,
    })
  })

  it('is earned when the first reward is', () => {
    expect(challengeQueueTarget(rowWith([reward(10, true)]))).toEqual({
      achievement: 10,
      done: true,
    })
  })

  it('reads unread as not earned', () => {
    expect(challengeQueueTarget(rowWith([reward(10, null)]))).toEqual({
      achievement: 10,
      done: false,
    })
  })

  it('is nothing when the challenge unlocks nothing', () => {
    expect(challengeQueueTarget(rowWith([]))).toBeNull()
  })
})
```

`rowWith` and `reward` are the file's existing builders (read the file first and use its names; if it has only one builder, write these two on top of it). Keep any existing case that is still meaningful under the new rule; delete only the ones about `queued`, which `queueAction.test.ts` now covers.

- [ ] **Step 2: Run it to see it fail**

Run: `pnpm --filter ui exec vitest run src/lib/challenges/challengeQueue.test.ts`
Expected: FAIL — `challengeQueueTarget` is not exported.

- [ ] **Step 3: Rewrite the rule**

```ts
// ui/src/lib/challenges/challengeQueue.ts
import type { ChallengeRow } from '@/lib/ipc/types'
import type { QueueTarget } from '@/lib/plan/queueAction'

/**
 * The achievement a challenge's Actions button puts in the queue: its first reward — the rest
 * are counted beside it. Unread is not earned: `done` is `null` when section 1 was not read,
 * and the button stays offered.
 */
export const challengeQueueTarget = (row: ChallengeRow): QueueTarget | null => {
  const first = row.rewards[0]
  if (first === undefined) return null
  return { achievement: first.achievement, done: first.done === true }
}
```

Run: `pnpm --filter ui exec vitest run src/lib/challenges/challengeQueue.test.ts`
Expected: PASS.

- [ ] **Step 4: Move the two cells with logic into components**

`ChallengeNameCell.vue` — the name (a link when it has a page) and the reward line; its markup is `ChallengeRow.vue`'s second `<span>` today, made a root:

```vue
<script setup lang="ts">
import { computed } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import type { ChallengeRow, Target } from '@/lib/ipc/types'

// A challenge's name and what it rewards. The first reward is the one the Actions button
// queues; the rest are counted, the way Unlock counts what a node unlocks beyond the first.
const props = defineProps<{ row: ChallengeRow; queued: boolean }>()
const emit = defineEmits<{ navigate: [target: Target, newTab: boolean] }>()
const { t } = useMessages()

const reward = computed(() => props.row.rewards[0] ?? null)
const more = computed(() => Math.max(0, props.row.rewards.length - 1))
const done = computed(() => props.row.state.kind === 'done')
</script>

<template>
  <span class="flex min-w-0 flex-col">
    <Button
      v-if="row.page"
      :variant="ButtonVariant.Link"
      :size="ButtonSize.Compact"
      :class="cn('justify-start truncate', done && 'text-subtle-foreground')"
      @click="row.page && emit('navigate', row.page, $event.ctrlKey)"
      >{{ row.name }}</Button
    >
    <span
      v-else
      :class="
        cn('truncate text-row', done ? 'text-subtle-foreground' : 'text-foreground')
      "
      >{{ row.name }}</span
    >
    <span class="min-w-0 truncate text-micro text-faint-foreground">
      <template v-if="reward"
        >{{ reward.text ?? t('challenges.unnamedReward', { id: reward.achievement })
        }}<template v-if="more > 0"> +{{ more }}</template></template
      >
      <template v-else>{{ t('challenges.unlocksNothing') }}</template>
      <template v-if="queued"> · {{ t('queue.inQueue') }}</template>
    </span>
  </span>
</template>
```

`ChallengeGoalCell.vue` — `ChallengeRow.vue`'s fourth `<span>`, made a root, `@max-compact/page:hidden` and `px-2` removed (the column carries both):

```vue
<script setup lang="ts">
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import WikiInline from '@/components/wiki/WikiInline.vue'
import { useMessages } from '@/i18n'
import type { ChallengeRow, Target } from '@/lib/ipc/types'

defineProps<{ row: ChallengeRow }>()
const emit = defineEmits<{ navigate: [target: Target, newTab: boolean] }>()
const { t } = useMessages()
</script>

<template>
  <span class="flex min-w-0 items-center gap-1.5">
    <!-- `WikiInline` is a fragment and takes no class of its own: this span sets the measure
         and the truncation, the way every other caller wraps it. -->
    <span v-if="row.goal" class="min-w-0 truncate text-caption">
      <WikiInline
        :inline="row.goal"
        @navigate="(target, newTab) => emit('navigate', target, newTab)"
      />
    </span>
    <EmptyValue v-else>{{ t('challenges.noCondition') }}</EmptyValue>
    <span v-if="row.blindfolded" class="shrink-0 text-label text-subtle-foreground">{{
      t('challenges.blindfolded')
    }}</span>
  </span>
</template>
```

If `pnpm scan` objects to `row.page && emit(…)` in the name cell's `@click` (a side effect in an expression), move it to a function `open(event)` in the script.

- [ ] **Step 5: Rewrite `ChallengesTable.vue`**

```vue
<script setup lang="ts">
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import QueueActionButton from '@/components/plan/QueueActionButton.vue'
import { GridTable } from '@/components/ui/grid-table'
import { useQueueOffer } from '@/composables/useQueueOffer'
import { useMessages } from '@/i18n'
import { challengeQueueTarget } from '@/lib/challenges/challengeQueue'
import type { ChallengeRow, Target } from '@/lib/ipc/types'
import { challengeColumns } from '@/lib/table/columns'
import ChallengeGoalCell from './ChallengeGoalCell.vue'
import ChallengeNameCell from './ChallengeNameCell.vue'
import ChallengeStateBadge from './ChallengeStateBadge.vue'

// Forty-five rows: no virtual list. `VirtualRows` exists for 733 items and 642 achievements,
// and a list this short pays its machinery for nothing.
defineProps<{ rows: ChallengeRow[] }>()
const emit = defineEmits<{ navigate: [target: Target, newTab: boolean] }>()
const { t } = useMessages()
const { queued } = useQueueOffer()

const isQueued = (row: ChallengeRow): boolean =>
  row.rewards.some((r) => queued.value.has(r.achievement))
</script>

<template>
  <GridTable
    :columns="challengeColumns"
    :rows="rows"
    :row-key="(row) => row.number"
  >
    <template #cell-number="{ row }">
      <span class="text-label text-subtle-foreground tabular-nums">{{
        row.number
      }}</span>
    </template>
    <template #cell-name="{ row }">
      <ChallengeNameCell
        :row="row"
        :queued="isQueued(row)"
        @navigate="(target, newTab) => emit('navigate', target, newTab)"
      />
    </template>
    <template #cell-character="{ row }">
      <!-- A challenge the wiki has no page for says nothing here: it must not read as "any
           character", which is a fact about the game nobody read. -->
      <span v-if="row.characterName" class="truncate text-caption text-foreground">{{
        row.characterName
      }}</span>
      <EmptyValue v-else>{{ t('challenges.noCondition') }}</EmptyValue>
    </template>
    <template #cell-goal="{ row }">
      <ChallengeGoalCell
        :row="row"
        @navigate="(target, newTab) => emit('navigate', target, newTab)"
      />
    </template>
    <template #cell-state="{ row }">
      <ChallengeStateBadge :state="row.state" />
    </template>
    <template #actions="{ row }">
      <QueueActionButton :target="challengeQueueTarget(row)" />
    </template>
  </GridTable>
</template>
```

Delete `ui/src/screens/challenges/ChallengeRow.vue`.

- [ ] **Step 6: Update the screen**

In `ui/src/screens/ChallengesScreen.vue`, the `<ChallengesTable>` call loses `:queued`, `:can-write`, `:busy` and `@add`:

```vue
          <ChallengesTable :rows="…the same rows expression…" @navigate="…the same handler…" />
```

Remove `queued` and `canWrite` from the `useQueueOffer()` destructuring if nothing else in the file reads them (keep `queue`: `queue.load()` and `QueueError` still use it). Grep the file for `queueableReward` and `canWrite` afterwards.

- [ ] **Step 7: Remove what is now dead**

- `utilities.css`: delete `@utility grid-cols-challenges` and `@utility grid-cols-challenges-narrow` with their comments.
- `spacing.css`: delete `--spacing-challenge-queue`.
- `lib/design/tables.ts`: delete `Challenges` from `FoldingTable` and its `TableTracks` entry.
- Grep `ui/src` for `queueableReward`, `ChallengeRow.vue`, `grid-cols-challenges`, `challenge-queue`: no hit may remain (comments included).

- [ ] **Step 8: The Kit — the case this card was opened for**

In `ui/src/kit/sections/app/WidthsSection.vue`: the `<ChallengesTable>` call becomes `<ChallengesTable :rows="challenges" />`, and the `challenges` fixture gains a row whose state is blocked by twelve gates. Read the `ChallengeStateView` blocked variant in `lib/ipc/types.ts` (it carries `missing: …[]`) and build twelve entries the way the fixture builds its existing blocked row. Update the comment at the top of the file: it names "the four pairs listed in `lib/design/tables.ts`" — make it "one fixture per list table".

- [ ] **Step 9: Run the checks**

Run: `pnpm --filter ui exec vitest run src/lib && pnpm typecheck && pnpm scan && pnpm lint`
Expected: clean. (`tables.test.ts` still passes: it iterates what `FoldingTable` lists.)

- [ ] **Step 10: Look at it**

Start the Kit and the app in the browser on a port of your own: `pnpm ui:dev -- --port 1430` (background; note the PID). Open `http://localhost:1430/#kit`, section *Larghezze*, Challenges at all three widths:
- the twelve-gates pill sits inside its column on two lines at 500, nowhere past the Actions column;
- the header band is the leather one, "Azioni" fits its cell;
- Tab onto a challenge's name link and onto an Actions button: the focus ring shows whole.
Then the Challenges screen itself with the fixtures (`/#/challenges` or the route `ui:dev` uses), at 640 × 480 with the browser's device toolbar, with the console snippet from `docs/frontend-conventions.md` §"Nothing runs past the page at 640 × 480": it prints nothing. Stop the server by its PID after checking its command line.

- [ ] **Step 11: Commit**

```bash
git add ui/src/lib/challenges/challengeQueue.ts ui/src/lib/challenges/challengeQueue.test.ts ui/src/screens/challenges ui/src/screens/ChallengesScreen.vue ui/src/kit/sections/app/WidthsSection.vue ui/src/assets/utilities.css ui/src/assets/theme/spacing.css ui/src/lib/design/tables.ts
git commit -m "fix(ui): Challenges on GridTable, its state pill stays in its column"
```

(`git add` on the `screens/challenges` directory stages the deletion of `ChallengeRow.vue` too; check `git status` shows exactly these paths.)

---

### Task 6: Unlock on `GridTable`

**Files:**
- Modify: `ui/src/lib/table/columns.ts`, `ui/src/lib/plan/queueRows.ts` (+ its test if one covers `canQueue`)
- Modify: `ui/src/screens/unlock/UnlockTable.vue`; Delete: `ui/src/screens/unlock/UnlockRow.vue`
- Create: `ui/src/screens/unlock/UnlockNameCell.vue`, `ui/src/screens/unlock/UnlockUnlocksCell.vue`
- Modify: `ui/src/screens/UnlockScreen.vue`, `ui/src/kit/sections/app/WidthsSection.vue`
- Modify: `ui/src/assets/utilities.css`, `ui/src/assets/theme/spacing.css`, `ui/src/lib/design/tables.ts`

**Interfaces:**
- Produces: `unlockColumns`; `nodeQueueTarget(node: UnlockNode): QueueTarget | null` in `lib/plan/queueRows.ts`.

- [ ] **Step 1: Test the node's queue target**

Add to the test file that covers `lib/plan/queueRows.ts` (create `queueRows.test.ts` beside it if there is none; reuse the node builder from `queueAction.test.ts` by copying it — it is six lines and a shared fixture would be a third file for one builder):

```ts
describe('nodeQueueTarget', () => {
  it('is the known achievement, with whether it is done', () => {
    expect(nodeQueueTarget(node(7, false))).toEqual({ achievement: 7, done: false })
    expect(nodeQueueTarget(node(7, true))).toEqual({ achievement: 7, done: true })
  })

  it('is nothing for an achievement the catalog does not know', () => {
    expect(nodeQueueTarget(unknownNode(9))).toBeNull()
  })
})
```

`unknownNode` builds an `UnlockNode` whose `achievement.kind` is the unknown variant — read `AchievementRef` in `lib/ipc/types.ts` for its exact shape.

- [ ] **Step 2: Run it to see it fail**, then **write it** in `queueRows.ts`:

```ts
// The achievement a node's Actions button puts in the queue; an unknown one has no id to queue.
export const nodeQueueTarget = (node: UnlockNode): QueueTarget | null => {
  const id = knownId(node)
  return id === null ? null : { achievement: id, done: node.done }
}
```

(import `type QueueTarget` from `./queueAction`). Run: `pnpm --filter ui exec vitest run src/lib/plan` → PASS. `canQueue` and `isQueued` stay if anything outside Unlock still calls them — grep; if Unlock was the only caller, delete them and their tests in this task.

- [ ] **Step 3: Add the columns** to `columns.ts`, and `Unlock: 'unlock'` to `ListTable` with its `TableColumns` entry:

```ts
// The drawing, the achievement, what it unlocks, its condition, the state, the fan-out.
export const unlockColumns: readonly GridColumn[] = [
  { key: 'art', header: null, width: fixed('w-unlock-art'), fold: ColumnFold.Never, align: ColumnAlign.Center },
  { key: 'achievement', header: 'unlock.columns.achievement', width: grow(1.5), fold: ColumnFold.Never, align: ColumnAlign.Start },
  { key: 'unlocks', header: 'unlock.columns.unlocks', width: grow(1.1), fold: ColumnFold.Compact, align: ColumnAlign.Start },
  { key: 'condition', header: 'unlock.columns.condition', width: grow(1), fold: ColumnFold.Compact, align: ColumnAlign.Start },
  { key: 'state', header: 'unlock.columns.state', width: fixed('w-unlock-state'), fold: ColumnFold.Never, align: ColumnAlign.Start },
  { key: 'fanOut', header: 'unlock.columns.fanOut', width: fixed('w-unlock-fan'), fold: ColumnFold.Compact, align: ColumnAlign.End },
]
```

(Prettier will break each object over lines; write them that way.) Run `pnpm --filter ui exec vitest run src/lib/table` → PASS.

- [ ] **Step 4: The cells with logic**

`UnlockNameCell.vue` (props `node: UnlockNode`, `queued: boolean`): `UnlockRow.vue`'s second `<span>` made a root, using `knownAchievement`, `nodeNumber`, `t('graph.unknownAchievement')`, `t('graph.slot', …)` and `· {{ t('queue.inQueue') }}` exactly as it does today.

`UnlockUnlocksCell.vue` (prop `node: UnlockNode`): `UnlockRow.vue`'s third `<span>` made a root, minus `px-2` and `@max-compact/page:hidden`: the `PixelSprite` of the first unlock when it is an item, `targetName(t, first)`, `+{{ more }}`, and `EmptyValue` with `t('unlock.unlocksNothing')`. Drop the comment about "paired with a track dropped from `grid-cols-unlock-narrow`" — it is false now; the fold is the column's.

- [ ] **Step 5: Rewrite `UnlockTable.vue`**

```vue
<script setup lang="ts">
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import AchievementArt from '@/components/graph/AchievementArt.vue'
import NodeStateBadge from '@/components/graph/NodeStateBadge.vue'
import { ArtSize } from '@/components/graph/artSize'
import QueueActionButton from '@/components/plan/QueueActionButton.vue'
import { GridTable } from '@/components/ui/grid-table'
import { useQueueOffer } from '@/composables/useQueueOffer'
import { useMessages } from '@/i18n'
import { knownAchievement, nodeNumber } from '@/lib/graph/achievementNode'
import type { UnlockNode } from '@/lib/ipc/types'
import { isQueued, nodeQueueTarget } from '@/lib/plan/queueRows'
import { unlockColumns } from '@/lib/table/columns'
import UnlockNameCell from './UnlockNameCell.vue'
import UnlockUnlocksCell from './UnlockUnlocksCell.vue'

// The screen scrolls as a page (`PageScroll`): the rows virtualize against it, and the
// columns' header pins to its top while the list goes by under it.
defineProps<{ nodes: UnlockNode[] }>()
const { t } = useMessages()
const { queued } = useQueueOffer()
</script>

<template>
  <GridTable
    :columns="unlockColumns"
    :rows="nodes"
    :row-key="nodeNumber"
    virtual
  >
    <template #cell-art="{ row }">
      <AchievementArt
        :url="knownAchievement(row)?.iconUrl ?? null"
        :size="ArtSize.Thumb"
      />
    </template>
    <template #cell-achievement="{ row }">
      <UnlockNameCell :node="row" :queued="isQueued(row, queued)" />
    </template>
    <template #cell-unlocks="{ row }">
      <UnlockUnlocksCell :node="row" />
    </template>
    <template #cell-condition="{ row }">
      <span
        v-if="knownAchievement(row)?.condition"
        class="truncate text-caption text-foreground-soft"
        >{{ knownAchievement(row)?.condition }}</span
      >
      <EmptyValue v-else>{{ t('unlock.noCondition') }}</EmptyValue>
    </template>
    <template #cell-state="{ row }">
      <NodeStateBadge :node="row" />
    </template>
    <template #cell-fanOut="{ row }">
      <span class="text-row text-foreground tabular-nums">{{
        row.done ? '—' : row.graph.fanOut
      }}</span>
    </template>
    <template #actions="{ row }">
      <QueueActionButton :target="nodeQueueTarget(row)" />
    </template>
  </GridTable>
</template>
```

Delete `UnlockRow.vue`. If `isQueued` was deleted in Step 2, inline `queued.has(nodeNumber(row))` instead.

- [ ] **Step 6: The screen and the Kit**

`UnlockScreen.vue`: `<UnlockTable :nodes="…">` — drop `:queued`, `:can-write`, `:busy`, `@add`, and the destructured names nothing reads any more. `WidthsSection.vue`: `<UnlockTable :nodes="nodes" />`; delete the `queued` constant if unused.

- [ ] **Step 7: Dead code**: delete `@utility grid-cols-unlock` and `-narrow` (with their comments), `--spacing-unlock-queue`, `Unlock` from `tables.ts`. Grep `ui/src` and `docs/` for `UnlockRow`, `grid-cols-unlock`, `unlock-queue`. In `docs/`, a hit inside `docs/completed/` or a dated spec/plan is history and stays; a hit in a living document (`frontend-conventions.md`, `architecture.md`) is fixed in this commit.

- [ ] **Step 8: Checks**: `pnpm --filter ui exec vitest run src/lib && pnpm typecheck && pnpm scan && pnpm lint` → clean. In the Kit (Step 10 of Task 5's procedure): Unlock at three widths, a long name truncates, the state pill stays inside.

- [ ] **Step 9: Commit**

```bash
git add ui/src/lib/table/columns.ts ui/src/lib/plan ui/src/screens/unlock ui/src/screens/UnlockScreen.vue ui/src/kit/sections/app/WidthsSection.vue ui/src/assets/utilities.css ui/src/assets/theme/spacing.css ui/src/lib/design/tables.ts
git commit -m "refactor(ui): Unlock on GridTable"
```

---

### Task 7: Collection on `GridTable`, with its Actions

**Files:**
- Create: `ui/src/lib/collection/collectionQueue.ts`, `ui/src/lib/collection/collectionQueue.test.ts`
- Modify: `ui/src/lib/table/columns.ts`
- Modify: `ui/src/screens/collection/CollectionTable.vue`; Delete: `ui/src/screens/collection/CollectionRow.vue`
- Create: `ui/src/screens/collection/CollectionNameCell.vue`, `ui/src/screens/collection/CollectionPoolsCell.vue`, `ui/src/screens/collection/CollectionStateCell.vue`
- Modify: `ui/src/screens/CollectionScreen.vue`
- Modify: `ui/src/assets/utilities.css`, `ui/src/lib/design/tables.ts`

**Interfaces:**
- Produces: `itemQueueTarget(item: CollectionItem): QueueTarget | null`; `collectionColumns`.

- [ ] **Step 1: The failing test**

```ts
// ui/src/lib/collection/collectionQueue.test.ts
import { describe, expect, it } from 'vitest'
import type { CollectionItem, LockView } from '@/lib/ipc/types'
import { ItemKindView } from '@/lib/ipc/types'
import { itemQueueTarget } from './collectionQueue'

const item = (lock: LockView): CollectionItem => ({
  id: 1,
  kind: ItemKindView.Passive,
  name: 'x',
  iconUrl: null,
  quality: null,
  pools: [],
  origin: null,
  inCollection: null,
  lock,
})

describe('itemQueueTarget', () => {
  it('is the achievement that unlocks a locked item', () => {
    expect(
      itemQueueTarget(item({ kind: 'locked', achievement: 5, text: null, page: null })),
    ).toEqual({ achievement: 5, done: false })
  })

  it('is nothing for an item already unlocked', () => {
    expect(
      itemQueueTarget(item({ kind: 'unlocked', achievement: 5, text: null, page: null })),
    ).toBeNull()
  })

  it('is nothing for an item nobody has to unlock', () => {
    expect(itemQueueTarget(item({ kind: 'free' }))).toBeNull()
  })
})
```

Read `LockView` in `lib/ipc/types.ts` first: if it has more variants than these three, add a case for each, and if the `locked` variant carries more fields, fill them.

- [ ] **Step 2: Run to fail; then write**

```ts
// ui/src/lib/collection/collectionQueue.ts
import type { CollectionItem } from '@/lib/ipc/types'
import type { QueueTarget } from '@/lib/plan/queueAction'
import { assertNever } from '@/lib/assertNever'

// What an item's Actions button puts in the Plan's queue: the achievement that unlocks it, while
// it is still locked. An unlocked or free item has nothing left to earn.
export const itemQueueTarget = (item: CollectionItem): QueueTarget | null => {
  const lock = item.lock
  switch (lock.kind) {
    case 'locked':
      return { achievement: lock.achievement, done: false }
    case 'unlocked':
    case 'free':
      return null
    default:
      return assertNever(lock)
  }
}
```

(A `switch` on a tag may name its literals — that is the scan's allowed shape.) Run → PASS.

- [ ] **Step 3: Columns** — `Collection: 'collection'` in `ListTable`, entry in `TableColumns`:

```ts
// The sprite, the name, the quality, the pools, the origin, the state.
export const collectionColumns: readonly GridColumn[] = [
  { key: 'sprite', header: null, width: fixed('w-collection-sprite'), fold: ColumnFold.Never, align: ColumnAlign.Center },
  { key: 'item', header: 'collection.columns.item', width: grow(1.4), fold: ColumnFold.Never, align: ColumnAlign.Start },
  { key: 'quality', header: 'collection.columns.quality', width: fixed('w-collection-quality'), fold: ColumnFold.Compact, align: ColumnAlign.Start },
  { key: 'pools', header: 'collection.columns.pools', width: grow(1), fold: ColumnFold.Compact, align: ColumnAlign.Start },
  { key: 'origin', header: 'collection.columns.origin', width: fixed('w-collection-origin'), fold: ColumnFold.Compact, align: ColumnAlign.Start },
  { key: 'state', header: 'collection.columns.state', width: fixed('w-collection-state'), fold: ColumnFold.Never, align: ColumnAlign.Start },
]
```

- [ ] **Step 4: The cells**, each made from `CollectionRow.vue`'s markup with `px-2` and `@max-compact/page:hidden` dropped:
  - `CollectionNameCell.vue` (props `item`, `findQuery`, `findCurrent`): the `FindHighlight` name and the `subtitle` line, computing `subtitle` as `CollectionRow.vue` does.
  - `CollectionPoolsCell.vue` (prop `item`): first pool, `+N`, `EmptyValue` `collection.poolNone`.
  - `CollectionStateCell.vue` (prop `item`): the `WhyMenu` + `Badge` with `variant`, `groups` (`lockWhy`) and `itemStateText`, as today.

- [ ] **Step 5: `CollectionTable.vue`**

```vue
<script setup lang="ts">
import { ref } from 'vue'
import QualityPips from '@/components/data-state/QualityPips.vue'
import QueueActionButton from '@/components/plan/QueueActionButton.vue'
import PixelSprite from '@/components/sprite/PixelSprite.vue'
import { GridTable } from '@/components/ui/grid-table'
import { useMessages } from '@/i18n'
import { CollectionFacet } from '@/lib/collection/collectionFacets'
import { collectionFacetValueLabel } from '@/lib/collection/collectionLabels'
import { itemQueueTarget } from '@/lib/collection/collectionQueue'
import type { CollectionItem } from '@/lib/ipc/types'
import { collectionColumns } from '@/lib/table/columns'
import CollectionNameCell from './CollectionNameCell.vue'
import CollectionPoolsCell from './CollectionPoolsCell.vue'
import CollectionStateCell from './CollectionStateCell.vue'

defineProps<{
  items: CollectionItem[]
  /** What the find bar is looking for, so a row can paint it. */
  findQuery: string
  /** The id of the match the bar is standing on. */
  findCurrent: string | null
}>()
const { t } = useMessages()

const origin = (item: CollectionItem): string =>
  item.origin
    ? collectionFacetValueLabel(t, CollectionFacet.Origin, item.origin)
    : '—'

// The find bar hands back an index; moving there is the virtualizer's job, reached through the
// table.
const table = ref<{ scrollToIndex: (index: number) => void } | null>(null)
defineExpose({
  scrollToIndex: (index: number) => table.value?.scrollToIndex(index),
})
</script>

<template>
  <GridTable
    ref="table"
    :columns="collectionColumns"
    :rows="items"
    :row-key="(item) => item.id"
    virtual
  >
    <template #cell-sprite="{ row }">
      <PixelSprite :url="row.iconUrl" placeholder class="size-8 shrink-0" />
    </template>
    <template #cell-item="{ row }">
      <CollectionNameCell
        :item="row"
        :find-query="findQuery"
        :find-current="String(row.id) === findCurrent"
      />
    </template>
    <template #cell-quality="{ row }">
      <QualityPips :quality="row.quality" />
    </template>
    <template #cell-pools="{ row }">
      <CollectionPoolsCell :item="row" />
    </template>
    <template #cell-origin="{ row }">
      <span class="truncate text-caption text-foreground-soft">{{
        origin(row)
      }}</span>
    </template>
    <template #cell-state="{ row }">
      <CollectionStateCell :item="row" />
    </template>
    <template #actions="{ row }">
      <QueueActionButton :target="itemQueueTarget(row)" />
    </template>
  </GridTable>
</template>
```

Delete `CollectionRow.vue`.

- [ ] **Step 6: The screen loads the queue.** In `CollectionScreen.vue`: `const { queue } = useQueueOffer()`, add `queue.load()` beside the screen's own load on mount (the way `ChallengesScreen.vue` does with `Promise.all`), and render `<QueueError v-if="queue.mutationFailed" :error="queue.mutationError" />` where `ChallengesScreen.vue` renders it relative to its table. Kit: `CollectionTable` call unchanged except `:offset` (not a prop — drop it).

- [ ] **Step 7: Dead code**: `@utility grid-cols-collection`, `-narrow`, `Collection` in `tables.ts`; grep `CollectionRow`, `grid-cols-collection`.

- [ ] **Step 8: Checks and a look**: `vitest run src/lib`, `typecheck`, `scan`, `lint` → clean. In `ui:dev` on the Collection screen with fixtures: a locked item's Actions button is **enabled** (the queue loaded — see Review Focus; with `ui:dev`'s fixture transport check `storeAvailable` is true in the queue fixture, and if it is false, that is why every button is disabled — note it and confirm on the real window later); an unlocked one's is disabled; the find bar still jumps to a match.

- [ ] **Step 9: Commit**

```bash
git add ui/src/lib/collection/collectionQueue.ts ui/src/lib/collection/collectionQueue.test.ts ui/src/lib/table/columns.ts ui/src/screens/collection ui/src/screens/CollectionScreen.vue ui/src/kit/sections/app/WidthsSection.vue ui/src/assets/utilities.css ui/src/lib/design/tables.ts
git commit -m "feat(ui): Collection on GridTable, a locked item's achievement can be queued"
```

---

### Task 8: Live's table on `GridTable`, with its Actions

**Files:**
- Modify: `ui/src/lib/live/opensRows.ts` and its test (`opensRows.test.ts` — create if absent)
- Modify: `ui/src/lib/table/columns.ts`, `ui/src/lib/table/columns.test.ts` (Live in `KEEPS_EVERY_COLUMN`)
- Modify: `ui/src/screens/live/LiveOpensTable.vue`, `ui/src/screens/LiveScreen.vue`
- Modify: `ui/src/assets/utilities.css` (remove `grid-cols-live-opens`), `ui/src/kit/sections/app/WidthsSection.vue` (add Live)

**Interfaces:**
- Produces: `OpenRow.achievement: number`; `liveColumns`.

- [ ] **Step 1: Test** — in the opens rows test, a case: every row carries `achievement` equal to `refNumber` of the `LiveAchievement` it came from (build one `LiveOpen` with two achievements, assert `rows.map(r => r.achievement)` equals their numbers). Run → FAIL (`achievement` undefined).

- [ ] **Step 2: Implement** — add `achievement: number` to `OpenRow` (doc comment: *the slot the Actions button queues*), and `achievement: refNumber(achievement),` in the row builder. Run → PASS.

- [ ] **Step 3: Columns** — `Live: 'live'` in `ListTable` and `TableColumns`; add `ListTable.Live` to `KEEPS_EVERY_COLUMN` in the test, with the comment already there.

```ts
// The achievement, the cell it needs, how the game words it, and how much it opens in turn.
export const liveColumns: readonly GridColumn[] = [
  { key: 'achievement', header: 'live.column.achievement', width: grow(1.3), fold: ColumnFold.Never, align: ColumnAlign.Start },
  { key: 'cell', header: 'live.column.cell', width: grow(0.8), fold: ColumnFold.Never, align: ColumnAlign.Start },
  { key: 'condition', header: 'live.column.condition', width: grow(1.4), fold: ColumnFold.Never, align: ColumnAlign.Start },
  { key: 'opens', header: 'live.column.opens', width: fixed('w-live-opens'), fold: ColumnFold.Never, align: ColumnAlign.End },
]
```

- [ ] **Step 4: `LiveOpensTable.vue`**

```vue
<script setup lang="ts">
import { computed } from 'vue'
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import QueueActionButton from '@/components/plan/QueueActionButton.vue'
import EntityChip from '@/components/runs/EntityChip.vue'
import { GridTable } from '@/components/ui/grid-table'
import { useMessages } from '@/i18n'
import type { LiveOpen } from '@/lib/ipc/types'
import { opensRows } from '@/lib/live/opensRows'
import { liveColumns } from '@/lib/table/columns'

// Everything the run could open, in one table instead of a card per cell: the cell is a
// column, so the boss is still said once per row and the rows can be read against each other
// — which a stack of cards cannot do. Live lists only what is missing, so nothing here is done.
const props = defineProps<{ opens: LiveOpen[] }>()
const { t } = useMessages()

const rows = computed(() => opensRows(props.opens, t))
</script>

<template>
  <GridTable :columns="liveColumns" :rows="rows" :row-key="(row) => row.key">
    <template #cell-achievement="{ row }">
      <EntityChip
        :target="row.target"
        :name="row.name"
        :detail="row.condition"
        :icon-url="row.iconUrl"
      />
    </template>
    <template #cell-cell="{ row }">
      <span class="truncate text-row">{{ row.cell }}</span>
    </template>
    <template #cell-condition="{ row }">
      <span v-if="row.condition" class="line-clamp-2 text-label text-subtle-foreground">{{
        row.condition
      }}</span>
      <EmptyValue v-else>{{ t('live.noCondition') }}</EmptyValue>
    </template>
    <template #cell-opens="{ row }">
      <span class="text-label tabular-nums">{{
        row.fanOut > 0 ? row.fanOut : t('live.opensNothingMore')
      }}</span>
    </template>
    <template #actions="{ row }">
      <QueueActionButton :target="{ achievement: row.achievement, done: false }" />
    </template>
  </GridTable>
</template>
```

Note what changes visually: Live's header was not pinned and its rows were `py-1` with free height; on `GridTable` they are 40px rows under a pinned band. Check in Step 7 that the `EntityChip` fits a 40px row; if it does not, stop and report — the fix is a `RowHeight` of its own, which is a spec change.

- [ ] **Step 5: The screen loads the queue** — `LiveScreen.vue`: `useQueueOffer()`, `queue.load()` on mount, `QueueError` above the table, as in Task 7 Step 6.

- [ ] **Step 6: Dead code** — `@utility grid-cols-live-opens`; grep it.

- [ ] **Step 7: Kit and checks** — add a `KitWidths` with `<LiveOpensTable :opens="…" />` to `WidthsSection.vue`, built from the Live fixture in `lib/ipc/fixtures/` (find it with a grep for `LiveOpen`). `vitest run src/lib`, `typecheck`, `scan`, `lint` → clean; the Kit at three widths: rows 40px, chip inside, nothing past the page.

- [ ] **Step 8: Commit**

```bash
git add ui/src/lib/live ui/src/lib/table ui/src/screens/live/LiveOpensTable.vue ui/src/screens/LiveScreen.vue ui/src/kit/sections/app/WidthsSection.vue ui/src/assets/utilities.css
git commit -m "feat(ui): Live's table on GridTable, what a run could open can be queued"
```

---

### Task 9: Runs on `GridTable`, "Open run" in Actions

**Files:**
- Modify: `ui/src/lib/table/columns.ts`
- Modify: `ui/src/screens/runs/RunsTable.vue`; Delete: `ui/src/screens/runs/RunRow.vue`
- Create: `ui/src/screens/runs/RunCharacterCell.vue`, `ui/src/screens/runs/RunOutcomeCell.vue`, `ui/src/screens/runs/RunSourceCell.vue`
- Modify: `ui/src/i18n/messages/it.ts`, `en.ts` (`runs.open`)
- Modify: `ui/src/kit/sections/app/WidthsSection.vue`
- Modify: `ui/src/assets/utilities.css` (remove `grid-cols-runs`, `-narrow`)
- Delete: `ui/src/lib/design/tables.ts`, `ui/src/lib/design/tables.test.ts`

**Interfaces:**
- Produces: `runsColumns`. `RunsTable` keeps its `open: [run: RunView, event: MouseEvent]` emit, so `RunsList.vue` does not change.

- [ ] **Step 1: Columns** — `Runs: 'runs'` in `ListTable`/`TableColumns`:

```ts
// The day, the character, how it ended, the floors, the seed, where it came from.
export const runsColumns: readonly GridColumn[] = [
  { key: 'date', header: 'runs.column.date', width: fixed('w-runs-date'), fold: ColumnFold.Never, align: ColumnAlign.Start },
  { key: 'character', header: 'runs.column.character', width: grow(1), fold: ColumnFold.Never, align: ColumnAlign.Start },
  { key: 'outcome', header: 'runs.column.outcome', width: grow(1.4), fold: ColumnFold.Never, align: ColumnAlign.Start },
  { key: 'floors', header: 'runs.column.floors', width: fixed('w-runs-floors'), fold: ColumnFold.Never, align: ColumnAlign.End },
  { key: 'seed', header: 'runs.column.seed', width: grow(1), fold: ColumnFold.Compact, align: ColumnAlign.Start },
  { key: 'source', header: 'runs.column.source', width: grow(1), fold: ColumnFold.Compact, align: ColumnAlign.Start },
]
```

Run `vitest run src/lib/table` → PASS.

- [ ] **Step 2: The message** — `runs.open`: it `'Apri la run'`, en `'Open run'`, inside the existing `runs` block.

- [ ] **Step 3: The cells** from `RunRow.vue`, each a root, `px-2`/fold classes dropped, its comments kept where still true:
  - `RunCharacterCell.vue` (prop `run`): face `PixelSprite` + name or `EmptyValue` `runs.noCharacter`.
  - `RunOutcomeCell.vue` (prop `run`): the `tone` record, the outcome `Badge`, and `detail` — moved whole from `RunRow.vue`'s script.
  - `RunSourceCell.vue` (prop `run`): source text + online `Badge`.
  `RunDate` is already a component; check whether its root carries `px-2` or grid assumptions and drop them (the cell has the padding now).

- [ ] **Step 4: `RunsTable.vue`**

```vue
<script setup lang="ts">
import { ExternalLinkIcon } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { GridTable } from '@/components/ui/grid-table'
import { useMessages } from '@/i18n'
import type { RunView } from '@/lib/ipc/types'
import { runKey } from '@/lib/runs/runKey'
import { runsColumns } from '@/lib/table/columns'
import RunCharacterCell from './RunCharacterCell.vue'
import RunDate from './RunDate.vue'
import RunOutcomeCell from './RunOutcomeCell.vue'
import RunSourceCell from './RunSourceCell.vue'

// A run is `(source, ordinal)`: that pair is its identity in the archive and so the key here,
// because two sources number their runs from one each. A click anywhere on the row opens it, and
// so does the button in Actions — the one the keyboard reaches.
defineProps<{ runs: RunView[] }>()
const emit = defineEmits<{ open: [run: RunView, event: MouseEvent] }>()
const { t } = useMessages()
</script>

<template>
  <GridTable
    :columns="runsColumns"
    :rows="runs"
    :row-key="runKey"
    virtual
    clickable
    @row-click="(run, event) => emit('open', run, event)"
  >
    <template #cell-date="{ row }"><RunDate :run="row" /></template>
    <template #cell-character="{ row }"><RunCharacterCell :run="row" /></template>
    <template #cell-outcome="{ row }"><RunOutcomeCell :run="row" /></template>
    <template #cell-floors="{ row }">
      <span class="text-row tabular-nums">{{ row.floors }}</span>
    </template>
    <template #cell-seed="{ row }">
      <span class="truncate text-label text-subtle-foreground">{{ row.seedWords }}</span>
    </template>
    <template #cell-source="{ row }"><RunSourceCell :run="row" /></template>
    <template #actions="{ row }">
      <Button
        :variant="ButtonVariant.Ghost"
        :size="ButtonSize.IconCompact"
        :aria-label="t('runs.open')"
        @click="emit('open', row, $event)"
      >
        <ExternalLinkIcon />
      </Button>
    </template>
  </GridTable>
</template>
```

Delete `RunRow.vue`. Kit: `<RunsTable :runs="runs" />` (drop `:selected`, `:offset`, which are not props).

- [ ] **Step 5: The last pair goes, and its record with it** — delete `@utility grid-cols-runs` and `-narrow`; with them the last entry of `lib/design/tables.ts`, so delete `tables.ts` and `tables.test.ts`. Grep `ui/src` for `FoldingTable`, `TableTracks`, `design/tables`, `RunRow`, `grid-cols-runs`, and fix every hit (the Kit's comment, `VirtualRows.vue`'s if it names them).

- [ ] **Step 6: Checks** — `vitest run src/lib`, `typecheck`, `scan`, `lint` → clean. `pnpm scan` still has its *narrow grid template* rule; with no `-narrow` class left it is silent, and Task 11 replaces it.

- [ ] **Step 7: Look** — `ui:dev`, Runs: a click on a row opens the run in the same tab; a Ctrl-click on a row opens **one** new tab; a click on the Actions button opens the run **once** (not twice: the Actions cell stops the row's click); Tab reaches the button and Enter opens.

- [ ] **Step 8: Commit**

```bash
git add ui/src/lib/table ui/src/lib/design ui/src/screens/runs ui/src/i18n/messages/it.ts ui/src/i18n/messages/en.ts ui/src/kit/sections/app/WidthsSection.vue ui/src/assets/utilities.css
git commit -m "refactor(ui): Runs on GridTable, a run opens from its Actions too" -m "Removes lib/design/tables.ts and its three tests: they held the wide and narrow grid templates together, and no template is left — a column folds by its own declaration, pinned by gridColumn.test.ts and columns.test.ts."
```

If `check-test-count.mjs` is run before the end, it will name `tables.test.ts`; the commit body is the reason.

---

### Task 10: The wiki's table on `GridTable`

**Files:**
- Create: `ui/src/lib/wiki/wikiQueue.ts`, `ui/src/lib/wiki/wikiQueue.test.ts`
- Modify: `ui/src/lib/table/columns.ts`, `ui/src/lib/table/columns.test.ts`
- Modify: `ui/src/screens/wiki/list/WikiTable.vue`, `ui/src/screens/wiki/WikiCategoryList.vue`

**Interfaces:**
- Consumes: `factColumns(category)` and `FactColumn` (`lib/wiki/factChips`), `categoryHasId` (`lib/wiki/listFacets`), `PageProgress`.
- Produces: `pageQueueTarget(page: WikiPageRef, progress: PageProgress | null): QueueTarget | null`; `wikiColumns(facts: readonly FactColumn[], hasId: boolean): GridColumn[]`.

- [ ] **Step 1: The failing test**

```ts
// ui/src/lib/wiki/wikiQueue.test.ts
import { describe, expect, it } from 'vitest'
import type { PageProgress, Target } from '@/lib/ipc/types'
import { pageQueueTarget } from './wikiQueue'

const achievement: Target = { kind: 'achievement', id: 12 }
const item: Target = { kind: 'item', id: 3 }

describe('pageQueueTarget', () => {
  it('is an achievement page itself, done as the save says', () => {
    expect(pageQueueTarget(achievement, { kind: 'achievement', done: false })).toEqual({ achievement: 12, done: false })
    expect(pageQueueTarget(achievement, { kind: 'achievement', done: true })).toEqual({ achievement: 12, done: true })
  })

  it('is an achievement page even with no save to read', () => {
    expect(pageQueueTarget(achievement, null)).toEqual({ achievement: 12, done: false })
  })

  it('is the achievement that unlocks an item not yet unlocked', () => {
    const progress: PageProgress = { kind: 'item', collected: null, unlocked: false, unlockedBy: 40 }
    expect(pageQueueTarget(item, progress)).toEqual({ achievement: 40, done: false })
  })

  it('is nothing for an item already unlocked, or one nobody unlocks', () => {
    expect(pageQueueTarget(item, { kind: 'item', collected: null, unlocked: true, unlockedBy: 40 })).toBeNull()
    expect(pageQueueTarget(item, { kind: 'item', collected: null, unlocked: null, unlockedBy: null })).toBeNull()
  })

  it('is the achievement behind an unlockable not yet unlocked', () => {
    expect(pageQueueTarget(item, { kind: 'unlockable', unlocked: false, unlockedBy: 41 })).toEqual({ achievement: 41, done: false })
    expect(pageQueueTarget(item, { kind: 'unlockable', unlocked: true, unlockedBy: 41 })).toBeNull()
  })

  it('is nothing for a page with no achievement behind it', () => {
    expect(pageQueueTarget({ kind: 'article', title: 'Pills' }, null)).toBeNull()
    expect(pageQueueTarget(item, { kind: 'bestiary', met: 0, killed: 0, killedYou: 0 })).toBeNull()
  })
})
```

- [ ] **Step 2: Run to fail; then write**

```ts
// ui/src/lib/wiki/wikiQueue.ts
import { assertNever } from '@/lib/assertNever'
import type { PageProgress, Target } from '@/lib/ipc/types'
import type { QueueTarget } from '@/lib/plan/queueAction'

// What a wiki row's Actions button puts in the Plan's queue: the achievement the page is, or the
// one that unlocks what the page is about while it is still locked. Pages with nothing to earn
// behind them — articles, monsters, a character's marks — offer nothing.
export const pageQueueTarget = (
  target: Target,
  progress: PageProgress | null,
): QueueTarget | null => {
  if (target.kind === 'achievement')
    return {
      achievement: target.id,
      done: progress?.kind === 'achievement' && progress.done,
    }
  if (progress === null) return null
  switch (progress.kind) {
    case 'item':
      return progress.unlocked === false && progress.unlockedBy !== null
        ? { achievement: progress.unlockedBy, done: false }
        : null
    case 'unlockable':
      return progress.unlocked ? null : { achievement: progress.unlockedBy, done: false }
    case 'achievement':
    case 'character':
    case 'challenge':
    case 'bestiary':
      return null
    default:
      return assertNever(progress)
  }
}
```

Run → PASS.

- [ ] **Step 3: The columns, built from the category**

```ts
// The wiki's table: the picture, the id (where the category has one), the name, the edition,
// every fact column the category gives (`factColumns`, the same table the card grid's chips
// read), and the save's state. The first fact stays at compact, the rest fold.
export const wikiColumns = (
  facts: readonly FactColumn[],
  hasId: boolean,
): GridColumn[] => [
  { key: 'figure', header: null, width: fixed('w-figure-row'), fold: ColumnFold.Never, align: ColumnAlign.Center },
  ...(hasId
    ? [{ key: 'id', header: 'wiki.list.sort.id', width: fixed('w-wiki-table-id'), fold: ColumnFold.Compact, align: ColumnAlign.Start } as const]
    : []),
  { key: 'name', header: 'wiki.list.sort.name', width: grow(1), fold: ColumnFold.Never, align: ColumnAlign.Start },
  { key: 'edition', header: 'wiki.list.sort.edition', width: fixed('w-wiki-table-edition'), fold: ColumnFold.Compact, align: ColumnAlign.Start },
  ...facts.map((fact, at): GridColumn => ({
    key: `fact-${fact.key}`,
    header: fact.label,
    width: fixed('w-wiki-table-fact'),
    fold: at > 0 ? ColumnFold.Compact : ColumnFold.Never,
    align: ColumnAlign.Start,
  })),
  { key: 'profile', header: 'wiki.list.facet.profile', width: fixed('w-wiki-table-fact'), fold: ColumnFold.Never, align: ColumnAlign.Start },
]
```

Register `Wiki: 'wiki'` in `ListTable` with `TableColumns[ListTable.Wiki] = wikiColumns(factColumns(<a category with facts and an id>), true)` — pick the items category's constant from `@/router/routeTable` (read `WikiCategory`); the test then checks every fixed token the wiki uses. Check `FactColumn.label` is a `Message`; if it is not, adapt `header`'s type rather than casting.

Run `vitest run src/lib/table` → PASS.

- [ ] **Step 4: `WikiTable.vue`** — keep its props and emits; replace the template:

```vue
<template>
  <GridTable
    v-if="pages.length > 0"
    :columns="columns"
    :rows="pages"
    :row-key="(page) => pageKey(page.target) ?? page.title"
    :row-height="RowHeight.Wiki"
    virtual
    clickable
    :offset="offset"
    @offset-change="emit('offsetChange', $event)"
    @row-click="(page, event) => emit('open', page, event)"
  >
    <template #head-edition>
      <span class="flex items-center gap-1 truncate">
        {{ t('wiki.list.sort.edition') }}
        <component :is="sortArrow('edition')" v-if="sortArrow('edition')" class="size-3" />
      </span>
    </template>
    <template v-for="fact in facts" :key="fact.key" #[`head-fact-${fact.key}`]>
      <span class="flex items-center gap-1 truncate">
        {{ t(fact.label) }}
        <component :is="sortArrow(fact.key)" v-if="sortArrow(fact.key)" class="size-3" />
      </span>
    </template>
    <template #cell-figure="{ row }">
      <WikiFigure :target="row.target" :url="row.iconUrl" :size="FigureSize.Row" />
    </template>
    <template #cell-id="{ row }">
      <span class="truncate text-faint-foreground tabular-nums">{{ pageId(row.target) }}</span>
    </template>
    <template #cell-name="{ row }">
      <Button
        :variant="ButtonVariant.RefQuiet"
        :size="ButtonSize.Compact"
        class="truncate text-body"
        @click.stop="emit('open', row, $event)"
        >{{ row.title }}</Button
      >
    </template>
    <template #cell-edition="{ row }"><EditionBadge :dlc="row.dlc" /></template>
    <template v-for="fact in facts" :key="fact.key" #[`cell-fact-${fact.key}`]="{ row }">
      <EmptyValue v-if="cellText(row, fact.key) === null">{{ t('wiki.infobox.none') }}</EmptyValue>
      <span v-else class="truncate tabular-nums">{{ cellText(row, fact.key) }}</span>
    </template>
    <template #cell-profile="{ row }">
      <ProgressBadge :progress="wiki.progressFor(row.target)" compact />
    </template>
    <template #actions="{ row }">
      <QueueActionButton :target="pageQueueTarget(row.target, wiki.progressFor(row.target))" />
    </template>
  </GridTable>
</template>
```

Script: keep `sortArrow`, `cellText`; `facts = computed(() => factColumns(props.category))` (rename of `columns`), `columns = computed(() => wikiColumns(facts.value, categoryHasId(props.category)))`; import `GridTable`, `RowHeight`, `wikiColumns`, `QueueActionButton`, `pageQueueTarget`; drop `VirtualRows`, `rowWikiPx`. The title becomes a button so the keyboard can open a page now that the row is not one: check `ButtonVariant.RefQuiet` and `ButtonSize.Compact` look right in a 72px row; another variant that exists is fine, a new one is not. The wiki had `pages.length > 0` on its `VirtualRows` and drew its header always; keep the header for an empty list only if `WikiCategoryList.vue` shows nothing else in that case — read it and decide, and say which in the commit body.

- [ ] **Step 5: The list loads the queue** — `WikiCategoryList.vue`: `useQueueOffer()`, `queue.load()` on mount, `QueueError`, as in Task 7 Step 6.

- [ ] **Step 6: Checks and a look** — `vitest run src/lib`, `typecheck`, `scan`, `lint` → clean. `ui:dev`, wiki → Achievements in table view: an unearned achievement's button is enabled and adds; Items: a locked item queues its achievement; a click on a row opens the page; a Ctrl-click opens **one** new tab; a click on the title opens once; the sort arrows still mark the sorted column; the list keeps its scroll position when you go to a page and back.

- [ ] **Step 7: Commit**

```bash
git add ui/src/lib/wiki/wikiQueue.ts ui/src/lib/wiki/wikiQueue.test.ts ui/src/lib/table ui/src/screens/wiki/list/WikiTable.vue ui/src/screens/wiki/WikiCategoryList.vue
git commit -m "feat(ui): the wiki's table on GridTable, a page's achievement can be queued"
```

---

### Task 11: The scan rule, and the documents

**Files:**
- Modify: `ui/scripts/scan-conventions.mjs`
- Modify: `docs/frontend-conventions.md`
- Modify: `docs/superpowers/specs/2026-10-04-grid-table-design.md` only if Task 3 Step 3 or Task 10 Step 4 changed a decision

- [ ] **Step 1: The fixtures first** — in the fixtures array of `scan-conventions.mjs`, delete *a narrow template with its hidden cells is allowed* and *a narrow template with no hidden cell is half the work*, and add:

```js
  {
    name: 'a striped list in a cn() outside GridTable is caught',
    file: 'src/screens/foo/FooTable.vue',
    body: "<template>\n  <div :class=\"cn('flex', index % 2 === 1 && 'bg-row-alt')\" />\n</template>\n",
    expect: ['a striped list outside GridTable'],
  },
  {
    name: 'a striped list in a ternary is caught',
    file: 'src/screens/foo/FooTable.vue',
    body: "<template>\n  <div :class=\"[index % 2 === 1 ? 'bg-row-alt' : 'bg-transparent']\" />\n</template>\n",
    expect: ['a striped list outside GridTable'],
  },
  {
    name: 'a striped list by even: is caught',
    file: 'src/components/foo/FooRow.vue',
    body: '<template>\n  <tr class="even:bg-row-alt" />\n</template>\n',
    expect: ['a striped list outside GridTable'],
  },
  {
    name: 'GridTable may stripe its rows',
    file: 'src/components/ui/grid-table/GridTable.vue',
    body: "<template>\n  <div :class=\"cn(index % 2 === 1 && 'bg-row-alt')\" />\n</template>\n",
    expect: [],
  },
```

- [ ] **Step 2: Run the scan and see the new fixtures fail**

Run: `pnpm scan`
Expected: the three "is caught" fixtures fail — their rule does not exist yet. That is the check being seen to fail before it is trusted.

- [ ] **Step 3: Replace the rule** — delete *narrow grid template with no column hidden* (its comment cites spec 3.13a §7, which this replaces), and add in its place:

```js
  {
    // A list drawn as a table is a GridTable: its cells keep to their columns and it ends with
    // Actions. Every hand-written list stripes its rows, so the stripe is the form this reads —
    // not a list of the tables there were. What it cannot see: a list that does not stripe.
    name: 'a striped list outside GridTable',
    test: (file, body) =>
      !GRID_TABLE_DIRS.some((dir) => isUnder(file, dir)) &&
      /\bbg-row-alt\b/.test(body),
  },
```

with, near `UI_DIR`:

```js
// The two places a row may be striped: the list table, and shadcn's `<table>` primitive.
const GRID_TABLE_DIRS = [join(UI_DIR, 'grid-table'), join(UI_DIR, 'table')]
```

and in `EXEMPTIONS`:

```js
  {
    file: 'src/components/marks/MarksGrid.vue',
    check: 'a striped list outside GridTable',
    reason:
      'the completion matrix is a grid of mark cells, one column per boss bound from data — a list table has rows of named columns, and its cells would be clipped',
  },
  {
    file: 'src/screens/search/SearchResults.vue',
    check: 'a striped list outside GridTable',
    reason:
      'search results are a list with no columns: a two-line result per row, nothing to align under a header',
  },
```

Add a line to the script's header comment where the other rules' blind spots are listed: the striped-list rule cannot see a list that does not stripe.

- [ ] **Step 4: Run the scan**

Run: `pnpm scan`
Expected: `0 violations`, every fixture passes, the declared exemptions now two more. If any other file trips the rule, it is a seventh table: stop and report.

- [ ] **Step 5: The conventions** — in `docs/frontend-conventions.md`:
  - replace §"A table drops columns by priority" with:

```markdown
### A list is a `GridTable`

A list drawn as a table — a header of named columns over a row per item — is a `GridTable`
(`components/ui/grid-table/`). Its columns are declared once, in `lib/table/columns.ts`: a key, a
header, a width (a token's class, or a grow weight), whether it folds at compact, its alignment.
The header and every row are drawn from that declaration, so a column that folds hides its header
and its cells together, and there is no narrow template to keep in step. Every cell keeps to its
track whatever it holds (`grid-cell` in `utilities.css`: clipped, its children allowed to shrink,
a badge allowed to wrap), and every table ends with **Actions**, which no screen declares.

**What survives a fold**: who the row is, how it is doing, and the button that acts on it. What
falls is what *explains* the row and what is *derived* from it. A table that folds nothing says
why in `columns.test.ts`.
```

  - in §"What checks it", replace "a narrow grid template with no hidden cell" with "a striped list outside `GridTable`", and keep the count of rules right;
  - in the checks table, replace the row *A narrow grid template with no column hidden* with `| **A striped list outside \`GridTable\`** — since 2026-10-04, card #100 | scan: \`a striped list outside GridTable\` |`;
  - grep the document for `grid-cols-`, `tables.ts`, `-narrow` and fix what is left.

- [ ] **Step 6: Commit**

```bash
git add ui/scripts/scan-conventions.mjs docs/frontend-conventions.md
git commit -m "chore(ui): a striped list outside GridTable is caught by the scan"
```

(Add the spec to the same commit only if it changed, and say what in the body.)

---

### Task 12: The full check, the measurement, the card

- [ ] **Step 1: `pnpm check`** from the worktree root. Expected: green. `check-test-count.mjs` names `tables.test.ts` (3 fewer) and the total is higher than at the merge base; if the total is lower, find which file lost tests before anything else.

- [ ] **Step 2: Measure in the browser** — `pnpm ui:dev -- --port 1430`, each of the six screens at 640 × 480 and at 1324 wide, with the console snippet from `docs/frontend-conventions.md`: it prints nothing. Then stop the server by its PID.

- [ ] **Step 3: Merge** — from the main worktree `C:\Projects\isaac-dome`, on `develop`: `git merge --no-ff feature/grid-table -m "merge: #100, the lists' table, into develop"`, `git push origin develop`; delete the branch locally and on the remote after checking `git rev-list --count develop..feature/grid-table` is 0; remove the worktree (it has no junctions: it was made without `samples/`, but list reparse points first as CLAUDE.md says).

- [ ] **Step 4: The board** — card #100: a comment with what was done, what changed from the spec (Task 3 Step 3's finding, Task 10's empty-list decision), and what is unverified — the real window at 100% and 150%, the queue buttons against a real save. Move it to `UAT` with `NEEDS WINDOW`. Card #53: a comment that the pill is fixed and its UAT can resume.
