import { describe, expect, it } from 'vitest'
import { hasKpiBar } from './kpiBar'

describe('hasKpiBar', () => {
  it('draws a bar against a declared denominator', () => {
    expect(hasKpiBar(368)).toBe(true)
  })

  it('draws no bar without a denominator', () => {
    expect(hasKpiBar(null)).toBe(false)
  })

  it('draws no bar against a denominator of zero', () => {
    expect(hasKpiBar(0)).toBe(false)
  })

  it('draws no bar against a denominator that is not a number', () => {
    expect(hasKpiBar(Number.NaN)).toBe(false)
  })
})
