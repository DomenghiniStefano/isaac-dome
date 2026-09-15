import { readFacetFilter, readString } from '@/lib/tabs/tabView'
import { readScrollOffset } from '@/lib/scale/scrollOffset'
import type { ScrollOffset } from '@/lib/scale/scrollOffset'
import type { TabViewSpec } from '@/lib/tabs/tabView'
import {
  UnlockSort,
  facetOrder,
  unlockFaceting,
} from '@/lib/graph/unlockFacets'
import type { FacetId, UnlockFilter } from '@/lib/graph/unlockFacets'

export interface UnlockReading {
  filter: UnlockFilter
  sort: UnlockSort
  offset: ScrollOffset | null
}

const sorts: readonly string[] = Object.values(UnlockSort)

// The filter is the part a reader can refuse; the sort is a value out of a closed set, and a
// stored one that has left the set falls back to the default rather than reaching `sortNodes` as
// a string it has no branch for.
export const unlockView: TabViewSpec<UnlockReading> = {
  empty: () => ({
    filter: unlockFaceting.empty(),
    sort: UnlockSort.FanOut,
    offset: null,
  }),
  read: (value) => {
    if (typeof value !== 'object' || value === null) return null
    const { filter, sort, offset } = value as {
      filter?: unknown
      sort?: unknown
      offset?: unknown
    }
    const read = readFacetFilter<FacetId>(filter, facetOrder)
    if (read === null) return null
    return {
      filter: read,
      sort: (readString(sort, sorts) as UnlockSort | null) ?? UnlockSort.FanOut,
      offset: readScrollOffset(offset),
    }
  },
}
