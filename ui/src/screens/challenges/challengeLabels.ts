import type { Message, Translate } from '@/i18n/message'
import type { FilterBarLabels } from '@/lib/facets/labels'
import { assertNever } from '@/lib/assertNever'
import {
  BlindfoldedValue,
  ChallengeFacet,
  RewardsValue,
} from '@/lib/challenges/challengeFacets'
import type { FacetSlot } from '@/lib/facets/facetOptions'
import { oneOf } from '@/lib/oneOf'

// The four states a challenge can be in, in the order the row shows them: what is behind you,
// what you can play now, what is not offered yet, and what could not be read.
export const challengeStateOrder = ['done', 'available', 'blocked', 'unknown']

export const challengeStateText: Record<string, Message> = {
  done: 'challenges.state.done',
  available: 'challenges.state.available',
  blocked: 'challenges.state.blocked',
  unknown: 'challenges.state.unknown',
}

// A state is never colour alone: the square carries the colour, the name says it. Unreadable
// wears the unknown hatch, as its badge does.
export const challengeStateDot: Record<string, string> = {
  done: 'bg-state-done',
  available: 'bg-state-now',
  blocked: 'bg-state-blocked',
  unknown: 'hatch-unknown border border-dashed border-state-unknown',
}

export const challengeFacetTitle: Record<ChallengeFacet, Message> = {
  [ChallengeFacet.State]: 'challenges.facet.state',
  [ChallengeFacet.Character]: 'challenges.facet.character',
  [ChallengeFacet.Rewards]: 'challenges.facet.rewards',
  [ChallengeFacet.Blindfolded]: 'challenges.facet.blindfolded',
}

// Spec 3.11 §7: the character is the filter a reader reaches for here; what it unlocks and
// whether it is blindfolded are behind the fold. The state is not one of these — it has its
// own row.
export const challengeSlots: FacetSlot<ChallengeFacet>[] = [
  { facet: ChallengeFacet.Character, inView: true },
  { facet: ChallengeFacet.Rewards, inView: false },
  { facet: ChallengeFacet.Blindfolded, inView: false },
]

export const barLabels: FilterBarLabels = {
  rows: 'challenges.rows',
  search: 'challenges.search',
}

/**
 * A facet value in words. The character facet stores the wiki's **id**, so the screen is the
 * only place that can name it — it holds the rows the names come from.
 */
export const challengeFacetValueLabel = (
  t: Translate,
  facet: ChallengeFacet,
  value: string,
  characterNames: Map<string, string>,
): string => {
  switch (facet) {
    case ChallengeFacet.State:
      return challengeStateText[value] ? t(challengeStateText[value]) : value
    case ChallengeFacet.Character:
      return characterNames.get(value) ?? value
    case ChallengeFacet.Rewards:
      return oneOf(RewardsValue, value) === RewardsValue.Some
        ? t('challenges.rewardsSome')
        : t('challenges.rewardsNone')
    case ChallengeFacet.Blindfolded:
      return oneOf(BlindfoldedValue, value) === BlindfoldedValue.Yes
        ? t('challenges.blindfoldedYes')
        : t('challenges.blindfoldedNo')
    default:
      return assertNever(facet)
  }
}
