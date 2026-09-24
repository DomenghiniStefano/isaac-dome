import { describe, expect, it } from 'vitest'
import { sameLayout } from './layout'

const open = { sidebarWidth: 240, sidebarCollapsed: false }

describe('sameLayout', () => {
  // Nothing announced yet: whatever this window holds is news to the others.
  it('is never the same as nothing', () => {
    expect(sameLayout(null, open)).toBe(false)
  })

  it('is the same when both halves are', () => {
    expect(sameLayout(open, { ...open })).toBe(true)
  })

  // Folding changes no width, and a watcher that compared the width alone would never say so.
  it('tells a fold from no change', () => {
    expect(sameLayout(open, { ...open, sidebarCollapsed: true })).toBe(false)
  })

  it('tells a drag from no change', () => {
    expect(sameLayout(open, { ...open, sidebarWidth: 260 })).toBe(false)
  })

  it('tells a width nobody set from one somebody did', () => {
    expect(sameLayout({ ...open, sidebarWidth: null }, open)).toBe(false)
  })
})
