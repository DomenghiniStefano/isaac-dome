import { countBy, sortBy, sumBy } from 'lodash-es'
import { assertNever } from '@/lib/assertNever'
import { OriginValue } from '@/lib/graph/unlockFilter'
import type { CollectionItem } from '@/lib/ipc/types'
import { ItemKindView } from '@/lib/ipc/types'
import { ItemState, itemState, itemStateOrder } from './itemState'

// The Collection's facets: only what the contract answers (quality and pools from the game's
// files, the origin from the id ranges, the state from the save).
export const CollectionFacet = {
  State: 'state',
  Quality: 'quality',
  Pool: 'pool',
  Kind: 'kind',
  Origin: 'origin',
} as const
export type CollectionFacet =
  (typeof CollectionFacet)[keyof typeof CollectionFacet]

export const collectionFacetOrder: CollectionFacet[] = [
  CollectionFacet.State,
  CollectionFacet.Quality,
  CollectionFacet.Pool,
  CollectionFacet.Kind,
  CollectionFacet.Origin,
]

// items_metadata.xml rates collectibles 0 to 4; an item it doesn't rate is a value of its own.
export const QualityValue = {
  Four: '4',
  Three: '3',
  Two: '2',
  One: '1',
  Zero: '0',
  Unrated: 'unrated',
} as const
export type QualityValue = (typeof QualityValue)[keyof typeof QualityValue]

const qualityOrder: string[] = Object.values(QualityValue)

// Pool names are the game's own strings; an item in none is a value of its own.
export const NoPool = 'none'

const kindOrder: string[] = [
  ItemKindView.Passive,
  ItemKindView.Active,
  ItemKindView.Familiar,
]
const originOrder: string[] = Object.values(OriginValue)

export const CollectionSort = {
  Quality: 'quality',
  Id: 'id',
  Name: 'name',
} as const
export type CollectionSort =
  (typeof CollectionSort)[keyof typeof CollectionSort]

export interface CollectionFilter {
  query: string
  picks: Record<CollectionFacet, string[]>
}

export const emptyCollectionFilter = (): CollectionFilter => ({
  query: '',
  picks: {
    [CollectionFacet.State]: [],
    [CollectionFacet.Quality]: [],
    [CollectionFacet.Pool]: [],
    [CollectionFacet.Kind]: [],
    [CollectionFacet.Origin]: [],
  },
})

// The screen opens on what hasn't been found: what can be found tonight, and what can't yet.
export const defaultCollectionFilter = (): CollectionFilter => {
  const empty = emptyCollectionFilter()
  return {
    ...empty,
    picks: {
      ...empty.picks,
      [CollectionFacet.State]: [ItemState.Available, ItemState.Locked],
    },
  }
}

// The filter a list opened on a name starts from: the name, and nothing else. The screen's
// default states are a convenience, and a search result is a request — an item asked for by
// name is often already in the collection, and keeping the default would answer "0 of 721" to
// a row the user just clicked (spec 3.5, Decision 8).
export const filterForQuery = (query: string): CollectionFilter => ({
  ...emptyCollectionFilter(),
  query,
})

// An item's values for one facet. It matches a facet when any of them is picked: an item can sit
// in several pools.
export const collectionFacetValues = (
  item: CollectionItem,
  facet: CollectionFacet,
): string[] => {
  switch (facet) {
    case CollectionFacet.State:
      return [itemState(item)]
    case CollectionFacet.Quality:
      return [
        item.quality === null ? QualityValue.Unrated : String(item.quality),
      ]
    case CollectionFacet.Pool:
      return item.pools.length > 0 ? item.pools : [NoPool]
    case CollectionFacet.Kind:
      return [item.kind]
    case CollectionFacet.Origin:
      return [item.origin ?? OriginValue.None]
    default:
      return assertNever(facet)
  }
}

const matchesQuery = (item: CollectionItem, query: string): boolean => {
  const wanted = query.trim().toLowerCase()
  return wanted === '' || item.name.toLowerCase().includes(wanted)
}

const matchesFacet = (
  item: CollectionItem,
  facet: CollectionFacet,
  picked: string[],
): boolean =>
  picked.length === 0 ||
  collectionFacetValues(item, facet).some((value) => picked.includes(value))

const matchesFacets = (
  item: CollectionItem,
  filter: CollectionFilter,
  facets: CollectionFacet[],
): boolean =>
  matchesQuery(item, filter.query) &&
  facets.every((facet) => matchesFacet(item, facet, filter.picks[facet]))

// Any value within a facet, every facet at once, and the search.
export const matchesCollectionFilter = (
  item: CollectionItem,
  filter: CollectionFilter,
): boolean => matchesFacets(item, filter, collectionFacetOrder)

// A value's count leaves its own facet out: it says how many rows picking it would give.
export const collectionFacetCounts = (
  items: CollectionItem[],
  filter: CollectionFilter,
  facet: CollectionFacet,
): Map<string, number> => {
  const others = collectionFacetOrder.filter((f) => f !== facet)
  const values = items
    .filter((item) => matchesFacets(item, filter, others))
    .flatMap((item) => collectionFacetValues(item, facet))
  return new Map(Object.entries(countBy(values)))
}

// A facet's values in the order they are offered. The pools are the view's: the ones its items
// belong to, in the catalog's order.
export const collectionFacetOptions = (
  pools: string[],
  facet: CollectionFacet,
): string[] => {
  switch (facet) {
    case CollectionFacet.State:
      return itemStateOrder
    case CollectionFacet.Quality:
      return qualityOrder
    case CollectionFacet.Pool:
      return [...pools, NoPool]
    case CollectionFacet.Kind:
      return kindOrder
    case CollectionFacet.Origin:
      return originOrder
    default:
      return assertNever(facet)
  }
}

// Every order ends on the id, so equal rows never swap between two renders. An unrated item
// goes after quality 0.
export const sortItems = (
  items: CollectionItem[],
  sort: CollectionSort,
): CollectionItem[] => {
  switch (sort) {
    case CollectionSort.Quality:
      return sortBy(items, [(i) => -(i.quality ?? -1), (i) => i.id])
    case CollectionSort.Id:
      return sortBy(items, (i) => i.id)
    case CollectionSort.Name:
      return sortBy(items, [(i) => i.name.toLowerCase(), (i) => i.id])
    default:
      return assertNever(sort)
  }
}

// How many values are picked across the facets; the search is shown on its own.
export const activeCollectionFilterCount = (filter: CollectionFilter): number =>
  sumBy(collectionFacetOrder, (facet) => filter.picks[facet].length)
