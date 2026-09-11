import { countBy } from 'lodash-es'
import { assertNever } from '@/lib/assertNever'
import type { CollectionItem } from '@/lib/ipc/types'

// What a collectible is for this save, one answer for every place that draws it. Unread is its
// own state: a missing section never reads as "not in the collection".
export const ItemState = {
  InCollection: 'inCollection',
  Available: 'available',
  Locked: 'locked',
  Unknown: 'unknown',
} as const
export type ItemState = (typeof ItemState)[keyof typeof ItemState]

// The order the states are shown in: what's found, what can be found tonight, what can't be yet,
// what the save doesn't let us read.
export const itemStateOrder: ItemState[] = [
  ItemState.InCollection,
  ItemState.Available,
  ItemState.Locked,
  ItemState.Unknown,
]

export const itemState = (item: CollectionItem): ItemState => {
  if (item.inCollection === null) return ItemState.Unknown
  if (item.inCollection) return ItemState.InCollection
  const { lock } = item
  switch (lock.kind) {
    case 'free':
    case 'unlocked':
      return ItemState.Available
    case 'locked':
      return ItemState.Locked
    case 'unknown':
      return ItemState.Unknown
    default:
      return assertNever(lock)
  }
}

export const itemStateCounts = (
  items: CollectionItem[],
): Record<ItemState, number> => {
  const counted = countBy(items, itemState)
  const count = (state: ItemState): number => counted[state] ?? 0
  return {
    [ItemState.InCollection]: count(ItemState.InCollection),
    [ItemState.Available]: count(ItemState.Available),
    [ItemState.Locked]: count(ItemState.Locked),
    [ItemState.Unknown]: count(ItemState.Unknown),
  }
}
