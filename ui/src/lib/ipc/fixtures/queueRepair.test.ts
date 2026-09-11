import { describe, expect, it } from 'vitest'
import type { Requires } from './queueRepair'
import { moveAfter, moveRow } from './queueRepair'

// The same readable cases as crates/plan/tests/order.rs: this port exists only for the
// development server, and it has to repair the way the app does.
const pairs =
  (...p: [number, number][]): Requires =>
  (a, b) =>
    p.some(([x, y]) => x === a && y === b)

describe('moveRow', () => {
  it('lands where dropped with no dependencies', () => {
    expect(moveRow([1, 2, 3, 4], 4, 1, pairs())).toEqual([1, 4, 2, 3])
  })

  it('drags what needs the row', () => {
    expect(moveRow([1, 2, 3], 1, 2, pairs([3, 1]))).toEqual([2, 1, 3])
  })

  it('stops under prerequisites, which never move', () => {
    expect(moveRow([1, 2, 3, 4], 4, 0, pairs([4, 1], [4, 2]))).toEqual([
      1, 2, 4, 3,
    ])
  })

  it('means the same place when dependents sit before the index', () => {
    expect(moveRow([1, 2, 3, 4], 1, 2, pairs([2, 1]))).toEqual([3, 1, 2, 4])
  })

  it('leaves a queue without the row as it is', () => {
    expect(moveRow([1, 2], 99, 0, pairs())).toEqual([1, 2])
  })
})

describe('moveAfter', () => {
  it('lands right below the row named, or at the top', () => {
    expect(moveAfter([1, 2, 3, 4], 1, 3, pairs())).toEqual([2, 3, 1, 4])
    expect(moveAfter([2, 3, 1, 4], 4, 2, pairs())).toEqual([2, 4, 3, 1])
    expect(moveAfter([1, 2, 3], 3, null, pairs())).toEqual([3, 1, 2])
  })

  it('goes as low as it can under one of its own dependents', () => {
    expect(moveAfter([1, 3, 2, 4], 1, 2, pairs([2, 1]))).toEqual([3, 1, 2, 4])
  })

  it('stops right below a prerequisite it rises past', () => {
    expect(moveAfter([1, 2, 4, 3], 3, 1, pairs([3, 2]))).toEqual([1, 2, 3, 4])
  })

  it('changes nothing for a stale anchor or the row itself', () => {
    expect(moveAfter([1, 2, 3], 1, 99, pairs())).toEqual([1, 2, 3])
    expect(moveAfter([1, 2, 3], 2, 2, pairs())).toEqual([1, 2, 3])
    expect(moveAfter([1, 2, 3], 99, null, pairs())).toEqual([1, 2, 3])
  })
})
