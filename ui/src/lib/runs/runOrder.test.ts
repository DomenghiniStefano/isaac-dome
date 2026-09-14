import { describe, expect, it } from 'vitest'
import type { RunView } from '@/lib/ipc/types'
import { orderRuns, sessionTime } from './runOrder'

const run = (
  source: RunView['source'],
  ordinal: number,
  seedWords = 'AAAA BBBB',
): RunView => ({
  source,
  ordinal,
  character: 'Cain',
  seedWords,
  online: false,
  outcome: { kind: 'abandoned' },
  floors: 1,
  startingItems: [],
  collected: [],
  heldActive: null,
  achievements: [],
})

const session = (name: string) => ({ kind: 'session', name }) as const
const live = { kind: 'live' } as const

describe('sessionTime', () => {
  // The folder's name is the only clock this archive has: the log itself carries none.
  it('reads the wall clock a session folder is named with', () => {
    expect(sessionTime('09_12_2026__13_34_26')).toBe(
      Date.UTC(2026, 8, 12, 13, 34, 26),
    )
  })

  it('answers nothing for a name that is not one', () => {
    expect(sessionTime('desyncs')).toBeNull()
    expect(sessionTime('09_12_2026')).toBeNull()
    expect(sessionTime('13_45_2026__13_34_26')).toBeNull()
  })
})

describe('orderRuns', () => {
  // `Live` has no folder and therefore no clock: it is first by decision, because it is the
  // launch the app is watching, not because it compares greater than anything.
  it('puts the launch being watched first', () => {
    const rows = [run(session('09_12_2026__13_34_26'), 1), run(live, 1)]
    expect(orderRuns(rows).map((r) => r.source.kind)).toEqual([
      'live',
      'session',
    ])
  })

  it('puts the newer session first', () => {
    const older = run(session('09_10_2026__08_00_00'), 1)
    const newer = run(session('09_12_2026__13_34_26'), 1)
    expect(orderRuns([older, newer])[0]).toBe(newer)
  })

  it('counts down the ordinals inside one source', () => {
    const s = session('09_12_2026__13_34_26')
    const rows = [run(s, 1, 'first'), run(s, 3, 'third'), run(s, 2, 'second')]
    expect(orderRuns(rows).map((r) => r.ordinal)).toEqual([3, 2, 1])
  })

  // A name we cannot read is not a date we can compare, and sorting it to either end would
  // say something about when it happened. It keeps the place the archive gave it.
  it('leaves a session whose name is not a clock where it was', () => {
    const rows = [
      run(session('09_12_2026__13_34_26'), 1, 'newest'),
      run(session('not a date'), 1, 'unreadable'),
      run(session('09_10_2026__08_00_00'), 1, 'older'),
    ]
    expect(orderRuns(rows).map((r) => r.seedWords)).toEqual([
      'newest',
      'unreadable',
      'older',
    ])
  })

  it('does not touch the array it was given', () => {
    const rows = [run(session('09_10_2026__08_00_00'), 1), run(live, 1)]
    const before = [...rows]
    orderRuns(rows)
    expect(rows).toEqual(before)
  })

  // The case that made the first attempt at this function wrong: with an unreadable name
  // *between* two readable ones, "newer first" and "keeps its place" cannot both be obeyed
  // by one comparison — A before B by clock, B before C by position, C before A by position
  // is a cycle, and `Array.sort` given a contradictory comparator answers something
  // arbitrary. Sorting only the readable names, into the slots they already hold, is a
  // total order.
  it('orders the readable sessions around an unreadable one, whatever the archive order', () => {
    const rows = [
      run(session('09_10_2026__08_00_00'), 1, 'older'),
      run(session('not a date'), 1, 'unreadable'),
      run(session('09_12_2026__13_34_26'), 1, 'newest'),
    ]
    expect(orderRuns(rows).map((r) => r.seedWords)).toEqual([
      'newest',
      'unreadable',
      'older',
    ])
  })
})
