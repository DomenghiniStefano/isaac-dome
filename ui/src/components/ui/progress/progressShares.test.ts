import { describe, expect, it } from 'vitest'
import {
  progressShares,
  progressBounds,
  defaultProgressMax,
} from './progressShares'

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

  it('treats a non-finite value or unknown as zero', () => {
    expect(progressShares(NaN, 10, 100)).toEqual({ value: 0, unknown: 10 })
    expect(progressShares(25, Number.POSITIVE_INFINITY, 100)).toEqual({
      value: 25,
      unknown: 0,
    })
  })

  it('is empty, not NaN, when max is not a finite positive number', () => {
    expect(progressShares(5, 5, NaN)).toEqual({ value: 0, unknown: 0 })
    expect(progressShares(5, 5, Number.POSITIVE_INFINITY)).toEqual({
      value: 0,
      unknown: 0,
    })
    expect(progressShares(5, 5, -10)).toEqual({ value: 0, unknown: 0 })
  })
})

describe('progressBounds', () => {
  it('keeps a value inside a valid max', () => {
    expect(progressBounds(40, 100)).toEqual({ value: 40, max: 100 })
  })

  it('clamps a value beyond max, and a negative or non-finite value to zero', () => {
    expect(progressBounds(150, 100)).toEqual({ value: 100, max: 100 })
    expect(progressBounds(-5, 100)).toEqual({ value: 0, max: 100 })
    expect(progressBounds(NaN, 100)).toEqual({ value: 0, max: 100 })
  })

  it('falls back to an empty bar on the default max when max is unusable', () => {
    expect(progressBounds(5, 0)).toEqual({ value: 0, max: defaultProgressMax })
    expect(progressBounds(5, NaN)).toEqual({
      value: 0,
      max: defaultProgressMax,
    })
  })
})
