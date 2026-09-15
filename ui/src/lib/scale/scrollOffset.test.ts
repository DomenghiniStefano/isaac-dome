import { describe, expect, it } from 'vitest'
import { offsetToApply, readScrollOffset } from './scrollOffset'

describe('an offset against the list it was taken from', () => {
  it('is applied when the list is the length it was', () => {
    expect(offsetToApply({ top: 900, rows: 641 }, 641)).toBe(900)
  })

  // An offset only means something against a list of the same length. A filter that changed
  // underneath — a facet picked in another window, a profile reloaded, an archive that grew —
  // keeps the top rather than guessing where 900 pixels now point (B39).
  it('is refused when the list changed underneath', () => {
    expect(offsetToApply({ top: 900, rows: 641 }, 12)).toBeNull()
  })

  it('is nothing when nothing was stored', () => {
    expect(offsetToApply(null, 641)).toBeNull()
  })

  it('reads back what it wrote, and refuses what it did not', () => {
    expect(readScrollOffset({ top: 900, rows: 641 })).toEqual({
      top: 900,
      rows: 641,
    })
    expect(readScrollOffset({ top: '900', rows: 641 })).toBeNull()
    expect(readScrollOffset(null)).toBeNull()
  })

  // A negative or non-finite offset is not a position: it is a number that arrived from
  // somewhere it should not have.
  it('refuses an offset that is not a position', () => {
    expect(readScrollOffset({ top: -1, rows: 3 })).toBeNull()
    expect(readScrollOffset({ top: Number.NaN, rows: 3 })).toBeNull()
  })
})
