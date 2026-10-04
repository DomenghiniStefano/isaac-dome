import { uniq } from 'lodash-es'
import type { Message } from '@/i18n/message'
import { assertNever } from '@/lib/assertNever'
import type { RunFloorView, RunItemRef, RunView } from '@/lib/ipc/types'

// The ways a run's items can be read. As logged is today's reading and the default; the other
// three cut the same items by what they are, where they came from, and where they were taken.
export const RunItemView = {
  AsLogged: 'asLogged',
  ByType: 'byType',
  ByOrigin: 'byOrigin',
  ByFloor: 'byFloor',
} as const
export type RunItemView = (typeof RunItemView)[keyof typeof RunItemView]

export const runItemViews: RunItemView[] = Object.values(RunItemView)

export const GroupTitleKind = {
  Message: 'message',
  Pool: 'pool',
  Floor: 'floor',
} as const

/** What a group is called: a message, a pool the page words or shows as written, a floor. */
export type GroupTitle =
  | { kind: typeof GroupTitleKind.Message; message: Message }
  | { kind: typeof GroupTitleKind.Pool; pool: string }
  | { kind: typeof GroupTitleKind.Floor; floor: RunFloorView | null }

export interface ItemGroup {
  title: GroupTitle
  items: RunItemRef[]
}

const titled = (message: Message, items: RunItemRef[]): ItemGroup => ({
  title: { kind: GroupTitleKind.Message, message },
  items,
})

const held = (run: RunView): RunItemRef[] =>
  run.heldActive === null ? [] : [run.heldActive]

const asLogged = (run: RunView): ItemGroup[] => [
  titled('runs.startingItems', run.startingItems),
  titled(
    'runs.collected',
    run.collected.map((p) => p.item),
  ),
  ...(run.heldActive === null ? [] : [titled('runs.heldActive', held(run))]),
]

// The owner's choice, 2026-10-04: in every view but the logged one, the starting items are a group
// of their own at the top, and are not repeated in another. The fold files a starting passive as a
// passive and a starting active as the one held, so by type those are taken out where they repeat.
const starting = (run: RunView): ItemGroup =>
  titled('runs.startingItems', run.startingItems)

const notStarting = (run: RunView, items: RunItemRef[]): RunItemRef[] => {
  const started = new Set(run.startingItems.map((i) => i.id))
  return items.filter((i) => !started.has(i.id))
}

const byType = (run: RunView): ItemGroup[] => [
  starting(run),
  titled('runs.passives', notStarting(run, run.passives)),
  titled('runs.familiars', notStarting(run, run.familiars)),
  titled('runs.heldActive', notStarting(run, held(run))),
]

const byOrigin = (run: RunView): ItemGroup[] => [
  starting(run),
  ...uniq(run.collected.map((p) => p.pool)).map((pool) => ({
    title: { kind: GroupTitleKind.Pool, pool },
    items: run.collected.filter((p) => p.pool === pool).map((p) => p.item),
  })),
]

// The floors in the order they were entered, the items with no floor before them.
const byFloor = (run: RunView): ItemGroup[] => [
  starting(run),
  ...[null, ...run.floorDetails.map((_, at) => at)].map((at) => ({
    title: {
      kind: GroupTitleKind.Floor,
      floor: at === null ? null : (run.floorDetails[at] ?? null),
    },
    items: run.collected.filter((p) => p.floor === at).map((p) => p.item),
  })),
]

/**
 * A run's items, cut one way. As logged keeps an empty group — the page says "none" there, as it
 * always has; the other readings leave out a group with nothing in it.
 */
export const groupItems = (run: RunView, view: RunItemView): ItemGroup[] => {
  switch (view) {
    case RunItemView.AsLogged:
      return asLogged(run)
    case RunItemView.ByType:
      return byType(run).filter((g) => g.items.length > 0)
    case RunItemView.ByOrigin:
      return byOrigin(run).filter((g) => g.items.length > 0)
    case RunItemView.ByFloor:
      return byFloor(run).filter((g) => g.items.length > 0)
    default:
      return assertNever(view)
  }
}
