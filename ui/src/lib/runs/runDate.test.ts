import { describe, expect, it } from 'vitest'
import type { RunView } from '@/lib/ipc/types'
import { RunDateKind, runDate } from './runDate'

const run = (source: RunView['source']) => ({ source, ordinal: 1 }) as RunView

describe("a run's date", () => {
  // The two times mean different things, which is why the tooltip has to say which.
  it('is when a session started', () => {
    const date = runDate(run({ kind: 'session', name: '09_12_2026__13_34_26' }))
    expect(date.kind).toBe(RunDateKind.Started)
    expect(date.kind !== RunDateKind.Undated && date.at.getHours()).toBe(13)
  })

  it('is when a launch was last written', () => {
    const date = runDate(
      run({ kind: 'launch', id: 1, writtenUnix: 1_790_000_000 }),
    )
    expect(date).toEqual({
      kind: RunDateKind.Written,
      at: new Date(1_790_000_000_000),
    })
  })

  it('is nothing for a launch read before dates were kept, or a name that is not a clock', () => {
    expect(runDate(run({ kind: 'live', id: 1, writtenUnix: null })).kind).toBe(
      RunDateKind.Undated,
    )
    expect(runDate(run({ kind: 'session', name: 'desyncs' })).kind).toBe(
      RunDateKind.Undated,
    )
  })
})
