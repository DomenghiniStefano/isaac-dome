import { readFacetFilter, readString } from '@/lib/tabs/tabView'
import type { TabViewSpec } from '@/lib/tabs/tabView'
import { RunItemView, runItemViews } from '@/lib/runs/itemViews'
import { RunFacet } from '@/lib/runs/runFacets'
import { emptyFilter } from '@/lib/facets/faceting'
import type { FacetFilter } from '@/lib/facets/faceting'

// What a Runs tab remembers: how the list was filtered, and how a run's items were being read on
// its page. No selection — a run is a page of its own, named in the location — and no offset: the
// screen scrolls as a page, and the page box keeps its own position.
export interface RunsReading {
  filter: FacetFilter<RunFacet>
  itemView: RunItemView
}

const order = Object.values(RunFacet)

export const runsView: TabViewSpec<RunsReading> = {
  empty: () => ({
    filter: emptyFilter<RunFacet>(order),
    itemView: RunItemView.AsLogged,
  }),
  read: (value) => {
    if (typeof value !== 'object' || value === null) return null
    const { filter, itemView } = value as {
      filter?: unknown
      itemView?: unknown
    }
    const read = readFacetFilter<RunFacet>(filter, order)
    if (read === null) return null
    return {
      filter: read,
      itemView:
        (readString(itemView, runItemViews) as RunItemView | null) ??
        RunItemView.AsLogged,
    }
  },
}
