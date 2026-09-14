import { unlockKindText } from '@/components/graph/unlockKindText'
import { dlcNames } from '@/components/wiki/dlcNames'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import { assertNever } from '@/lib/assertNever'
import { characterLabel } from '@/lib/graph/characterName'
import type { CharacterForm } from '@/lib/graph/characterName'
import { NodeState } from '@/lib/graph/nodeState'
import { FacetId } from '@/lib/graph/unlockFacets'
import { OriginValue, TargetKind } from '@/lib/ipc/values'
import { Dlc } from '@/lib/ipc/types'

type Translate = (
  key: MessageKey<MessageSchema>,
  params?: Record<string, unknown>,
) => string

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

// The origin DLC's names are game data, the same as the wiki's editions; only "not stated"
// is ours to say.
const originName: Record<OriginValue, string | null> = {
  [OriginValue.Rebirth]: dlcNames[Dlc.Rebirth],
  [OriginValue.Afterbirth]: dlcNames[Dlc.Afterbirth],
  [OriginValue.AfterbirthPlus]: dlcNames[Dlc.AfterbirthPlus],
  [OriginValue.Repentance]: dlcNames[Dlc.Repentance],
  [OriginValue.None]: null,
}

const find = <T extends string>(values: Record<string, T>, value: string) =>
  Object.values(values).find((v) => v === value)

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
      const state = find(NodeState, value)
      return state ? t(stateText[state]) : value
    }
    case FacetId.Unlocks: {
      const kind = find(TargetKind, value)
      return kind ? t(unlockKindText[kind]) : value
    }
    case FacetId.Origin: {
      const origin = find(OriginValue, value)
      if (!origin) return value
      return originName[origin] ?? t('graph.originNone')
    }
    case FacetId.Character: {
      const form = characters?.get(value)
      return form ? characterLabel(t, form) : value
    }
    default:
      return assertNever(facet)
  }
}
