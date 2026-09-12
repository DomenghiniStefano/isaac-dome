import { describe, expect, it } from 'vitest'
import { latest } from './latest'

describe('latest', () => {
  it('keeps only the answer to the newest question', () => {
    // A slow scan must never overwrite a fresher result.
    const guard = latest()
    const first = guard.next()
    const second = guard.next()
    expect(guard.isCurrent(second)).toBe(true)
    expect(guard.isCurrent(first)).toBe(false)
  })
})
