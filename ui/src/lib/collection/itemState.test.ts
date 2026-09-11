import { describe, expect, it } from 'vitest'
import type { CollectionItem, LockView } from '@/lib/ipc/types'
import { ItemState, itemState, itemStateCounts } from './itemState'

const item = (over: Partial<CollectionItem> = {}): CollectionItem => ({
  id: 1,
  kind: 'passive',
  name: 'The Sad Onion',
  iconUrl: null,
  quality: 3,
  pools: ['treasure'],
  origin: 'rebirth',
  inCollection: false,
  lock: { kind: 'free' },
  ...over,
})
const locked: LockView = {
  kind: 'locked',
  achievement: 62,
  text: '"Epic Fetus" has appeared in the basement',
}

describe('itemState', () => {
  it('is in the collection whatever the lock says', () => {
    expect(itemState(item({ inCollection: true, lock: locked }))).toBe(
      ItemState.InCollection,
    )
  })

  it('is to find when nothing locks it, or its achievement is done', () => {
    expect(itemState(item())).toBe(ItemState.Available)
    expect(
      itemState(
        item({ lock: { kind: 'unlocked', achievement: 1, text: null } }),
      ),
    ).toBe(ItemState.Available)
  })

  it('is locked when its achievement is not done', () => {
    expect(itemState(item({ lock: locked }))).toBe(ItemState.Locked)
  })

  it('is unknown when the collection or the lock is unread, never "not found"', () => {
    expect(itemState(item({ inCollection: null }))).toBe(ItemState.Unknown)
    expect(
      itemState(
        item({ lock: { kind: 'unknown', achievement: 62, text: null } }),
      ),
    ).toBe(ItemState.Unknown)
  })

  it('counts every state', () => {
    expect(
      itemStateCounts([
        item({ inCollection: true }),
        item(),
        item({ lock: locked }),
        item({ lock: locked }),
      ]),
    ).toEqual({ inCollection: 1, available: 1, locked: 2, unknown: 0 })
  })
})
