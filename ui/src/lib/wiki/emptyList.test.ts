import { describe, expect, it } from 'vitest'
import { emptyList } from './emptyList'

describe('emptyList', () => {
  it('says the category is empty when it holds no page at all', () => {
    expect(emptyList(0, '').text).toBe('wiki.emptyCategory')
  })

  it('offers no reset where nothing was ever typed', () => {
    expect(emptyList(0, '').reset).toBe(false)
  })

  it('says no page has this name when a query emptied a category that has pages', () => {
    expect(emptyList(16, 'zzz')).toEqual({
      text: 'wiki.noResults',
      reset: true,
    })
  })

  it('still names the empty category when a query was typed into it', () => {
    expect(emptyList(0, 'zzz')).toEqual({
      text: 'wiki.emptyCategory',
      reset: true,
    })
  })

  it('reads a query of spaces as nothing typed, the way the filter does', () => {
    expect(emptyList(16, '   ').reset).toBe(false)
  })
})
