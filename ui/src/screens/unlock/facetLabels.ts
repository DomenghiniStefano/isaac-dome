import { unlockKindText } from '@/components/graph/unlockKindText'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import { assertNever } from '@/lib/assertNever'
import { oneOf } from '@/lib/oneOf'
import { characterLabel } from '@/lib/graph/characterName'
import type { CharacterForm } from '@/lib/graph/characterName'
import { NodeState } from '@/lib/graph/nodeState'
import { originLabel } from '@/components/facets/labels'
import type {
  DrawerLabels,
  ToolbarLabels,
  Translate,
} from '@/components/facets/labels'
import { FacetId, UnlockSort } from '@/lib/graph/unlockFacets'
import { TargetKind } from '@/lib/ipc/values'

export const facetTitle: Record<FacetId, MessageKey<MessageSchema>> = {
  [FacetId.State]: 'unlock.facet.state',
  [FacetId.Unlocks]: 'unlock.facet.unlocks',
  [FacetId.Origin]: 'unlock.facet.origin',
  [FacetId.Character]: 'unlock.facet.character',
}

// A state's plain name, for the toggle and the chips; the badge says "blocked by N" itself.
export const stateText: Record<NodeState, MessageKey<MessageSchema>> = {
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

// The facets the drawer holds: the state has its own control above the table.
export const drawerFacets: FacetId[] = [
  FacetId.Unlocks,
  FacetId.Origin,
  FacetId.Character,
]

export const sortOrder: UnlockSort[] = [
  UnlockSort.FanOut,
  UnlockSort.Steps,
  UnlockSort.Name,
]

export const sortText: Record<UnlockSort, MessageKey<MessageSchema>> = {
  [UnlockSort.FanOut]: 'unlock.sort.fanOut',
  [UnlockSort.Steps]: 'unlock.sort.steps',
  [UnlockSort.Name]: 'unlock.sort.name',
}

export const toolbarLabels: ToolbarLabels = {
  rows: 'unlock.rows',
  search: 'unlock.search',
  sortBy: 'unlock.sortBy',
  activeFilters: 'unlock.activeFilters',
}

export const drawerLabels: DrawerLabels = {
  facets: 'unlock.facets',
  activeFilters: 'unlock.activeFilters',
  noFilters: 'unlock.noFilters',
  reset: 'unlock.reset',
}
