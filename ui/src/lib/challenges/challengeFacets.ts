import { assertNever } from '@/lib/assertNever'
import { createFaceting } from '@/lib/facets/faceting'
import type { ChallengeRow } from '@/lib/ipc/types'

// The Challenges screen's half of a faceted list: which facets it has, how a row answers one,
// what the search reads, what each facet offers. Matching, the counts and the active count are
// the engine's — this is the fourth screen on it.

export const ChallengeFacet = {
  State: 'state',
  Character: 'character',
  Rewards: 'rewards',
  Blindfolded: 'blindfolded',
} as const
export type ChallengeFacet =
  (typeof ChallengeFacet)[keyof typeof ChallengeFacet]

const facetOrder: ChallengeFacet[] = [
  ChallengeFacet.State,
  ChallengeFacet.Character,
  ChallengeFacet.Rewards,
  ChallengeFacet.Blindfolded,
]

// Ours, not the wire's: "it unlocks something" and "it unlocks nothing" are two values a reader
// can pick, and the wire carries a list. A const object and not a union of strings — the repo's
// rule.
export const RewardsValue = { Some: 'some', None: 'none' } as const
export type RewardsValue = (typeof RewardsValue)[keyof typeof RewardsValue]

export const BlindfoldedValue = { Yes: 'yes', No: 'no' } as const
export type BlindfoldedValue =
  (typeof BlindfoldedValue)[keyof typeof BlindfoldedValue]

const rewardsOrder: RewardsValue[] = [RewardsValue.Some, RewardsValue.None]
const blindfoldedOrder: BlindfoldedValue[] = [
  BlindfoldedValue.Yes,
  BlindfoldedValue.No,
]

// What a row answers for a facet. **A row the wiki knows nothing about answers with nothing**:
// `blindfolded: null` is "there is no page", and offering it as "not blindfolded" would put a
// row under a value the app cannot support. Same for a challenge the page names no character
// for.
const facetValues = (row: ChallengeRow, facet: ChallengeFacet): string[] => {
  switch (facet) {
    case ChallengeFacet.State:
      return [row.state.kind]
    case ChallengeFacet.Character:
      return row.character === null || row.character.kind !== 'character'
        ? []
        : [String(row.character.id)]
    case ChallengeFacet.Rewards:
      return [row.rewards.length > 0 ? RewardsValue.Some : RewardsValue.None]
    case ChallengeFacet.Blindfolded:
      return row.blindfolded === null
        ? []
        : [row.blindfolded ? BlindfoldedValue.Yes : BlindfoldedValue.No]
    default:
      return assertNever(facet)
  }
}

// What the search reads: the name is how a player names a challenge, the number is how the
// game does — "Sfida 33" is what Unlock prints, so typing 33 has to find it.
const searchText = (row: ChallengeRow): string => `${row.name} ${row.number}`

// The values each facet offers, in the order the screen shows them. The state's order is the
// screen's own table; the others are read off the rows, because how many characters appear
// depends on the catalog and not on us.
const facetOptions = (
  rows: ChallengeRow[],
  facet: ChallengeFacet,
): string[] => {
  switch (facet) {
    case ChallengeFacet.State:
      return [...new Set(rows.map((r) => r.state.kind))]
    case ChallengeFacet.Character:
      return [...new Set(rows.flatMap((r) => facetValues(r, facet)))]
    case ChallengeFacet.Rewards:
      return rewardsOrder.filter((v) =>
        rows.some((r) => facetValues(r, facet).includes(v)),
      )
    case ChallengeFacet.Blindfolded:
      return blindfoldedOrder.filter((v) =>
        rows.some((r) => facetValues(r, facet).includes(v)),
      )
    default:
      return assertNever(facet)
  }
}

export const challengeFaceting = createFaceting<ChallengeRow, ChallengeFacet>({
  order: facetOrder,
  values: facetValues,
  text: searchText,
  options: facetOptions,
})
