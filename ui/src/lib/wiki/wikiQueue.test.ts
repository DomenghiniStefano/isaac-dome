import { describe, expect, it } from 'vitest'
import type { PageProgress, Target } from '@/lib/ipc/types'
import { pageQueueTarget } from './wikiQueue'

const achievement: Target = { kind: 'achievement', id: 12 }
const item: Target = { kind: 'item', id: 3 }

describe('pageQueueTarget', () => {
  it('is an achievement page itself, done as the save says', () => {
    expect(
      pageQueueTarget(achievement, { kind: 'achievement', done: false }),
    ).toEqual({ achievement: 12, done: false })
    expect(
      pageQueueTarget(achievement, { kind: 'achievement', done: true }),
    ).toEqual({ achievement: 12, done: true })
  })

  it('is an achievement page even with no save to read', () => {
    expect(pageQueueTarget(achievement, null)).toEqual({
      achievement: 12,
      done: false,
    })
  })

  it('is the achievement that unlocks an item not yet unlocked', () => {
    const progress: PageProgress = {
      kind: 'item',
      collected: null,
      unlocked: false,
      unlockedBy: 40,
    }
    expect(pageQueueTarget(item, progress)).toEqual({
      achievement: 40,
      done: false,
    })
  })

  it('is nothing for an item already unlocked, or one nobody unlocks', () => {
    expect(
      pageQueueTarget(item, {
        kind: 'item',
        collected: null,
        unlocked: true,
        unlockedBy: 40,
      }),
    ).toBeNull()
    expect(
      pageQueueTarget(item, {
        kind: 'item',
        collected: null,
        unlocked: null,
        unlockedBy: null,
      }),
    ).toBeNull()
  })

  it('is the achievement behind an unlockable not yet unlocked', () => {
    expect(
      pageQueueTarget(item, {
        kind: 'unlockable',
        unlocked: false,
        unlockedBy: 41,
      }),
    ).toEqual({ achievement: 41, done: false })
    expect(
      pageQueueTarget(item, {
        kind: 'unlockable',
        unlocked: true,
        unlockedBy: 41,
      }),
    ).toBeNull()
  })

  it('is nothing for a page with no achievement behind it', () => {
    expect(
      pageQueueTarget({ kind: 'article', title: 'Pills' }, null),
    ).toBeNull()
    expect(
      pageQueueTarget(item, {
        kind: 'bestiary',
        met: 0,
        killed: 0,
        killedYou: 0,
      }),
    ).toBeNull()
  })
})
