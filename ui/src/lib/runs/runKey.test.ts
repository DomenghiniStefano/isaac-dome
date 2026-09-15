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
    expect(runKey(run({ kind: 'live' }, 1))).not.toBe(
      runKey(run({ kind: 'session', name: 'live' }, 1)),
    )
  })
})
