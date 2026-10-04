import { describe, expect, it } from 'vitest'
import type { QueueRow, QueueView } from '@/lib/ipc/types'
import { QueueAction, membershipOf, queueAction } from './queueAction'

const row = (id: number, wanted: boolean): QueueRow => ({
  node: {
    achievement: {
      kind: 'known',
      id,
      text: 'x',
      condition: null,
      iconUrl: null,
    },
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
  },
  wanted,
  origins: wanted ? [] : [1],
  stepsNotQueued: 0,
})

const view = (rows: QueueRow[]): QueueView => ({
  rows,
  diagnostics: [],
  storeAvailable: true,
})

const membership = membershipOf(view([row(1, true), row(2, false)]))

describe('membershipOf', () => {
  it('splits what you asked for from what was dragged in', () => {
    expect([...membership.wanted]).toEqual([1])
    expect([...membership.steps]).toEqual([2])
  })

  it('reads nothing from a queue that is not there', () => {
    const none = membershipOf(null)
    expect(none.wanted.size).toBe(0)
    expect(none.steps.size).toBe(0)
  })
})

describe('queueAction', () => {
  it('offers to add what is not queued', () => {
    expect(queueAction({ achievement: 3, done: false }, membership, true)).toBe(
      QueueAction.Add,
    )
  })

  it('offers to remove what you asked for', () => {
    expect(queueAction({ achievement: 1, done: false }, membership, true)).toBe(
      QueueAction.Remove,
    )
  })

  it('offers nothing for a row dragged in as a prerequisite', () => {
    expect(queueAction({ achievement: 2, done: false }, membership, true)).toBe(
      QueueAction.Unavailable,
    )
  })

  it('offers nothing for an achievement already earned', () => {
    expect(queueAction({ achievement: 3, done: true }, membership, true)).toBe(
      QueueAction.Unavailable,
    )
  })

  it('offers nothing for a row with no achievement', () => {
    expect(queueAction(null, membership, true)).toBe(QueueAction.Unavailable)
  })

  it('offers nothing when the queue cannot be written, whatever the row', () => {
    for (const target of [
      { achievement: 1, done: false },
      { achievement: 3, done: false },
    ])
      expect(queueAction(target, membership, false)).toBe(
        QueueAction.Unavailable,
      )
  })
})
