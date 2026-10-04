import { ColumnAlign, ColumnFold, fixed, grow } from './gridColumn'
import type { GridColumn } from './gridColumn'

// Each list table's columns, in the order they are drawn. What folds at compact is 3.13b's rule:
// who the row is and how it is doing stay; what explains it, and what is derived from it, go.
// The weights are the `fr` of the grid templates these replaced.

export const ListTable = {
  Challenges: 'challenges',
  Unlock: 'unlock',
  Collection: 'collection',
  Live: 'live',
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

// The drawing, the achievement, what it unlocks, its condition, the state, the fan-out.
export const unlockColumns: readonly GridColumn[] = [
  {
    key: 'art',
    header: null,
    width: fixed('w-unlock-art'),
    fold: ColumnFold.Never,
    align: ColumnAlign.Center,
  },
  {
    key: 'achievement',
    header: 'unlock.columns.achievement',
    width: grow(1.5),
    fold: ColumnFold.Never,
    align: ColumnAlign.Start,
  },
  {
    key: 'unlocks',
    header: 'unlock.columns.unlocks',
    width: grow(1.1),
    fold: ColumnFold.Compact,
    align: ColumnAlign.Start,
  },
  {
    key: 'condition',
    header: 'unlock.columns.condition',
    width: grow(1),
    fold: ColumnFold.Compact,
    align: ColumnAlign.Start,
  },
  {
    key: 'state',
    header: 'unlock.columns.state',
    width: fixed('w-unlock-state'),
    fold: ColumnFold.Never,
    align: ColumnAlign.Start,
  },
  {
    key: 'fanOut',
    header: 'unlock.columns.fanOut',
    width: fixed('w-unlock-fan'),
    fold: ColumnFold.Compact,
    align: ColumnAlign.End,
  },
]

// The sprite, the name, the quality, the pools, the origin, the state.
export const collectionColumns: readonly GridColumn[] = [
  {
    key: 'sprite',
    header: null,
    width: fixed('w-collection-sprite'),
    fold: ColumnFold.Never,
    align: ColumnAlign.Center,
  },
  {
    key: 'item',
    header: 'collection.columns.item',
    width: grow(1.4),
    fold: ColumnFold.Never,
    align: ColumnAlign.Start,
  },
  {
    key: 'quality',
    header: 'collection.columns.quality',
    width: fixed('w-collection-quality'),
    fold: ColumnFold.Compact,
    align: ColumnAlign.Start,
  },
  {
    key: 'pools',
    header: 'collection.columns.pools',
    width: grow(1),
    fold: ColumnFold.Compact,
    align: ColumnAlign.Start,
  },
  {
    key: 'origin',
    header: 'collection.columns.origin',
    width: fixed('w-collection-origin'),
    fold: ColumnFold.Compact,
    align: ColumnAlign.Start,
  },
  {
    key: 'state',
    header: 'collection.columns.state',
    width: fixed('w-collection-state'),
    fold: ColumnFold.Never,
    align: ColumnAlign.Start,
  },
]

// The achievement, the cell it needs, how the game words it, and how much it opens in turn.
export const liveColumns: readonly GridColumn[] = [
  {
    key: 'achievement',
    header: 'live.column.achievement',
    width: grow(1.3),
    fold: ColumnFold.Never,
    align: ColumnAlign.Start,
  },
  {
    key: 'cell',
    header: 'live.column.cell',
    width: grow(0.8),
    fold: ColumnFold.Never,
    align: ColumnAlign.Start,
  },
  {
    key: 'condition',
    header: 'live.column.condition',
    width: grow(1.4),
    fold: ColumnFold.Never,
    align: ColumnAlign.Start,
  },
  {
    key: 'opens',
    header: 'live.column.opens',
    width: fixed('w-live-opens'),
    fold: ColumnFold.Never,
    align: ColumnAlign.End,
  },
]

export const TableColumns: Record<ListTable, readonly GridColumn[]> = {
  [ListTable.Challenges]: challengeColumns,
  [ListTable.Unlock]: unlockColumns,
  [ListTable.Collection]: collectionColumns,
  [ListTable.Live]: liveColumns,
}
