import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { effectScope } from 'vue'
import { msUntilTomorrow, useToday } from './useToday'

describe('msUntilTomorrow', () => {
  it('counts to the next local midnight', () => {
    expect(msUntilTomorrow(new Date(2026, 8, 25, 23, 59, 0))).toBe(60_000)
    expect(msUntilTomorrow(new Date(2026, 8, 25, 0, 0, 0))).toBe(24 * 3_600_000)
  })
})

describe('useToday', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    vi.setSystemTime(new Date(2026, 8, 25, 23, 59, 0))
  })
  afterEach(() => {
    vi.useRealTimers()
  })

  // Card #80, P10: the app lives in the tray, and "today" read once at mount stays today
  // for as long as the process does.
  it('moves on at midnight, and again the midnight after', () => {
    const scope = effectScope()
    const today = scope.run(() => useToday())!
    expect(today.value.getDate()).toBe(25)
    vi.advanceTimersByTime(60_000)
    expect(today.value.getDate()).toBe(26)
    vi.advanceTimersByTime(24 * 3_600_000)
    expect(today.value.getDate()).toBe(27)
    scope.stop()
  })

  it('stops when its scope does', () => {
    const scope = effectScope()
    scope.run(() => useToday())
    scope.stop()
    expect(vi.getTimerCount()).toBe(0)
  })
})
