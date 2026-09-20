import { describe, expect, it } from 'vitest'
import { graphAnswers } from '@/lib/ipc/fixtures/graph'
import type { GraphInfo, UnlockNode } from '@/lib/ipc/types'
import { stateRowCounts } from '@/lib/facets/facetOptions'
import { FacetId, unlockFaceting } from '@/lib/graph/unlockFacets'
import {
  NodeState,
  RequirementKind,
  missingGroups,
  nodeState,
  stateOrder,
} from './nodeState'

const computed = (availableNow: boolean): GraphInfo => ({
  kind: 'computed',
  availableNow,
  blockedBy: availableNow ? 0 : 2,
  fanOut: 3,
  stepsMissing: availableNow ? 0 : 2,
})
const partial: GraphInfo = {
  kind: 'partial',
  blockedBy: 1,
  fanOut: 0,
  unknown: 1,
}

const node = (over: Partial<UnlockNode>): UnlockNode => ({
  achievement: {
    kind: 'known',
    id: 1,
    text: 'a',
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

// DESIGN-BRIEF.md §7.1 and the spec's Decision 1.
describe('nodeState', () => {
  it('calls a node done when the save says so, whatever the graph', () => {
    expect(nodeState(node({ done: true, graph: computed(false) }))).toBe(
      NodeState.Done,
    )
    expect(nodeState(node({ done: true, graph: partial }))).toBe(NodeState.Done)
  })

  it('calls a computed node unlockable now or blocked', () => {
    expect(nodeState(node({ graph: computed(true) }))).toBe(NodeState.Now)
    expect(nodeState(node({ graph: computed(false) }))).toBe(NodeState.Blocked)
  })

  it('never calls a partial node unlockable', () => {
    expect(nodeState(node({ graph: partial }))).toBe(NodeState.Partial)
  })

  it('reads an unknown achievement by the same rules', () => {
    expect(
      nodeState(
        node({ achievement: { kind: 'unknown', slot: 640 }, graph: partial }),
      ),
    ).toBe(NodeState.Partial)
  })
})

describe('the state row on the reference profile', () => {
  // The four numbers of the committed payload, through the path the bar actually walks since
  // 3.10: the screens' own tally is gone, and these numbers are the fixture's shape rather than
  // that function's, so they are pinned where they are now read.
  it('counts the four states of the committed payload', () => {
    const { unlock } = graphAnswers({ withCatalog: true })
    expect(
      stateRowCounts(
        unlockFaceting,
        unlock.nodes,
        unlockFaceting.empty(),
        FacetId.State,
        stateOrder,
      ),
    ).toEqual({
      [NodeState.Done]: 387,
      [NodeState.Now]: 119,
      [NodeState.Blocked]: 117,
      [NodeState.Partial]: 18,
    })
  })
})

describe('missingGroups', () => {
  // A tainted character's name comes from a message; the stub prints the key and the name.
  const t = (key: string, params?: Record<string, unknown>) =>
    params ? `${key}:${params.name}` : key

  it('groups what stands in the way by kind, in a fixed order, names kept', () => {
    expect(
      missingGroups(
        node({
          missing: [
            { kind: 'unknown', label: 'Collect' },
            {
              kind: 'character',
              id: 10,
              name: 'The Lost',
              tainted: false,
              page: null,
            },
            { kind: 'unknown', label: 'ending' },
            {
              kind: 'item',
              itemKind: 'passive',
              id: 1,
              name: 'The Sad Onion',
              page: null,
            },
          ],
        }),
        t,
      ),
    ).toEqual([
      {
        kind: RequirementKind.Character,
        entries: [{ key: 'character-10', name: 'The Lost', location: null }],
      },
      {
        kind: RequirementKind.Item,
        entries: [{ key: 'item-1', name: 'The Sad Onion', location: null }],
      },
      {
        kind: RequirementKind.Unknown,
        entries: [
          { key: 'unknown-Collect', name: 'Collect', location: null },
          { key: 'unknown-ending', name: 'ending', location: null },
        ],
      },
    ])
  })

  it('keeps a gate apart from an unknown requirement', () => {
    expect(
      missingGroups(
        node({
          missing: [
            { kind: 'gate', label: 'Greedier' },
            { kind: 'boss', id: 6, name: 'Mom', page: null },
          ],
        }),
        t,
      ),
    ).toEqual([
      {
        kind: RequirementKind.Boss,
        entries: [{ key: 'boss-6', name: 'Mom', location: null }],
      },
      {
        kind: RequirementKind.Gate,
        entries: [{ key: 'gate-Greedier', name: 'Greedier', location: null }],
      },
    ])
  })

  it('has nothing to say when nothing is missing', () => {
    expect(missingGroups(node({}), t)).toEqual([])
  })
})

describe('a tainted character in the way', () => {
  const t = (key: string, params?: Record<string, unknown>) =>
    params ? `${key}:${params.name}` : key

  it('is named by its form, not by the name it shares with the base one', () => {
    expect(
      missingGroups(
        node({
          missing: [
            {
              kind: 'character',
              id: 31,
              name: 'The Lost',
              tainted: true,
              page: null,
            },
            {
              kind: 'character',
              id: 10,
              name: 'The Lost',
              tainted: false,
              page: null,
            },
          ],
        }),
        t,
      ),
    ).toEqual([
      {
        kind: RequirementKind.Character,
        entries: [
          {
            key: 'character-31',
            name: 'graph.taintedName:The Lost',
            location: null,
          },
          { key: 'character-10', name: 'The Lost', location: null },
        ],
      },
    ])
  })
})
