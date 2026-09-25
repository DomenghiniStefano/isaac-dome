import { sortBy, uniq } from 'lodash-es'
import { assertNever } from '@/lib/assertNever'
import { createFaceting } from '@/lib/facets/faceting'
import type { FacetFilter } from '@/lib/facets/faceting'
import type { UnlockNode, UnlockTarget } from '@/lib/ipc/types'
import { OriginValue, TargetKind, originOrder } from '@/lib/ipc/values'
import { knownAchievement, knownText } from './achievementNode'
import { characterForms, characterValue } from './characterName'
import { NodeState, nodeState, stateOrder } from './nodeState'

// Unlock's facets: only the ones the contract can answer (DESIGN-BRIEF.md §7.5). Mode, effort,
// required ending, quality, pool and rarity have no field, and a facet of guesses would lie.
export const FacetId = {
  State: 'state',
  Unlocks: 'unlocks',
  Origin: 'origin',
  Character: 'character',
} as const
export type FacetId = (typeof FacetId)[keyof typeof FacetId]

export const facetOrder: FacetId[] = [
  FacetId.State,
  FacetId.Unlocks,
  FacetId.Origin,
  FacetId.Character,
]

const targetKindOrder: TargetKind[] = [
  TargetKind.Passive,
  TargetKind.Active,
  TargetKind.Familiar,
  TargetKind.Trinket,
  TargetKind.Character,
  TargetKind.Boss,
  TargetKind.Challenge,
  TargetKind.Nothing,
]

export const UnlockSort = {
  FanOut: 'fanOut',
  Steps: 'steps',
  Name: 'name',
} as const
export type UnlockSort = (typeof UnlockSort)[keyof typeof UnlockSort]

export type UnlockFilter = FacetFilter<FacetId>

export const nodeSlot = (node: UnlockNode): number =>
  node.achievement.kind === 'known'
    ? node.achievement.id
    : node.achievement.slot

export const targetKind = (target: UnlockTarget): TargetKind => {
  switch (target.kind) {
    case 'item':
      return target.itemKind
    case 'character':
      return TargetKind.Character
    case 'boss':
      return TargetKind.Boss
    case 'challenge':
      return TargetKind.Challenge
    default:
      return assertNever(target)
  }
}

// A node's values for one facet, each once. A node matches a facet when any of its values is
// picked: "what it unlocks" holds several kinds for 31 nodes of the reference profile.
export const facetValues = (node: UnlockNode, facet: FacetId): string[] => {
  switch (facet) {
    case FacetId.State:
      return [nodeState(node)]
    case FacetId.Unlocks:
      return node.unlocks.length > 0
        ? uniq(node.unlocks.map(targetKind))
        : [TargetKind.Nothing]
    case FacetId.Origin:
      return [node.origin ?? OriginValue.None]
    case FacetId.Character:
      // The id, never the name: the base and Tainted forms share the name, so a facet on
      // names would fold two characters into one value (`docs/BACKLOG.md` B28).
      return uniq(
        node.missing.flatMap((r) =>
          r.kind === 'character' ? [characterValue(r)] : [],
        ),
      )
    default:
      return assertNever(facet)
  }
}

// What the search reads: an achievement's text, the condition the wiki answers where the game
// file is silent, and the names of what it unlocks. The engine lowercases it.
const searchText = (node: UnlockNode): string =>
  [
    knownText(node) ?? '',
    knownAchievement(node)?.condition ?? '',
    ...node.unlocks.map((t) => t.name),
  ].join('\n')

const facetOptions = (nodes: UnlockNode[], facet: FacetId): string[] => {
  switch (facet) {
    case FacetId.State:
      return stateOrder
    case FacetId.Unlocks:
      return targetKindOrder
    case FacetId.Origin:
      return originOrder
    case FacetId.Character: {
      // Sorted by the name the player reads, then by form, so the two Losts sit together;
      // the values themselves stay the ids.
      const forms = characterForms(nodes)
      return sortBy(uniq(nodes.flatMap((n) => facetValues(n, facet))), [
        (value) => forms.get(value)?.name ?? value,
        (value) => (forms.get(value)?.tainted ? 1 : 0),
      ])
    }
    default:
      return assertNever(facet)
  }
}

// Unlock's half of a faceted list: which facets it has, how a node answers one, what the search
// reads, what each facet offers. Matching, the counts and the active count are the engine's.
export const unlockFaceting = createFaceting<UnlockNode, FacetId>({
  order: facetOrder,
  values: facetValues,
  text: searchText,
  options: facetOptions,
})

// Steps: what can be done now and what is closest first, then what the graph can't vouch for,
// then what is already done.
const stepsRank: Record<NodeState, number> = {
  [NodeState.Now]: 0,
  [NodeState.Blocked]: 0,
  [NodeState.Partial]: 1,
  [NodeState.Done]: 2,
}

// Every order ends on the slot, so equal rows never swap between two renders.
export const sortNodes = (
  nodes: UnlockNode[],
  sort: UnlockSort,
): UnlockNode[] => {
  switch (sort) {
    case UnlockSort.FanOut:
      // What is done opens nothing more for the player, whatever its fan-out: it goes last.
      return sortBy(nodes, [
        (n) => (n.done ? 1 : 0),
        (n) => -n.graph.fanOut,
        nodeSlot,
      ])
    case UnlockSort.Steps:
      return sortBy(nodes, [
        (n) => stepsRank[nodeState(n)],
        (n) => n.graph.blockedBy,
        nodeSlot,
      ])
    case UnlockSort.Name:
      return sortBy(nodes, [
        (n) => (knownAchievement(n) ? 0 : 1),
        (n) => knownText(n)?.toLowerCase() ?? '',
        nodeSlot,
      ])
    default:
      return assertNever(sort)
  }
}
