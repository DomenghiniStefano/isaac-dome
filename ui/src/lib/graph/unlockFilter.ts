import { countBy, sortBy, sumBy, uniq } from 'lodash-es'
import { assertNever } from '@/lib/assertNever'
import type { UnlockNode, UnlockTarget } from '@/lib/ipc/types'
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

// What a node unlocks, by kind. "Nothing" is a value of its own: 231 nodes on the reference
// profile unlock nothing the catalog knows, and that is something to filter on.
export const UnlockKind = {
  Passive: 'passive',
  Active: 'active',
  Familiar: 'familiar',
  Trinket: 'trinket',
  Character: 'character',
  Boss: 'boss',
  Challenge: 'challenge',
  Nothing: 'nothing',
} as const
export type UnlockKind = (typeof UnlockKind)[keyof typeof UnlockKind]

const unlockKindOrder: UnlockKind[] = [
  UnlockKind.Passive,
  UnlockKind.Active,
  UnlockKind.Familiar,
  UnlockKind.Trinket,
  UnlockKind.Character,
  UnlockKind.Boss,
  UnlockKind.Challenge,
  UnlockKind.Nothing,
]

// The origin DLC as the catalog infers it, plus the nodes it can't say for.
export const OriginValue = {
  Rebirth: 'rebirth',
  Afterbirth: 'afterbirth',
  AfterbirthPlus: 'afterbirthPlus',
  Repentance: 'repentance',
  None: 'none',
} as const
export type OriginValue = (typeof OriginValue)[keyof typeof OriginValue]

const originOrder: OriginValue[] = [
  OriginValue.Rebirth,
  OriginValue.Afterbirth,
  OriginValue.AfterbirthPlus,
  OriginValue.Repentance,
  OriginValue.None,
]

export const UnlockSort = {
  FanOut: 'fanOut',
  Steps: 'steps',
  Name: 'name',
} as const
export type UnlockSort = (typeof UnlockSort)[keyof typeof UnlockSort]

export interface UnlockFilter {
  query: string
  picks: Record<FacetId, string[]>
}

export const emptyFilter = (): UnlockFilter => ({
  query: '',
  picks: {
    [FacetId.State]: [],
    [FacetId.Unlocks]: [],
    [FacetId.Origin]: [],
    [FacetId.Character]: [],
  },
})

export const nodeSlot = (node: UnlockNode): number =>
  node.achievement.kind === 'known'
    ? node.achievement.id
    : node.achievement.slot

export const targetKind = (target: UnlockTarget): UnlockKind => {
  switch (target.kind) {
    case 'item':
      return target.itemKind
    case 'character':
      return UnlockKind.Character
    case 'boss':
      return UnlockKind.Boss
    case 'challenge':
      return UnlockKind.Challenge
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
        : [UnlockKind.Nothing]
    case FacetId.Origin:
      return [node.origin ?? OriginValue.None]
    case FacetId.Character:
      return uniq(
        node.missing.flatMap((r) => (r.kind === 'character' ? [r.name] : [])),
      )
    default:
      return assertNever(facet)
  }
}

const searchText = (node: UnlockNode): string =>
  [
    node.achievement.kind === 'known' ? node.achievement.text : '',
    node.achievement.kind === 'known' ? (node.achievement.hint ?? '') : '',
    ...node.unlocks.map((t) => t.name),
  ]
    .join('\n')
    .toLowerCase()

const matchesQuery = (node: UnlockNode, query: string): boolean => {
  const wanted = query.trim().toLowerCase()
  return wanted === '' || searchText(node).includes(wanted)
}

const matchesFacet = (
  node: UnlockNode,
  facet: FacetId,
  picked: string[],
): boolean =>
  picked.length === 0 ||
  facetValues(node, facet).some((value) => picked.includes(value))

const matchesFacets = (
  node: UnlockNode,
  filter: UnlockFilter,
  facets: FacetId[],
): boolean =>
  matchesQuery(node, filter.query) &&
  facets.every((facet) => matchesFacet(node, facet, filter.picks[facet]))

// Any value within a facet, every facet at once, and the search.
export const matchesFilter = (
  node: UnlockNode,
  filter: UnlockFilter,
): boolean => matchesFacets(node, filter, facetOrder)

// A value's count leaves its own facet out: it says how many rows picking it would give.
export const facetCounts = (
  nodes: UnlockNode[],
  filter: UnlockFilter,
  facet: FacetId,
): Map<string, number> => {
  const others = facetOrder.filter((f) => f !== facet)
  const values = nodes
    .filter((node) => matchesFacets(node, filter, others))
    .flatMap((node) => facetValues(node, facet))
  return new Map(Object.entries(countBy(values)))
}

export const facetOptions = (nodes: UnlockNode[], facet: FacetId): string[] => {
  switch (facet) {
    case FacetId.State:
      return stateOrder
    case FacetId.Unlocks:
      return unlockKindOrder
    case FacetId.Origin:
      return originOrder
    case FacetId.Character:
      return sortBy(uniq(nodes.flatMap((n) => facetValues(n, facet))))
    default:
      return assertNever(facet)
  }
}

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
      return sortBy(nodes, [(n) => -n.graph.fanOut, nodeSlot])
    case UnlockSort.Steps:
      return sortBy(nodes, [
        (n) => stepsRank[nodeState(n)],
        (n) => n.graph.blockedBy,
        nodeSlot,
      ])
    case UnlockSort.Name:
      return sortBy(nodes, [
        (n) => (n.achievement.kind === 'known' ? 0 : 1),
        (n) =>
          n.achievement.kind === 'known'
            ? n.achievement.text.toLowerCase()
            : '',
        nodeSlot,
      ])
    default:
      return assertNever(sort)
  }
}

// How many values are picked across the facets; the search is shown on its own.
export const activeFilterCount = (filter: UnlockFilter): number =>
  sumBy(facetOrder, (facet) => filter.picks[facet].length)
