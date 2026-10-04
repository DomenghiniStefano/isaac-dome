import { factColumns } from '@/lib/wiki/factChips'
import type { FactColumn } from '@/lib/wiki/factChips'
import { WikiCategory } from '@/router/routeTable'
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
  Runs: 'runs',
  Wiki: 'wiki',
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

// The day, the character, how it ended, the floors, the seed, where it came from. The day stays
// at compact because when you played is what a diary is read by. The floors fold: with Actions
// beside them, keeping them left the outcome 69px at 428 — "abbandonata" is one word and needs
// about 107 — and the run's own page still says how far you got.
export const runsColumns: readonly GridColumn[] = [
  {
    key: 'date',
    header: 'runs.column.date',
    width: fixed('w-runs-date'),
    fold: ColumnFold.Never,
    align: ColumnAlign.Start,
  },
  {
    key: 'character',
    header: 'runs.column.character',
    width: grow(1),
    fold: ColumnFold.Never,
    align: ColumnAlign.Start,
  },
  {
    key: 'outcome',
    header: 'runs.column.outcome',
    width: grow(2),
    fold: ColumnFold.Never,
    align: ColumnAlign.Start,
  },
  {
    key: 'floors',
    header: 'runs.column.floors',
    width: fixed('w-runs-floors'),
    fold: ColumnFold.Compact,
    align: ColumnAlign.End,
  },
  {
    key: 'seed',
    header: 'runs.column.seed',
    width: grow(1),
    fold: ColumnFold.Compact,
    align: ColumnAlign.Start,
  },
  {
    key: 'source',
    header: 'runs.column.source',
    width: grow(1),
    fold: ColumnFold.Compact,
    align: ColumnAlign.Start,
  },
]

// The wiki's table: the picture, the id (where the category has one), the name, the edition,
// every fact column the category gives (`factColumns`, the same table the card grid's chips
// read), and the save's state. Every fact folds at compact: they explain a page, and at 428px
// with Actions beside the state, keeping even one left the name 16px — its padding. Above it the
// facts share what is left with the name, which weighs two: a category can have eight, and at a
// fixed width each the row would be wider than the window.
export const wikiColumns = (
  facts: readonly FactColumn[],
  hasId: boolean,
): GridColumn[] => [
  {
    key: 'figure',
    header: null,
    width: fixed('w-figure-row'),
    fold: ColumnFold.Never,
    align: ColumnAlign.Center,
  },
  ...(hasId ? [idColumn] : []),
  {
    key: 'name',
    header: 'wiki.list.sort.name',
    width: grow(2),
    fold: ColumnFold.Never,
    align: ColumnAlign.Start,
  },
  {
    key: 'edition',
    header: 'wiki.list.sort.edition',
    width: fixed('w-wiki-table-edition'),
    fold: ColumnFold.Compact,
    align: ColumnAlign.Start,
  },
  ...facts.map((fact): GridColumn => ({
    key: `fact-${fact.key}`,
    header: fact.label,
    width: grow(1),
    fold: ColumnFold.Compact,
    align: ColumnAlign.Start,
  })),
  {
    key: 'profile',
    header: 'wiki.list.facet.profile',
    width: fixed('w-wiki-table-fact'),
    fold: ColumnFold.Never,
    align: ColumnAlign.Start,
  },
]

const idColumn: GridColumn = {
  key: 'id',
  header: 'wiki.list.sort.id',
  width: fixed('w-wiki-table-id'),
  fold: ColumnFold.Compact,
  align: ColumnAlign.Start,
}

export const TableColumns: Record<ListTable, readonly GridColumn[]> = {
  [ListTable.Challenges]: challengeColumns,
  [ListTable.Unlock]: unlockColumns,
  [ListTable.Collection]: collectionColumns,
  [ListTable.Live]: liveColumns,
  [ListTable.Runs]: runsColumns,
  // The items category: it has an id and fact columns, so every width token the wiki's table
  // uses is read here.
  [ListTable.Wiki]: wikiColumns(factColumns(WikiCategory.Items), true),
}
