import { describe, expect, it } from 'vitest'
import {
  CollectionSort,
  defaultCollectionFilter,
  emptyCollectionFilter,
} from '@/lib/collection/collectionFacets'
import { collectionView } from './tabView'

describe("the Collection's reading", () => {
  // The Collection does **not** open on the empty filter: it opens on what has not been found,
  // and that is the reading a tab with nothing stored starts from.
  it('is the default filter, not the empty one, when there is nothing to read', () => {
    expect(collectionView.empty()).toEqual({
      filter: defaultCollectionFilter(),
      sort: CollectionSort.Quality,
      offset: null,
      find: null,
    })
  })

  // And a filter that was deliberately cleared must come back cleared: reading an empty stored
  // filter as "nothing stored" would put the opening filter back on every restart.
  it('reads back a filter that was cleared', () => {
    const reading = {
      filter: emptyCollectionFilter(),
      sort: CollectionSort.Name,
      offset: null,
      find: null,
    }
    expect(collectionView.read(JSON.parse(JSON.stringify(reading)))).toEqual(
      reading,
    )
  })

  it('falls back to the default sort when the stored one is gone', () => {
    expect(
      collectionView.read({ filter: emptyCollectionFilter(), sort: 'byVibes' })
        ?.sort,
    ).toBe(CollectionSort.Quality)
  })

  it('refuses a record that is not a reading', () => {
    expect(collectionView.read({ sort: CollectionSort.Name })).toBeNull()
  })

  // The find bar is part of how the list was being read (#79): open, what it was looking for, and
  // which match it was on. It used to be three refs, and a tab switch closed it.
  it('reads back an open find bar, its words and the match it was on', () => {
    const reading = {
      filter: emptyCollectionFilter(),
      sort: CollectionSort.Name,
      offset: null,
      find: { query: 'sacred', current: '331' },
    }
    expect(collectionView.read(JSON.parse(JSON.stringify(reading)))).toEqual(
      reading,
    )
  })

  it('reads a find bar that does not read as a closed one, and keeps the rest', () => {
    const read = collectionView.read({
      filter: emptyCollectionFilter(),
      sort: CollectionSort.Name,
      find: { query: 42 },
    })
    expect(read?.find).toBeNull()
    expect(read?.sort).toBe(CollectionSort.Name)
  })

  it('reads a match that is not a key as no match, and keeps the words', () => {
    expect(
      collectionView.read({
        filter: emptyCollectionFilter(),
        find: { query: 'sacred', current: 331 },
      })?.find,
    ).toEqual({ query: 'sacred', current: null })
  })
})
