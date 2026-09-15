import { readStringArray } from '@/lib/tabs/tabView'
import { readScrollOffset } from '@/lib/scale/scrollOffset'
import type { ScrollOffset } from '@/lib/scale/scrollOffset'
import type { TabViewSpec } from '@/lib/tabs/tabView'
import { RowGroup } from '@/lib/search/rows'

export interface SearchReading {
  picked: RowGroup[]
  offset: ScrollOffset | null
}

const groups: readonly string[] = Object.values(RowGroup)

// The query is not here: it is in the location already, because a search is a place you can link
// to. What the tab remembers is which groups you narrowed it to.
export const searchView: TabViewSpec<SearchReading> = {
  empty: () => ({ picked: [], offset: null }),
  read: (value) => {
    if (typeof value !== 'object' || value === null) return null
    const picked = readStringArray(
      (value as { picked?: unknown }).picked,
      groups,
    )
    if (picked === null) return null
    return {
      picked: picked as RowGroup[],
      offset: readScrollOffset((value as { offset?: unknown }).offset),
    }
  },
}
