import { describe, expect, it } from 'vitest'
import { progressShares } from './progressShares'

describe('progressShares', () => {
  it('turns value and unknown into shares of max', () => {
    expect(progressShares(25, 10, 100)).toEqual({ value: 25, unknown: 10 })
  })

  it('scales to a max other than 100', () => {
    expect(progressShares(1, 1, 4)).toEqual({ value: 25, unknown: 25 })
  })

  it('clamps negative inputs to zero', () => {
    expect(progressShares(-5, -3, 100)).toEqual({ value: 0, unknown: 0 })
  })

  it('clamps a value beyond max and leaves no room for unknown', () => {
    expect(progressShares(150, 10, 100)).toEqual({ value: 100, unknown: 0 })
  })

  it('gives unknown only the room the value leaves', () => {
    expect(progressShares(80, 40, 100)).toEqual({ value: 80, unknown: 20 })
  })

  it('is empty, not NaN, when there is nothing to measure', () => {
    expect(progressShares(5, 5, 0)).toEqual({ value: 0, unknown: 0 })
  })
})
