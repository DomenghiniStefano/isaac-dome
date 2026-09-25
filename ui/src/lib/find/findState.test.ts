import { describe, expect, it } from 'vitest'
import { findWithCurrent, findWithQuery, openFind } from './findState'

// The find bar as a tab keeps it: `null` is the bar closed, an object is the bar open with its
// words and the match it is on.
describe('the find bar, as the tab keeps it', () => {
  it('opens empty when it was closed', () => {
    expect(openFind(null)).toEqual({ query: '', current: null })
  })

  // Ctrl+F on a bar already open is not a reset: the words stay.
  it('stays as it was when it is already open', () => {
    const open = { query: 'onion', current: '1' }
    expect(openFind(open)).toBe(open)
  })

  it('takes new words and keeps the match it was on', () => {
    expect(findWithQuery({ query: 'on', current: '7' }, 'onion')).toEqual({
      query: 'onion',
      current: '7',
    })
  })

  it('moves to another match and keeps its words', () => {
    expect(findWithCurrent({ query: 'onion', current: '7' }, '9')).toEqual({
      query: 'onion',
      current: '9',
    })
  })

  // A write that arrives with the bar closed opens it: the field is what was typed into.
  it('opens on a write that arrives while it is closed', () => {
    expect(findWithQuery(null, 'eye')).toEqual({ query: 'eye', current: null })
    expect(findWithCurrent(null, '3')).toEqual({ query: '', current: '3' })
  })
})
