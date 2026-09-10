import { describe, expect, it } from 'vitest'
import { kpiBar } from './kpiBar'

describe('kpiBar', () => {
  it('is the share of the declared denominator, in percent', () => {
    expect(kpiBar(166, 368)).toBeCloseTo((166 / 368) * 100)
  })

  it('has no bar without a denominator', () => {
    expect(kpiBar(5, null)).toBeNull()
  })

  it('has no bar against a denominator of zero', () => {
    expect(kpiBar(5, 0)).toBeNull()
  })

  it('never passes a full bar', () => {
    expect(kpiBar(500, 368)).toBe(100)
  })
})
