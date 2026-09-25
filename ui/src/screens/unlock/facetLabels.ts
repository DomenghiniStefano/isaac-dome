import type { Message, Translate } from '@/i18n/message'
import { unlockKindText } from '@/components/graph/unlockKindText'
import { assertNever } from '@/lib/assertNever'
import { oneOf } from '@/lib/oneOf'
import { characterLabel } from '@/lib/graph/characterName'
import type { CharacterForm } from '@/lib/graph/characterName'
import { NodeState, stateOrder } from '@/lib/graph/nodeState'
import { originLabel } from '@/lib/facets/labels'
import type { FilterBarLabels } from '@/lib/facets/labels'
import type { FacetSlot } from '@/lib/facets/facetOptions'
import type { FilterBarDescriptor } from '@/lib/facets/filterBar'
import { FacetId, UnlockSort, unlockFaceting } from '@/lib/graph/unlockFacets'
import type { UnlockNode } from '@/lib/ipc/types'
import { TargetKind } from '@/lib/ipc/values'

export const facetTitle: Record<FacetId, Message> = {
  [FacetId.State]: 'unlock.facet.state',
  [FacetId.Unlocks]: 'unlock.facet.unlocks',
  [FacetId.Origin]: 'unlock.facet.origin',
  [FacetId.Character]: 'unlock.facet.character',
}

// A state's plain name, for the toggle and the chips; the badge says "blocked by N" itself.
export const stateText: Record<NodeState, Message> = {
  [NodeState.Done]: 'graph.stateName.done',
  [NodeState.Now]: 'graph.stateName.now',
  [NodeState.Blocked]: 'graph.stateName.blocked',
  [NodeState.Partial]: 'graph.stateName.partial',
}

// A facet value in words. Values come back from the filter as strings; one that isn't in its
// set is shown as it came rather than dropped.
export const facetValueLabel = (
  t: Translate,
  facet: FacetId,
  value: string,
  // The characters the nodes are missing, by the facet's value: a character facet stores
  // ids, because the base and Tainted forms share the name (`docs/BACKLOG.md` B28).
  characters?: Map<string, CharacterForm>,
): string => {
  switch (facet) {
    case FacetId.State: {
      const state = oneOf(NodeState, value)
      return state ? t(stateText[state]) : value
    }
    case FacetId.Unlocks: {
      const kind = oneOf(TargetKind, value)
      return kind ? t(unlockKindText[kind]) : value
    }
    case FacetId.Origin:
      return originLabel(t, value)
    case FacetId.Character: {
      const form = characters?.get(value)
      return form ? characterLabel(t, form) : value
    }
    default:
      return assertNever(facet)
  }
}

// A state is never colour alone: the square carries its colour, the name says it. Partial's
// square is the blocked colour with a dashed edge, like its badge.
export const stateDot: Record<NodeState, string> = {
  [NodeState.Done]: 'bg-state-done',
  [NodeState.Now]: 'bg-state-now',
  [NodeState.Blocked]: 'bg-state-blocked',
  [NodeState.Partial]: 'border border-dashed border-state-blocked',
}

// Spec 3.10 §3: what a node unlocks is the filter a reader reaches for here; the origin and
// the character are behind the fold. The state is not one of these — it has its own row.
export const unlockSlots: FacetSlot<FacetId>[] = [
  { facet: FacetId.Unlocks, inView: true },
  { facet: FacetId.Origin, inView: false },
  { facet: FacetId.Character, inView: false },
]

export const sortOrder: UnlockSort[] = [
  UnlockSort.FanOut,
  UnlockSort.Steps,
  UnlockSort.Name,
]

export const sortText: Record<UnlockSort, Message> = {
  [UnlockSort.FanOut]: 'unlock.sort.fanOut',
  [UnlockSort.Steps]: 'unlock.sort.steps',
  [UnlockSort.Name]: 'unlock.sort.name',
}

export const barLabels: FilterBarLabels = {
  rows: 'unlock.rows',
  search: 'unlock.search',
  sortBy: 'unlock.sortBy',
}

// Unlock's filter bar: everything about it that holds for as long as the screen is open.
export const unlockBar: FilterBarDescriptor<UnlockNode, FacetId, UnlockSort> = {
  faceting: unlockFaceting,
  facets: unlockSlots,
  state: {
    facet: FacetId.State,
    order: stateOrder,
    dot: stateDot,
    text: stateText,
  },
  title: facetTitle,
  labels: barLabels,
  sorts: { order: sortOrder, text: sortText },
}
