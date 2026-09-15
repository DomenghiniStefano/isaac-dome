import { readFacetFilter } from '@/lib/tabs/tabView'
import type { TabViewSpec } from '@/lib/tabs/tabView'
import { RunFacet } from '@/lib/runs/runFacets'
import { emptyFilter } from '@/lib/facets/faceting'
import type { FacetFilter } from '@/lib/facets/faceting'

export interface RunsReading {
  filter: FacetFilter<RunFacet>
  selected: string | null
}

const order = Object.values(RunFacet)

// The selection travels as a key and never as the row: a `RunView` is the archive's answer of
// the moment — `runs` is a cache the fold rebuilds when the rules version changes — and a stored
// one would come back describing a run that has since been re-derived.
export const runsView: TabViewSpec<RunsReading> = {
  empty: () => ({ filter: emptyFilter<RunFacet>(order), selected: null }),
  read: (value) => {
    if (typeof value !== 'object' || value === null) return null
    const { filter, selected } = value as {
      filter?: unknown
      selected?: unknown
    }
    const read = readFacetFilter<RunFacet>(filter, order)
    if (read === null) return null
    return {
      filter: read,
      selected: typeof selected === 'string' ? selected : null,
    }
  },
}
