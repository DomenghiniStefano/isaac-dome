import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { newWindowLabel, nextMint } from './windowPort'
import { mintedAt, windowOrder } from './sessionWriter'

describe('the time a label is minted at', () => {
  it('is the clock, when the clock has moved on', () => {
    expect(nextMint(1_757_900_000_005, 1_757_900_000_001)).toBe(
      1_757_900_000_005,
    )
  })

  it('is one past the last, when the clock has not', () => {
    expect(nextMint(1_757_900_000_001, 1_757_900_000_001)).toBe(
      1_757_900_000_002,
    )
    // A clock set back is the same case: never a label already given out.
    expect(nextMint(1_757_899_999_000, 1_757_900_000_001)).toBe(
      1_757_900_000_002,
    )
  })
})

describe('newWindowLabel', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    vi.setSystemTime(1_757_900_000_001)
  })
  afterEach(() => {
    vi.useRealTimers()
  })

  // Card #80, R8: `reopen` mints one label per restored window in a loop, and two in the same
  // millisecond were the same label — the second window Tauri refuses to create.
  it('mints two different labels in the same millisecond, in the order minted', () => {
    const labels = [newWindowLabel(), newWindowLabel(), newWindowLabel()]
    expect(new Set(labels).size).toBe(3)
    expect(windowOrder([...labels].reverse())).toEqual(labels)
    // Still the shape the session and the tray read (`crates/ipc/src/tray.rs`).
    expect(labels.every((l) => mintedAt(l) !== null)).toBe(true)
  })
})
