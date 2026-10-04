import { describe, expect, it } from 'vitest'
import type { RunView } from '@/lib/ipc/types'
import { orderRuns, runTime, sessionTime } from './runOrder'

const run = (
  source: RunView['source'],
  ordinal: number,
  seedWords = 'AAAA BBBB',
): RunView => ({
  source,
  ordinal,
  character: 'Cain',
  characterId: 2,
  characterHeadUrl: null,
  seedWords,
  online: false,
  outcome: { kind: 'abandoned' },
  floors: 1,
  floorDetails: [],
  startingItems: [],
  collected: [],
  passives: [],
  familiars: [],
  heldActive: null,
  achievements: [],
})

const session = (name: string) => ({ kind: 'session', name }) as const
const live = { kind: 'live', id: 9, writtenUnix: null } as const
const launch = (id: number, writtenUnix: number | null) =>
  ({ kind: 'launch', id, writtenUnix }) as const

describe('sessionTime', () => {
  // The folder's name is a wall clock on the player's machine: it is read as local time, so
  // the hour shown is the hour in the name, whatever the zone.
  it('reads the wall clock a session folder is named with, as local time', () => {
    const at = new Date(sessionTime('09_12_2026__13_34_26') ?? Number.NaN)
    expect([at.getFullYear(), at.getMonth(), at.getDate()]).toEqual([
      2026, 8, 12,
    ])
    expect([at.getHours(), at.getMinutes(), at.getSeconds()]).toEqual([
      13, 34, 26,
    ])
  })

  it('answers nothing for a name that is not one', () => {
    expect(sessionTime('desyncs')).toBeNull()
    expect(sessionTime('09_12_2026')).toBeNull()
    expect(sessionTime('13_45_2026__13_34_26')).toBeNull()
  })
})

describe('orderRuns', () => {
  // `Live` is first by decision, because it is the launch the app is watching, not because its
  // date compares greater than anything: it may carry none at all.
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

describe('runTime', () => {
  it('dates a session by its name and a launch by its file', () => {
    expect(runTime(run(session('09_12_2026__13_34_26'), 1))).toBe(
      sessionTime('09_12_2026__13_34_26'),
    )
    expect(runTime(run(launch(4, 1_790_000_000), 1))).toBe(1_790_000_000_000)
  })

  it('answers nothing for a launch read before dates were kept', () => {
    expect(runTime(run(launch(4, null), 1))).toBeNull()
  })
})

describe('orderRuns with past launches', () => {
  it('orders past launches among the sessions by date, the live one still first', () => {
    const named = (sessionTime('09_12_2026__13_34_26') ?? 0) / 1000
    const rows = [
      run(launch(1, named - 3600), 1, 'L1'),
      run(session('09_12_2026__13_34_26'), 1, 'S1'),
      run(launch(2, named + 3600), 1, 'L2'),
      run(live, 1, 'NOW'),
    ]
    expect(orderRuns(rows).map((r) => r.seedWords)).toEqual([
      'NOW',
      'L2',
      'S1',
      'L1',
    ])
  })

  // The rule unreadable session names already follow: no date says nothing about when, so the
  // source keeps the place the archive gave it instead of being pushed to an end.
  it('leaves an undated launch where the archive put it', () => {
    const rows = [
      run(session('09_10_2026__10_00_00'), 1, 'OLD'),
      run(launch(1, null), 1, 'UNDATED'),
      run(session('09_12_2026__10_00_00'), 1, 'NEW'),
    ]
    expect(orderRuns(rows).map((r) => r.seedWords)).toEqual([
      'NEW',
      'UNDATED',
      'OLD',
    ])
  })
})

describe('orderRuns with an undated source among dated ones', () => {
  // The archive hands its sources out oldest first and the diary is newest first: the place an
  // undated source keeps is counted from the newest end, or it mirrors. Played between 09-10
  // and 09-11, it must be drawn between them — not under the newest session.
  it('keeps it between its neighbours in time, counted from the newest end', () => {
    const rows = [
      run(session('09_10_2026__10_00_00'), 1, 'S10'),
      run(launch(1, null), 1, 'L'),
      run(session('09_11_2026__10_00_00'), 1, 'S11'),
      run(session('09_12_2026__10_00_00'), 1, 'S12'),
      run(session('09_13_2026__10_00_00'), 1, 'S13'),
    ]
    expect(orderRuns(rows).map((r) => r.seedWords)).toEqual([
      'S13',
      'S12',
      'S11',
      'L',
      'S10',
    ])
  })
})

describe('orderRuns with the live launch under any id', () => {
  // The live launch is first by its kind, not by its key: it shares the key shape of every other
  // launch, and its id may be lower than an older one's in another database.
  it('puts it first whatever its id', () => {
    const rows = [
      run(launch(12, 1_790_000_000), 1, 'OLDER'),
      run({ kind: 'live', id: 2, writtenUnix: null }, 1, 'NOW'),
    ]
    expect(orderRuns(rows).map((r) => r.seedWords)).toEqual(['NOW', 'OLDER'])
  })
})
