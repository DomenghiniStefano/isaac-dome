import { ColumnAlign, ColumnFold, fixed, grow } from './gridColumn'
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
