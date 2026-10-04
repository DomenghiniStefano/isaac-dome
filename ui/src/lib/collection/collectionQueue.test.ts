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
      itemQueueTarget(
        item({ kind: 'locked', achievement: 5, text: null, page: null }),
      ),
    ).toEqual({ achievement: 5, done: false })
  })

  // Unread is never "earned": the button stays offered when the save could not say.
  it('is the achievement behind an item whose lock the save could not read', () => {
    expect(
      itemQueueTarget(
        item({ kind: 'unknown', achievement: 6, text: null, page: null }),
      ),
    ).toEqual({ achievement: 6, done: false })
  })

  it('is nothing for an item already unlocked', () => {
    expect(
      itemQueueTarget(
        item({ kind: 'unlocked', achievement: 5, text: null, page: null }),
      ),
    ).toBeNull()
  })

  it('is nothing for an item nobody has to unlock', () => {
    expect(itemQueueTarget(item({ kind: 'free' }))).toBeNull()
  })
})
