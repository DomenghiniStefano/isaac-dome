import { describe, expect, it } from 'vitest'
import { queryToRecall } from './recall'

describe('queryToRecall', () => {
  it('keeps the search the palette closed on, exactly as it was typed', () => {
    expect(queryToRecall('mom')).toBe('mom')
    expect(queryToRecall('blue baby ')).toBe('blue baby ')
  })

  it('keeps nothing when the palette closed empty', () => {
    expect(queryToRecall('')).toBe('')
  })

  it('keeps nothing when only spaces were typed, so the next opening is empty, not blank-looking', () => {
    expect(queryToRecall('   ')).toBe('')
  })
})
