import { describe, expect, it } from 'vitest'
import { readFacetFilter, readString, readStringArray } from './tabView'

const order = ['state', 'origin'] as const

describe('reading a value written by an older version of this app', () => {
  it('takes a string only from the set that still exists', () => {
    expect(readString('name', ['name', 'id'])).toBe('name')
    expect(readString('fanOut', ['name', 'id'])).toBeNull()
    expect(readString(3, ['name'])).toBeNull()
  })

  it('takes an array of strings, and nothing else', () => {
    expect(readStringArray(['a', 'b'])).toEqual(['a', 'b'])
    expect(readStringArray([])).toEqual([])
    expect(readStringArray(['a', 2])).toBeNull()
    expect(readStringArray('a')).toBeNull()
  })

  // A value that was legal when it was written and is not a facet value any more: dropped,
  // rather than kept as a pick that filters everything away and reads as an empty profile.
  it('drops an array value the screen no longer knows', () => {
    expect(readStringArray(['now', 'gone'], ['now'])).toEqual(['now'])
  })

  it('reads a facet filter and fills the facets the document never had', () => {
    expect(
      readFacetFilter({ query: 'brim', picks: { state: ['now'] } }, order),
    ).toEqual({ query: 'brim', picks: { state: ['now'], origin: [] } })
  })

  // The facets are what `order` says they are: a key from a facet that no longer exists is not
  // part of the faceting, and carrying it would hand the engine a `picks` it cannot index.
  it('drops a facet the screen no longer has', () => {
    expect(
      readFacetFilter({ query: '', picks: { gone: ['x'] } }, order)?.picks,
    ).toEqual({ state: [], origin: [] })
  })

  it('refuses something that is not a filter at all', () => {
    expect(readFacetFilter(null, order)).toBeNull()
    expect(readFacetFilter({ query: 7 }, order)).toBeNull()
  })
})
