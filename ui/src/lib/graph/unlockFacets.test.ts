import { describe, expect, it } from 'vitest'
import { graphAnswers } from '@/lib/ipc/fixtures/graph'
import type { GraphInfo, UnlockNode, UnlockTarget } from '@/lib/ipc/types'
import { OriginValue, TargetKind } from '@/lib/ipc/values'
import { characterForms } from './characterName'
import { NodeState, stateOrder } from './nodeState'
import {
  FacetId,
  UnlockSort,
  facetValues,
  sortNodes,
  unlockFaceting,
} from './unlockFacets'

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
    condition: null,
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
  page: null,
})
const character = (name: string): UnlockTarget => ({
  kind: 'character',
  id: 2,
  name,
  tainted: false,
  page: null,
})

const reference = graphAnswers({ withCatalog: true }).unlock.nodes

describe('facetValues', () => {
  it('reads what a node unlocks by kind, once each', () => {
    const n = node(1, { unlocks: [passive('A'), character('B'), passive('C')] })
    expect(facetValues(n, FacetId.Unlocks)).toEqual([
      TargetKind.Passive,
      TargetKind.Character,
    ])
  })

  it('says a node unlocks nothing rather than no value', () => {
    expect(facetValues(node(1), FacetId.Unlocks)).toEqual([TargetKind.Nothing])
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
        {
          kind: 'character',
          id: 10,
          name: 'The Lost',
          tainted: false,
          page: null,
        },
        { kind: 'unknown', label: 'ending' },
        { kind: 'character', id: 0, name: 'Isaac', tainted: false, page: null },
      ],
    })
    expect(facetValues(n, FacetId.State)).toEqual([NodeState.Blocked])
    // The id, never the name: the two forms of a character share the name (B28).
    expect(facetValues(n, FacetId.Character)).toEqual(['10', '0'])
  })
})

// Matching itself — a value within a facet, every facet at once, a blank query — belongs to
// `lib/facets/faceting.test.ts` and is tested there on a row neither screen owns. What is
// Unlock's here is *what the search reads*, which is the one part of the pair that was never
// shared: three fields joined, against the Collection's single name.
describe('the search', () => {
  const lost = node(1, {
    achievement: {
      kind: 'known',
      id: 1,
      text: 'You unlocked "The Lost"',
      condition: 'die 4 times',
      iconUrl: null,
    },
    unlocks: [character('The Lost')],
    origin: 'rebirth',
  })

  it('searches the text, the condition and what it unlocks, in any case', () => {
    const query = (q: string) => ({ ...unlockFaceting.empty(), query: q })
    expect(unlockFaceting.matches(lost, query('LOST'))).toBe(true)
    expect(unlockFaceting.matches(lost, query('4 times'))).toBe(true)
    expect(
      unlockFaceting.matches(
        node(2, { unlocks: [passive('Cube of Meat')] }),
        query('cube'),
      ),
    ).toBe(true)
    expect(unlockFaceting.matches(lost, query('onion'))).toBe(false)
    expect(unlockFaceting.matches(lost, query('   '))).toBe(true)
  })
})

// How a count is taken is the engine's; that it lands on the right numbers for *these* nodes
// is Unlock's, and only real data can say so.
describe('the counts on the reference profile', () => {
  // Counted on the committed unlock.json outside our code: 231 nodes unlock nothing the
  // catalog knows, 274 have no origin, 119 are unlockable now (one origin value each).
  it('counts the reference profile', () => {
    expect(
      unlockFaceting
        .counts(reference, unlockFaceting.empty(), FacetId.Unlocks)
        .get(TargetKind.Nothing),
    ).toBe(231)
    expect(
      unlockFaceting
        .counts(reference, unlockFaceting.empty(), FacetId.Origin)
        .get(OriginValue.None),
    ).toBe(274)
    const now = {
      ...unlockFaceting.empty(),
      picks: {
        ...unlockFaceting.empty().picks,
        [FacetId.State]: [NodeState.Now],
      },
    }
    const byOrigin = [
      ...unlockFaceting.counts(reference, now, FacetId.Origin).values(),
    ]
    expect(byOrigin.reduce((sum, n) => sum + n, 0)).toBe(119)
  })
})

describe('the options a facet offers', () => {
  it('keeps the fixed orders', () => {
    expect(unlockFaceting.options(reference, FacetId.State)).toEqual(stateOrder)
    expect(unlockFaceting.options(reference, FacetId.Unlocks)).toEqual([
      TargetKind.Passive,
      TargetKind.Active,
      TargetKind.Familiar,
      TargetKind.Trinket,
      TargetKind.Character,
      TargetKind.Boss,
      TargetKind.Challenge,
      TargetKind.Nothing,
    ])
    expect(unlockFaceting.options(reference, FacetId.Origin)).toEqual([
      OriginValue.Rebirth,
      OriginValue.Afterbirth,
      OriginValue.AfterbirthPlus,
      OriginValue.Repentance,
      OriginValue.None,
    ])
  })

  it('lists the required characters found in the nodes, ordered by their name', () => {
    const characters = unlockFaceting.options(reference, FacetId.Character)
    expect(characters).toHaveLength(10)
    // The values are ids (B28); what they are sorted by is the name the player reads.
    const forms = characterForms(reference)
    const names = characters.map((value) => forms.get(value)?.name ?? value)
    expect(names).toEqual([...names].sort())
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
          condition: null,
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

function slots(nodes: UnlockNode[]): number[] {
  return nodes.map((n) =>
    n.achievement.kind === 'known' ? n.achievement.id : n.achievement.slot,
  )
}

describe('the character facet keeps the two forms apart', () => {
  const lost = (id: number, tainted: boolean) =>
    node(id, {
      graph: computed(false, 1),
      missing: [
        { kind: 'character', id, name: 'The Lost', tainted, page: null },
      ],
    })

  it('offers one option per character, not one per name', () => {
    const nodes = [lost(10, false), lost(31, true)]
    expect(unlockFaceting.options(nodes, FacetId.Character)).toEqual([
      '10',
      '31',
    ])
  })

  it('a pick on one form leaves the other out', () => {
    const filter = {
      ...unlockFaceting.empty(),
      picks: { ...unlockFaceting.empty().picks, [FacetId.Character]: ['31'] },
    }
    expect(unlockFaceting.matches(lost(31, true), filter)).toBe(true)
    expect(unlockFaceting.matches(lost(10, false), filter)).toBe(false)
  })
})
