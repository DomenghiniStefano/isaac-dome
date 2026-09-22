import { describe, expect, it } from 'vitest'
import type { QueueRow, UnlockNode } from '@/lib/ipc/types'
import { queueExtras } from './queueExtras'

const node = (id: number, text: string): UnlockNode =>
  ({
    achievement: { kind: 'known', id, text, condition: null, iconUrl: null },
    done: false,
    unlocks: [],
    origin: null,
    missing: [],
    graph: {
      kind: 'computed',
      availableNow: true,
      blockedBy: 0,
      fanOut: 0,
      stepsMissing: 0,
    },
  }) as UnlockNode

const row = (
  id: number,
  text: string,
  over: Partial<QueueRow> = {},
): QueueRow =>
  ({
    node: node(id, text),
    wanted: false,
    origins: [],
    stepsNotQueued: 0,
    ...over,
  }) as QueueRow

describe('why a row is in the queue', () => {
  it('names the wish it serves, by that wish’s own text', () => {
    const polaroid = row(1, 'Polaroid', { wanted: true })
    const heart = row(2, 'Cuore di Isaac', { origins: [1] })
    expect(queueExtras(heart, [polaroid, heart]).serves).toEqual([
      { id: 1, text: 'Polaroid' },
    ])
  })

  // A wish can have left the queue between the write and the read: the row still says it
  // serves something, and says it by id rather than by nothing.
  it('falls back to the id when the wish is not in the queue', () => {
    const orphan = row(2, 'Cuore di Isaac', { origins: [99] })
    expect(queueExtras(orphan, [orphan]).serves).toEqual([
      { id: 99, text: null },
    ])
  })

  it('serves nothing when you asked for it yourself', () => {
    const own = row(1, 'Polaroid', { wanted: true })
    const extras = queueExtras(own, [own])
    expect(extras.wanted).toBe(true)
    expect(extras.serves).toEqual([])
  })

  it('carries how many of its steps are still outside the queue', () => {
    const r = row(3, 'Negativo', { stepsNotQueued: 2 })
    expect(queueExtras(r, [r]).stepsNotQueued).toBe(2)
  })
})
