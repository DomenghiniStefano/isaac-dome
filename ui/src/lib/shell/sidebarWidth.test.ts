import { describe, expect, it } from 'vitest'
import { clampSidebarWidth, shownSidebarWidth } from './sidebarWidth'

describe('clampSidebarWidth', () => {
  it('raises a width below the minimum to the minimum', () => {
    expect(clampSidebarWidth(100)).toBe(168)
  })

  it('keeps the minimum itself', () => {
    expect(clampSidebarWidth(168)).toBe(168)
  })

  it('keeps a width inside the bounds', () => {
    expect(clampSidebarWidth(300)).toBe(300)
  })

  it('lowers a width above the maximum to the maximum', () => {
    expect(clampSidebarWidth(500)).toBe(420)
  })

  it('falls back to the default for a width that is not a number', () => {
    expect(clampSidebarWidth(Number.NaN)).toBe(212)
  })

  it('rounds to a whole pixel', () => {
    expect(clampSidebarWidth(250.6)).toBe(251)
  })
})

describe('shownSidebarWidth', () => {
  it('is the default while nobody has sized the sidebar', () => {
    expect(shownSidebarWidth(null)).toBe(212)
  })

  it('is a stored width inside the bounds, as stored', () => {
    expect(shownSidebarWidth(300)).toBe(300)
  })

  it('brings a width stored under other bounds back inside these', () => {
    expect(shownSidebarWidth(9000)).toBe(420)
    expect(shownSidebarWidth(10)).toBe(168)
  })
})
