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
    })
  })

  // And a filter that was deliberately cleared must come back cleared: reading an empty stored
  // filter as "nothing stored" would put the opening filter back on every restart.
  it('reads back a filter that was cleared', () => {
    const reading = {
      filter: emptyCollectionFilter(),
      sort: CollectionSort.Name,
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
})
