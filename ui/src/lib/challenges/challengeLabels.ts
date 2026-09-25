import type { Message, Translate } from '@/i18n/message'
import type { FilterBarLabels } from '@/lib/facets/labels'
import { assertNever } from '@/lib/assertNever'
import { StateTone, stateDots } from '@/lib/facets/stateTone'
import {
  BlindfoldedValue,
  ChallengeFacet,
  ChallengeState,
  RewardsValue,
  challengeFaceting,
} from '@/lib/challenges/challengeFacets'
import type { FacetSlot } from '@/lib/facets/facetOptions'
import type { FilterBarDescriptor } from '@/lib/facets/filterBar'
import type { ChallengeRow } from '@/lib/ipc/types'
import { oneOf } from '@/lib/oneOf'

// The four states a challenge can be in, in the order the row shows them: what is behind you,
// what you can play now, what is not offered yet, and what could not be read.
const challengeStateOrder: ChallengeState[] = [
  ChallengeState.Done,
  ChallengeState.Available,
  ChallengeState.Blocked,
  ChallengeState.Unknown,
]

// A state's plain name: the row's toggle, its chips, and the badge but for a blocked one, which
// says how many gates it waits for.
export const challengeStateText: Record<ChallengeState, Message> = {
  [ChallengeState.Done]: 'challenges.state.done',
  [ChallengeState.Available]: 'challenges.state.available',
  [ChallengeState.Blocked]: 'challenges.state.blocked',
  [ChallengeState.Unknown]: 'challenges.state.unknown',
}

// Unreadable wears the unknown hatch, as its badge does.
const challengeStateDot = stateDots<ChallengeState>({
  [ChallengeState.Done]: StateTone.Done,
  [ChallengeState.Available]: StateTone.Now,
  [ChallengeState.Blocked]: StateTone.Blocked,
  [ChallengeState.Unknown]: StateTone.Unknown,
})

const challengeFacetTitle: Record<ChallengeFacet, Message> = {
  [ChallengeFacet.State]: 'challenges.facet.state',
  [ChallengeFacet.Character]: 'challenges.facet.character',
  [ChallengeFacet.Rewards]: 'challenges.facet.rewards',
  [ChallengeFacet.Blindfolded]: 'challenges.facet.blindfolded',
}

// Spec 3.11 §7: the character is the filter a reader reaches for here; what it unlocks and
// whether it is blindfolded are behind the fold. The state is not one of these — it has its
// own row.
const challengeSlots: FacetSlot<ChallengeFacet>[] = [
  { facet: ChallengeFacet.Character, inView: true },
  { facet: ChallengeFacet.Rewards, inView: false },
  { facet: ChallengeFacet.Blindfolded, inView: false },
]

const barLabels: FilterBarLabels = {
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
    case ChallengeFacet.State: {
      const state = oneOf(ChallengeState, value)
      return state ? t(challengeStateText[state]) : value
    }
    case ChallengeFacet.Character:
      return characterNames.get(value) ?? value
    case ChallengeFacet.Rewards: {
      const rewards = oneOf(RewardsValue, value)
      if (rewards === RewardsValue.Some) return t('challenges.rewardsSome')
      if (rewards === RewardsValue.None) return t('challenges.rewardsNone')
      return value
    }
    case ChallengeFacet.Blindfolded: {
      const blindfolded = oneOf(BlindfoldedValue, value)
      if (blindfolded === BlindfoldedValue.Yes)
        return t('challenges.blindfoldedYes')
      if (blindfolded === BlindfoldedValue.No)
        return t('challenges.blindfoldedNo')
      return value
    }
    default:
      return assertNever(facet)
  }
}

// The Challenges' filter bar: everything about it that holds while the screen is open. No sort:
// the rows keep the game's order.
export const challengeBar: FilterBarDescriptor<
  ChallengeRow,
  ChallengeFacet,
  never
> = {
  faceting: challengeFaceting,
  facets: challengeSlots,
  state: {
    facet: ChallengeFacet.State,
    order: challengeStateOrder,
    dot: challengeStateDot,
    text: challengeStateText,
  },
  title: challengeFacetTitle,
  labels: barLabels,
}
