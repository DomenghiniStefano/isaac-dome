import { readFacetFilter, readString } from '@/lib/tabs/tabView'
import type { TabViewSpec } from '@/lib/tabs/tabView'
import {
  CollectionSort,
  collectionFacetOrder,
  defaultCollectionFilter,
} from '@/lib/collection/collectionFacets'
import type {
  CollectionFacet,
  CollectionFilter,
} from '@/lib/collection/collectionFacets'

export interface CollectionReading {
  filter: CollectionFilter
  sort: CollectionSort
}

const sorts: readonly string[] = Object.values(CollectionSort)

// `empty` is the *default* filter and not the empty one: this screen opens on what has not been
// found. `read` never falls back to it — a filter the user cleared is a filter they cleared.
export const collectionView: TabViewSpec<CollectionReading> = {
  empty: () => ({
    filter: defaultCollectionFilter(),
    sort: CollectionSort.Quality,
  }),
  read: (value) => {
    if (typeof value !== 'object' || value === null) return null
    const { filter, sort } = value as { filter?: unknown; sort?: unknown }
    const read = readFacetFilter<CollectionFacet>(filter, collectionFacetOrder)
    if (read === null) return null
    return {
      filter: read,
      sort:
        (readString(sort, sorts) as CollectionSort | null) ??
        CollectionSort.Quality,
    }
  },
}
