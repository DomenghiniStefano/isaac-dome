import { describe, expect, it } from 'vitest'
import type { RunView } from '@/lib/ipc/types'
import { runKey } from './runKey'

const run = (source: RunView['source'], ordinal: number) =>
  ({ source, ordinal }) as RunView

describe("a run's key", () => {
  it('is its source and its position in that source', () => {
    expect(runKey(run({ kind: 'session', name: '09_14' }, 2))).toBe(
      'session:09_14#2',
    )
  })

  // A launch of `log.txt` has no name — that is why the store keys it `NULL` — so the kind is
  // what tells it from a session, and the two must never collide.
  it('tells a launch from a session that could be named like one', () => {
    expect(runKey(run({ kind: 'live', id: 1, writtenUnix: null }, 1))).not.toBe(
      runKey(run({ kind: 'session', name: 'live' }, 1)),
    )
  })

  // Every launch numbers its runs from one, so two launches' first runs must still differ.
  it('tells two launches apart', () => {
    expect(
      runKey(run({ kind: 'launch', id: 1, writtenUnix: null }, 1)),
    ).not.toBe(runKey(run({ kind: 'launch', id: 2, writtenUnix: null }, 1)))
  })
})

describe("a launch's key when a newer launch arrives", () => {
  // The latest launch is `live` and the one before it `launch`, with the same row id: the run it
  // holds is the same run, and a tab or a link that named it must still find it.
  it('stays the same', () => {
    expect(runKey(run({ kind: 'live', id: 3, writtenUnix: null }, 1))).toBe(
      runKey(run({ kind: 'launch', id: 3, writtenUnix: null }, 1)),
    )
    expect(runKey(run({ kind: 'live', id: 3, writtenUnix: null }, 1))).toBe(
      'log:3#1',
    )
  })
})
