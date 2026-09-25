import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { replaceableTimeout } from './replaceableTimeout'

beforeEach(() => {
  vi.useFakeTimers()
})
afterEach(() => {
  vi.useRealTimers()
})

describe('replaceableTimeout', () => {
  it('runs once its time is up', () => {
    const timeout = replaceableTimeout()
    const run = vi.fn()
    timeout.set(run, 100)
    vi.advanceTimersByTime(99)
    expect(run).not.toHaveBeenCalled()
    vi.advanceTimersByTime(1)
    expect(run).toHaveBeenCalledTimes(1)
  })

  it('a new one replaces the one pending, and starts the time again', () => {
    const timeout = replaceableTimeout()
    const first = vi.fn()
    const second = vi.fn()
    timeout.set(first, 100)
    vi.advanceTimersByTime(60)
    timeout.set(second, 100)
    vi.advanceTimersByTime(60)
    expect(second).not.toHaveBeenCalled()
    vi.advanceTimersByTime(40)
    expect(first).not.toHaveBeenCalled()
    expect(second).toHaveBeenCalledTimes(1)
  })

  it('called off, runs nothing', () => {
    const timeout = replaceableTimeout()
    const run = vi.fn()
    timeout.set(run, 100)
    timeout.clear()
    timeout.clear()
    vi.advanceTimersByTime(200)
    expect(run).not.toHaveBeenCalled()
  })
})
