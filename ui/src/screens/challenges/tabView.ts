import { readFacetFilter } from '@/lib/tabs/tabView'
import type { TabViewSpec } from '@/lib/tabs/tabView'
import {
  ChallengeFacet,
  challengeFaceting,
} from '@/lib/challenges/challengeFacets'
import type { FacetFilter } from '@/lib/facets/faceting'

export interface ChallengesReading {
  filter: FacetFilter<ChallengeFacet>
}

// No scroll offset and no sort: forty-five rows fit without a virtual list, and the order is
// the game's own numbering — there is nothing to choose between.
export const challengesView: TabViewSpec<ChallengesReading> = {
  empty: () => ({ filter: challengeFaceting.empty() }),
  read: (value) => {
    if (typeof value !== 'object' || value === null) return null
    const { filter } = value as { filter?: unknown }
    const read = readFacetFilter<ChallengeFacet>(
      filter,
      Object.values(ChallengeFacet),
    )
    if (read === null) return null
    return { filter: read }
  },
}
