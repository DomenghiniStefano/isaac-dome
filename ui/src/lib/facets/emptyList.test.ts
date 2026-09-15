import { describe, expect, it } from 'vitest'
import { emptyList, isFiltering, queryTyped } from './emptyList'

const text = { empty: 'x.empty', noResults: 'x.noResults' } as const

describe('emptyList', () => {
  it('says the list is empty when it holds nothing at all', () => {
    expect(emptyList(0, false, text).text).toBe('x.empty')
  })

  it('offers no reset where nothing is filtering', () => {
    expect(emptyList(0, false, text).reset).toBe(false)
  })

  it('says nothing matched when a filter emptied a list that has rows', () => {
    expect(emptyList(16, true, text)).toEqual({
      text: 'x.noResults',
      reset: true,
    })
  })

  // The two halves answer different questions, so they are read separately: a filter typed
  // into an empty list still finds nothing there, and still has something to clear.
  it('still names the empty list when a filter was set on it', () => {
    expect(emptyList(0, true, text)).toEqual({
      text: 'x.empty',
      reset: true,
    })
  })
})

describe('queryTyped', () => {
  it('reads a query of spaces as nothing typed, the way the filter does', () => {
    expect(queryTyped('   ')).toBe(false)
    expect(queryTyped('')).toBe(false)
    expect(queryTyped('zzz')).toBe(true)
  })
})

describe('isFiltering', () => {
  const filter = (query: string, picks: Record<string, string[]>) => ({
    query,
    picks,
  })

  it('is false on a filter that picks nothing and reads no name', () => {
    expect(isFiltering(filter('', { state: [], origin: [] }))).toBe(false)
  })

  it('is true where a facet holds a pick, with no name typed', () => {
    expect(isFiltering(filter('', { state: ['locked'], origin: [] }))).toBe(
      true,
    )
  })

  // The Collection opens on `state: [available, locked]` — a filter nobody set by hand. A
  // reset undoes that too, so it counts: otherwise the screen's own opening filter would hide
  // the button that clears it.
  it('counts the filter a screen opens on, which a reset also undoes', () => {
    expect(
      isFiltering(filter('', { state: ['available', 'locked'], origin: [] })),
    ).toBe(true)
  })

  it('is true where a name is typed and no facet is picked', () => {
    expect(isFiltering(filter('breakfast', { state: [], origin: [] }))).toBe(
      true,
    )
  })
})
