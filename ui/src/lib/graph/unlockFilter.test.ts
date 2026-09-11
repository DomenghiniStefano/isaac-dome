import { describe, expect, it } from 'vitest'
import { graphAnswers } from '@/lib/ipc/fixtures/graph'
import type { GraphInfo, UnlockNode, UnlockTarget } from '@/lib/ipc/types'
import { NodeState, stateOrder } from './nodeState'
import {
  FacetId,
  OriginValue,
  UnlockKind,
  UnlockSort,
  activeFilterCount,
  emptyFilter,
  facetCounts,
  facetOptions,
  facetValues,
  matchesFilter,
  sortNodes,
} from './unlockFilter'

const computed = (
  availableNow: boolean,
  blockedBy = 0,
  fanOut = 0,
): GraphInfo => ({
  kind: 'computed',
  availableNow,
  blockedBy,
  fanOut,
  stepsMissing: blockedBy,
})
const partial: GraphInfo = {
  kind: 'partial',
  blockedBy: 1,
  fanOut: 0,
  unknown: 1,
}

const node = (slot: number, over: Partial<UnlockNode> = {}): UnlockNode => ({
  achievement: {
    kind: 'known',
    id: slot,
    text: `achievement ${slot}`,
    hint: null,
    iconUrl: null,
  },
  done: false,
  unlocks: [],
  origin: null,
  missing: [],
  graph: computed(true),
  ...over,
})

const passive = (name: string): UnlockTarget => ({
  kind: 'item',
  itemKind: 'passive',
  id: 1,
  name,
  iconUrl: null,
})
const character = (name: string): UnlockTarget => ({
  kind: 'character',
  id: 2,
  name,
})

const reference = graphAnswers({ withArt: false, withCatalog: true }).unlock
  .nodes

describe('facetValues', () => {
  it('reads what a node unlocks by kind, once each', () => {
    const n = node(1, { unlocks: [passive('A'), character('B'), passive('C')] })
    expect(facetValues(n, FacetId.Unlocks)).toEqual([
      UnlockKind.Passive,
      UnlockKind.Character,
    ])
  })

  it('says a node unlocks nothing rather than no value', () => {
    expect(facetValues(node(1), FacetId.Unlocks)).toEqual([UnlockKind.Nothing])
  })

  it('names a missing origin', () => {
    expect(facetValues(node(1), FacetId.Origin)).toEqual([OriginValue.None])
    expect(
      facetValues(node(1, { origin: 'repentance' }), FacetId.Origin),
    ).toEqual([OriginValue.Repentance])
  })

  it('reads the state and the required characters', () => {
    const n = node(1, {
      graph: computed(false, 2),
      missing: [
        { kind: 'character', id: 10, name: 'The Lost' },
        { kind: 'unknown', label: 'ending' },
        { kind: 'character', id: 0, name: 'Isaac' },
      ],
    })
    expect(facetValues(n, FacetId.State)).toEqual([NodeState.Blocked])
    expect(facetValues(n, FacetId.Character)).toEqual(['The Lost', 'Isaac'])
  })
})

describe('matchesFilter', () => {
  const lost = node(1, {
    achievement: {
      kind: 'known',
      id: 1,
      text: 'You unlocked "The Lost"',
      hint: 'die 4 times',
      iconUrl: null,
    },
    unlocks: [character('The Lost')],
    origin: 'rebirth',
  })

  it('lets everything through an empty filter', () => {
    expect(matchesFilter(lost, emptyFilter())).toBe(true)
  })

  it('takes any value within a facet, and every facet at once', () => {
    const picks = (unlocks: string[], origin: string[]) => ({
      ...emptyFilter(),
      picks: {
        ...emptyFilter().picks,
        [FacetId.Unlocks]: unlocks,
        [FacetId.Origin]: origin,
      },
    })
    expect(
      matchesFilter(
        lost,
        picks([UnlockKind.Passive, UnlockKind.Character], []),
      ),
    ).toBe(true)
    expect(
      matchesFilter(lost, picks([UnlockKind.Character], [OriginValue.Rebirth])),
    ).toBe(true)
    expect(
      matchesFilter(
        lost,
        picks([UnlockKind.Character], [OriginValue.Repentance]),
      ),
    ).toBe(false)
  })

  it('searches the text, the condition and what it unlocks, in any case', () => {
    const query = (q: string) => ({ ...emptyFilter(), query: q })
    expect(matchesFilter(lost, query('LOST'))).toBe(true)
    expect(matchesFilter(lost, query('4 times'))).toBe(true)
    expect(
      matchesFilter(
        node(2, { unlocks: [passive('Cube of Meat')] }),
        query('cube'),
      ),
    ).toBe(true)
    expect(matchesFilter(lost, query('onion'))).toBe(false)
    expect(matchesFilter(lost, query('   '))).toBe(true)
  })
})

describe('facetCounts', () => {
  const nodes = [
    node(1, { unlocks: [passive('A'), passive('B')], origin: 'rebirth' }),
    node(2, { unlocks: [character('C')], origin: 'rebirth' }),
    node(3, { unlocks: [passive('D')], origin: 'repentance' }),
  ]
  const filter = {
    ...emptyFilter(),
    picks: { ...emptyFilter().picks, [FacetId.Unlocks]: [UnlockKind.Passive] },
  }

  it('leaves its own facet out, so a count says what picking it would give', () => {
    const counts = facetCounts(nodes, filter, FacetId.Unlocks)
    expect(counts.get(UnlockKind.Passive)).toBe(2)
    expect(counts.get(UnlockKind.Character)).toBe(1)
  })

  it('applies every other facet', () => {
    const counts = facetCounts(nodes, filter, FacetId.Origin)
    expect(counts.get(OriginValue.Rebirth)).toBe(1)
    expect(counts.get(OriginValue.Repentance)).toBe(1)
  })

  // Counted on the committed unlock.json outside our code: 231 nodes unlock nothing the
  // catalog knows, 274 have no origin, 119 are unlockable now (one origin value each).
  it('counts the reference profile', () => {
    expect(
      facetCounts(reference, emptyFilter(), FacetId.Unlocks).get(
        UnlockKind.Nothing,
      ),
    ).toBe(231)
    expect(
      facetCounts(reference, emptyFilter(), FacetId.Origin).get(
        OriginValue.None,
      ),
    ).toBe(274)
    const now = {
      ...emptyFilter(),
      picks: { ...emptyFilter().picks, [FacetId.State]: [NodeState.Now] },
    }
    const byOrigin = [...facetCounts(reference, now, FacetId.Origin).values()]
    expect(byOrigin.reduce((sum, n) => sum + n, 0)).toBe(119)
  })
})

describe('facetOptions', () => {
  it('keeps the fixed orders', () => {
    expect(facetOptions(reference, FacetId.State)).toEqual(stateOrder)
    expect(facetOptions(reference, FacetId.Unlocks)).toEqual([
      UnlockKind.Passive,
      UnlockKind.Active,
      UnlockKind.Familiar,
      UnlockKind.Trinket,
      UnlockKind.Character,
      UnlockKind.Boss,
      UnlockKind.Challenge,
      UnlockKind.Nothing,
    ])
    expect(facetOptions(reference, FacetId.Origin)).toEqual([
      OriginValue.Rebirth,
      OriginValue.Afterbirth,
      OriginValue.AfterbirthPlus,
      OriginValue.Repentance,
      OriginValue.None,
    ])
  })

  it('lists the required characters found in the nodes, by name', () => {
    const characters = facetOptions(reference, FacetId.Character)
    expect(characters).toHaveLength(10)
    expect(characters).toEqual([...characters].sort())
  })
})

describe('sortNodes', () => {
  it('puts the most fan-out first, ties in slot order', () => {
    const nodes = [
      node(1, { graph: computed(true, 0, 1) }),
      node(2, { graph: computed(true, 0, 5) }),
      node(3, { graph: computed(true, 0, 1) }),
    ]
    expect(slots(sortNodes(nodes, UnlockSort.FanOut))).toEqual([2, 1, 3])
  })

  it('puts done nodes after the rest: what is done opens nothing more for you', () => {
    const nodes = [
      node(1, { done: true, graph: computed(true, 0, 10) }),
      node(2, { graph: computed(true, 0, 3) }),
      node(3, { graph: computed(false, 2, 1) }),
    ]
    expect(slots(sortNodes(nodes, UnlockSort.FanOut))).toEqual([2, 3, 1])
  })

  it('puts the fewest steps first, then partial, then done', () => {
    const nodes = [
      node(1, { graph: computed(false, 2) }),
      node(2, { done: true }),
      node(3, { graph: computed(true, 0) }),
      node(4, { graph: partial }),
      node(5, { graph: computed(false, 1) }),
    ]
    expect(slots(sortNodes(nodes, UnlockSort.Steps))).toEqual([3, 5, 1, 4, 2])
  })

  it('sorts by name, unknown achievements last', () => {
    const named = (slot: number, text: string) =>
      node(slot, {
        achievement: {
          kind: 'known',
          id: slot,
          text,
          hint: null,
          iconUrl: null,
        },
      })
    const nodes = [
      node(9, { achievement: { kind: 'unknown', slot: 9 } }),
      named(1, 'Zeta'),
      named(2, 'alpha'),
    ]
    expect(slots(sortNodes(nodes, UnlockSort.Name))).toEqual([2, 1, 9])
  })
})

describe('activeFilterCount', () => {
  it('counts the picked values, not the search', () => {
    const filter = {
      query: 'lost',
      picks: {
        ...emptyFilter().picks,
        [FacetId.State]: [NodeState.Now],
        [FacetId.Origin]: [OriginValue.Rebirth, OriginValue.None],
      },
    }
    expect(activeFilterCount(filter)).toBe(3)
  })
})

function slots(nodes: UnlockNode[]): number[] {
  return nodes.map((n) =>
    n.achievement.kind === 'known' ? n.achievement.id : n.achievement.slot,
  )
}
