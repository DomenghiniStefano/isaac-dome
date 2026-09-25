import type { FilterBarDescriptor } from '@/lib/facets/filterBar'
import type { RunView } from '@/lib/ipc/types'
import { RunFacet, outcomeOrder, runFaceting } from '@/lib/runs/runFacets'
import {
  barLabels,
  facetTitle,
  outcomeDot,
  outcomeTextByKind,
  runSlots,
} from '@/lib/runs/runLabels'

// The Run diary's filter bar. No sort: the order is the archive's (`runOrder.ts`).
export const runsBar: FilterBarDescriptor<RunView, RunFacet, never> = {
  faceting: runFaceting,
  facets: runSlots,
  state: {
    facet: RunFacet.Outcome,
    order: outcomeOrder,
    dot: outcomeDot,
    text: outcomeTextByKind,
  },
  title: facetTitle,
  labels: barLabels,
}
