import { readFacetFilter } from '@/lib/tabs/tabView'
import type { TabViewSpec } from '@/lib/tabs/tabView'
import { readScrollOffset } from '@/lib/scale/scrollOffset'
import type { ScrollOffset } from '@/lib/scale/scrollOffset'
import {
  WikiSort,
  emptyWikiListFilter,
  wikiFacetOrder,
} from '@/lib/wiki/listFacets'
import type { WikiFacet, WikiListFilter } from '@/lib/wiki/listFacets'
import { SortDirection } from '@/lib/wiki/listSort'
import type { WikiSortSpec } from '@/lib/wiki/listSort'

// How a category's list was being read: what it was filtered by (the title and every facet,
// design decision 4), how it was sorted, and how far down it was.
//
// **The filter was a `ref` in `WikiCategoryList`** (#79). While the router reused one component
// for every Wiki tab, a second tab on the same category opened with the first one's filter
// already typed; once each entry got its own instance, the filter was lost on every tab switch
// instead. Either way it never reached a torn-off tab. It is the entry's reading now, like every
// other list's.
//
// The landing and a page have nothing to keep here: a page's position is a scrolling region's,
// and `v-scroll-memory` keeps it on the entry beside this. The card-or-table choice is not
// here either — it is a per-category convenience, not a per-tab one, and it survives a
// restart through the window session instead (`lib/wiki/listViewMode.ts`).
export interface WikiReading {
  filter: WikiListFilter
  sort: WikiSortSpec
  offset: ScrollOffset | null
}

const DefaultSort: WikiSortSpec = {
  key: WikiSort.Name,
  direction: SortDirection.Asc,
}

const readSort = (value: unknown): WikiSortSpec => {
  if (typeof value !== 'object' || value === null) return DefaultSort
  const { key, direction } = value as { key?: unknown; direction?: unknown }
  return {
    key: typeof key === 'string' && key !== '' ? key : DefaultSort.key,
    direction:
      direction === SortDirection.Desc ? SortDirection.Desc : SortDirection.Asc,
  }
}

export const wikiView: TabViewSpec<WikiReading> = {
  empty: () => ({
    filter: emptyWikiListFilter(),
    sort: DefaultSort,
    offset: null,
  }),
  read: (value) => {
    if (typeof value !== 'object' || value === null) return null
    const { filter, sort, offset } = value as {
      filter?: unknown
      sort?: unknown
      offset?: unknown
    }
    return {
      filter:
        readFacetFilter<WikiFacet>(filter, wikiFacetOrder) ??
        emptyWikiListFilter(),
      sort: readSort(sort),
      offset: readScrollOffset(offset),
    }
  },
}
