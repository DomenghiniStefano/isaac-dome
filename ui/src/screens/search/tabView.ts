import { readStringArray } from '@/lib/tabs/tabView'
import type { TabViewSpec } from '@/lib/tabs/tabView'
import { RowGroup } from '@/lib/search/rows'

export interface SearchReading {
  picked: RowGroup[]
}

const groups: readonly string[] = Object.values(RowGroup)

// The query is not here: it is in the location already, because a search is a place you can link
// to. What the tab remembers is which groups you narrowed it to.
export const searchView: TabViewSpec<SearchReading> = {
  empty: () => ({ picked: [] }),
  read: (value) => {
    if (typeof value !== 'object' || value === null) return null
    const picked = readStringArray(
      (value as { picked?: unknown }).picked,
      groups,
    )
    return picked === null ? null : { picked: picked as RowGroup[] }
  },
}
