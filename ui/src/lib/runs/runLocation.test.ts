import { describe, expect, it } from 'vitest'
import type { RunView } from '@/lib/ipc/types'
import { RouteName } from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'
import type { Tab } from '@/stores/tabModel'
import { listIsBehind, runKeyOf, runLocation } from './runLocation'

const run = {
  source: { kind: 'live', id: 3, writtenUnix: null },
  ordinal: 2,
} as RunView

describe("a run's place", () => {
  // The page is the Runs screen with a `run` query, as a wiki page is the Wiki screen with a
  // `page` one: no route of its own, and a tab can hold it.
  it('is the runs screen with the run named in the query', () => {
    expect(runLocation(run)).toEqual({
      name: RouteName.Runs,
      query: { run: 'log:3#2' },
    })
  })

  it('reads the run back from a query', () => {
    expect(runKeyOf({ run: 'log:3#2' })).toBe('log:3#2')
  })

  // The router hands a repeated parameter as an array and a missing one as undefined; neither
  // names one run.
  it('reads no run from a query that does not name exactly one', () => {
    expect(runKeyOf({})).toBeNull()
    expect(runKeyOf({ run: ['a', 'b'] })).toBeNull()
    expect(runKeyOf({ run: '' })).toBeNull()
  })
})

describe('the way back from a run', () => {
  const tab = (locations: TabLocation[], index: number): Tab => ({
    id: 't',
    entries: locations.map((location) => ({ location })),
    index,
  })
  const list: TabLocation = { name: RouteName.Runs }
  const page: TabLocation = { name: RouteName.Runs, query: { run: 'log:3#2' } }

  // Back to the list the run was opened from, where its page box kept its place — not a new entry
  // that opens the list at the top.
  it('is back when the entry before is the list', () => {
    expect(listIsBehind(tab([list, page], 1))).toBe(true)
  })

  it('is not back when the entry before is another run, another screen, or nothing', () => {
    expect(listIsBehind(tab([page, page], 1))).toBe(false)
    expect(listIsBehind(tab([{ name: RouteName.Unlock }, page], 1))).toBe(false)
    expect(listIsBehind(tab([page], 0))).toBe(false)
    expect(listIsBehind(undefined)).toBe(false)
  })
})
