import { describe, expect, it } from 'vitest'
import type { QueueRow, QueueView } from '@/lib/ipc/types'
import { planNow } from './planNow'

const row = (id: number, availableNow: boolean, done: boolean): QueueRow => ({
  node: {
    achievement: {
      kind: 'known',
      id,
      text: `t${id}`,
      condition: null,
      iconUrl: null,
    },
    done,
    unlocks: [],
    origin: null,
    missing: [],
    graph: {
      kind: 'computed',
      availableNow,
      blockedBy: availableNow ? 0 : 1,
      fanOut: 0,
      stepsMissing: availableNow ? 0 : 1,
    },
  },
  wanted: true,
  origins: [],
  stepsNotQueued: 0,
})

const view = (rows: QueueRow[]): QueueView => ({
  rows,
  diagnostics: [],
  storeAvailable: true,
})

const ids = (rows: QueueRow[]): (number | false)[] =>
  rows.map((r) => r.node.achievement.kind === 'known' && r.node.achievement.id)

describe('planNow', () => {
  // The order is the queue's, and the queue's order is the user's. A landing page that
  // re-sorted it would be contradicting the screen that owns it.
  it('keeps the queue order and only what can be played now', () => {
    const got = planNow(
      view([row(1, false, false), row(2, true, false), row(3, true, false)]),
      5,
    )
    expect(ids(got)).toEqual([2, 3])
  })

  it('leaves out what is already done', () => {
    expect(planNow(view([row(4, true, true)]), 5)).toEqual([])
  })

  it('answers nothing without a queue', () => {
    expect(planNow(null, 5)).toEqual([])
  })

  it('answers nothing when the store could not be opened', () => {
    const unavailable: QueueView = {
      rows: [row(1, true, false)],
      diagnostics: [],
      storeAvailable: false,
    }
    expect(planNow(unavailable, 5)).toEqual([])
  })

  it('never shows more than the limit', () => {
    expect(
      planNow(view([row(1, true, false), row(2, true, false)]), 1),
    ).toHaveLength(1)
  })
})
