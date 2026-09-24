import { readFacetFilter, readString } from '@/lib/tabs/tabView'
import { readScrollOffset } from '@/lib/scale/scrollOffset'
import type { ScrollOffset } from '@/lib/scale/scrollOffset'
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
  offset: ScrollOffset | null
  find: CollectionFind | null
}

// The find bar (B67), open: what it is looking for and which match it is on. `null` is the bar
// closed. It was three refs on the screen (#79), and a tab switch closed it with the words in it.
export interface CollectionFind {
  query: string
  current: string | null
}

const readFind = (value: unknown): CollectionFind | null => {
  if (typeof value !== 'object' || value === null) return null
  const { query, current } = value as { query?: unknown; current?: unknown }
  if (typeof query !== 'string') return null
  return { query, current: typeof current === 'string' ? current : null }
}

const sorts: readonly string[] = Object.values(CollectionSort)

// `empty` is the *default* filter and not the empty one: this screen opens on what has not been
// found. `read` never falls back to it — a filter the user cleared is a filter they cleared.
export const collectionView: TabViewSpec<CollectionReading> = {
  empty: () => ({
    filter: defaultCollectionFilter(),
    sort: CollectionSort.Quality,
    offset: null,
    find: null,
  }),
  read: (value) => {
    if (typeof value !== 'object' || value === null) return null
    const { filter, sort, offset, find } = value as {
      filter?: unknown
      sort?: unknown
      offset?: unknown
      find?: unknown
    }
    const read = readFacetFilter<CollectionFacet>(filter, collectionFacetOrder)
    if (read === null) return null
    return {
      filter: read,
      sort:
        (readString(sort, sorts) as CollectionSort | null) ??
        CollectionSort.Quality,
      offset: readScrollOffset(offset),
      find: readFind(find),
    }
  },
}
