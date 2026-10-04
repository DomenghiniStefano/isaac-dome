import type { Message } from '@/i18n/message'
import { rowWidePx, rowWikiPx } from '@/lib/scale/rows'

// A list table's columns, declared once: the header and every row are drawn from the same
// declaration, so a column that folds at compact folds in both, and a cell's width is the
// same in every row. Rows are flex, not grid: a grid template would have to be assembled from
// token names in TypeScript, which `pnpm scan` refuses, while a flex cell carries its own width.

export const ColumnFold = { Never: 'never', Compact: 'compact' } as const
export type ColumnFold = (typeof ColumnFold)[keyof typeof ColumnFold]

export const ColumnAlign = {
  Start: 'start',
  Center: 'center',
  End: 'end',
} as const
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
    ...alignClass[column.align].split(' '),
    foldClass[column.fold],
  ].filter((c): c is string => c !== null)

export const cellStyle = (
  column: GridColumn,
): Record<string, string> | undefined =>
  column.width.kind === ColumnWidthKind.Grow
    ? { '--cell-grow': String(column.width.weight) }
    : undefined
