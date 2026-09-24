import { readScrollOffset } from '@/lib/scale/scrollOffset'
import type { ScrollOffset } from '@/lib/scale/scrollOffset'
import type { TabViewSpec } from '@/lib/tabs/tabView'

// How a category's list was being read: what it was filtered by, and how far down it was.
//
// **The filter was a `ref` in `WikiCategoryList`** (#79). While the router reused one component
// for every Wiki tab, a second tab on the same category opened with the first one's filter
// already typed; once each entry got its own instance, the filter was lost on every tab switch
// instead. Either way it never reached a torn-off tab. It is the entry's reading now, like every
// other list's.
//
// The landing and a page have nothing to keep here: a page's position is a scrolling region's,
// and `v-scroll-memory` keeps it on the entry beside this.
export interface WikiReading {
  query: string
  offset: ScrollOffset | null
}

export const wikiView: TabViewSpec<WikiReading> = {
  empty: () => ({ query: '', offset: null }),
  read: (value) => {
    if (typeof value !== 'object' || value === null) return null
    const { query, offset } = value as { query?: unknown; offset?: unknown }
    return {
      query: typeof query === 'string' ? query : '',
      offset: readScrollOffset(offset),
    }
  },
}
