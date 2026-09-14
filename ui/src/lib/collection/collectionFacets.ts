import { sortBy } from 'lodash-es'
import { assertNever } from '@/lib/assertNever'
import { createFaceting, emptyFilter } from '@/lib/facets/faceting'
import type { FacetFilter } from '@/lib/facets/faceting'
import type { CollectionItem } from '@/lib/ipc/types'
import { ItemKindView } from '@/lib/ipc/types'
import { OriginValue, originOrder } from '@/lib/ipc/values'
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

export const CollectionSort = {
  Quality: 'quality',
  Id: 'id',
  Name: 'name',
} as const
export type CollectionSort =
  (typeof CollectionSort)[keyof typeof CollectionSort]

export type CollectionFilter = FacetFilter<CollectionFacet>

export const emptyCollectionFilter = (): CollectionFilter =>
  emptyFilter(collectionFacetOrder)

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

// A facet's values in the order they are offered. The pools are the view's — the catalog's own
// order, filtered to the ones its items belong to — and that is why this is a factory and not a
// constant: the order cannot be read back off the items, only the set can.
const collectionFacetOptions = (
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

// The Collection's half of a faceted list. Matching, the counts and the active count are the
// engine's; what is here is the five facets, how an item answers one, the name the search
// reads, and the options — which need the view's pools, so this takes them.
export const collectionFaceting = (pools: string[]) =>
  createFaceting<CollectionItem, CollectionFacet>({
    order: collectionFacetOrder,
    values: collectionFacetValues,
    text: (item) => item.name,
    options: (_items, facet) => collectionFacetOptions(pools, facet),
  })

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
