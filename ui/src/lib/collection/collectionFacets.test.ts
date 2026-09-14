import { describe, expect, it } from 'vitest'
import type { CollectionItem, LockView } from '@/lib/ipc/types'
import type { CollectionFilter } from './collectionFacets'
import {
  CollectionFacet,
  CollectionSort,
  collectionFaceting,
  collectionFacetValues,
  defaultCollectionFilter,
  emptyCollectionFilter,
  filterForQuery,
  sortItems,
} from './collectionFacets'

const item = (over: Partial<CollectionItem> = {}): CollectionItem => ({
  id: 1,
  kind: 'passive',
  name: 'The Sad Onion',
  iconUrl: null,
  quality: 3,
  pools: ['treasure'],
  origin: 'rebirth',
  inCollection: false,
  lock: { kind: 'free' },
  ...over,
})
const locked: LockView = {
  kind: 'locked',
  achievement: 62,
  text: null,
  page: null,
}

const sadOnion = item({ id: 1 })
const innerEye = item({
  id: 2,
  name: 'The Inner Eye',
  quality: 2,
  pools: ['treasure', 'boss'],
  inCollection: true,
})
const epicFetus = item({
  id: 168,
  name: 'Epic Fetus',
  quality: 4,
  pools: [],
  lock: locked,
})
const unrated = item({
  id: 600,
  name: 'Unrated',
  quality: null,
  pools: ['devil'],
  origin: null,
  kind: 'familiar',
})
const rows = [sadOnion, innerEye, epicFetus, unrated]

const ids = (xs: CollectionItem[]) => xs.map((x) => x.id)
// The pools a view of these four items would carry, in the catalog's order.
const faceting = collectionFaceting(['treasure', 'boss'])
const matching = (filter: CollectionFilter) =>
  ids(rows.filter((r) => faceting.matches(r, filter)))

describe('the Collection facets', () => {
  it('gives each item its values', () => {
    expect(collectionFacetValues(unrated, CollectionFacet.Quality)).toEqual([
      'unrated',
    ])
    expect(collectionFacetValues(epicFetus, CollectionFacet.Pool)).toEqual([
      'none',
    ])
    expect(collectionFacetValues(innerEye, CollectionFacet.Pool)).toEqual([
      'treasure',
      'boss',
    ])
    expect(collectionFacetValues(unrated, CollectionFacet.Origin)).toEqual([
      'none',
    ])
    expect(collectionFacetValues(unrated, CollectionFacet.Kind)).toEqual([
      'familiar',
    ])
    expect(collectionFacetValues(epicFetus, CollectionFacet.State)).toEqual([
      'locked',
    ])
  })

  // How matching and counting work is `lib/facets/faceting.test.ts`; what is the Collection's
  // is *which* field the search reads — the name alone, where Unlock joins three.
  it('searches the name, whatever the case', () => {
    expect(matching({ ...emptyCollectionFilter(), query: 'EYE' })).toEqual([2])
    expect(matching({ ...emptyCollectionFilter(), query: 'treasure' })).toEqual(
      [],
    )
  })

  it('opens on what has not been found: to find, and locked', () => {
    expect(matching(defaultCollectionFilter())).toEqual([1, 168, 600])
  })

  it("offers the view's pools and then no pool, and the qualities from 4 down", () => {
    expect(faceting.options(rows, CollectionFacet.Pool)).toEqual([
      'treasure',
      'boss',
      'none',
    ])
    expect(faceting.options(rows, CollectionFacet.Quality)).toEqual([
      '4',
      '3',
      '2',
      '1',
      '0',
      'unrated',
    ])
  })

  it('sorts by quality, by id and by name', () => {
    expect(ids(sortItems(rows, CollectionSort.Quality))).toEqual([
      168, 1, 2, 600,
    ])
    expect(ids(sortItems([...rows].reverse(), CollectionSort.Id))).toEqual([
      1, 2, 168, 600,
    ])
    expect(ids(sortItems(rows, CollectionSort.Name))).toEqual([168, 2, 1, 600])
  })
})

describe('a list opened on a name', () => {
  it('shows that name, and drops the state the screen defaults to', () => {
    // The Collection opens on "to find" and "locked" as a convenience. A search result asks
    // for one item by name, and that item is often already in the collection: keeping the
    // default would answer "0 of 721" to a row the user just clicked.
    const filter = filterForQuery('Brimstone Bombs')
    expect(filter.query).toBe('Brimstone Bombs')
    expect(filter.picks[CollectionFacet.State]).toEqual([])
    expect(defaultCollectionFilter().picks[CollectionFacet.State]).not.toEqual(
      [],
    )
  })
})
