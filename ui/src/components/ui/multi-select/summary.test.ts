import { describe, expect, it } from 'vitest'
import { pickedSummary } from './summary'

describe('pickedSummary', () => {
  it('says nothing when nothing is picked, so the trigger reads as the facet alone', () => {
    expect(pickedSummary([])).toBeNull()
  })

  it('names one pick, and two', () => {
    expect(pickedSummary(['3'])).toBe('3')
    expect(pickedSummary(['3', '4'])).toBe('3, 4')
  })

  // A trigger is a button on a wrapping row, not a paragraph: past two the count says more
  // than three truncated names would, and the chips row underneath spells them all out.
  it('stops at two and counts the rest', () => {
    expect(pickedSummary(['angel', 'boss', 'devil'])).toBe('angel, boss +1')
    expect(pickedSummary(['a', 'b', 'c', 'd', 'e'])).toBe('a, b +3')
  })
})
