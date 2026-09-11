import { describe, expect, it } from 'vitest'
import { graphAnswers } from '@/lib/ipc/fixtures/graph'
import type { GraphInfo, UnlockNode } from '@/lib/ipc/types'
import {
  NodeState,
  RequirementKind,
  missingGroups,
  nodeState,
  stateCounts,
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
  achievement: { kind: 'known', id: 1, text: 'a', hint: null, iconUrl: null },
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

describe('stateCounts on the reference profile', () => {
  it('counts the four states of the committed payload', () => {
    const { unlock } = graphAnswers({ withArt: false, withCatalog: true })
    expect(stateCounts(unlock.nodes)).toEqual({
      [NodeState.Done]: 387,
      [NodeState.Now]: 119,
      [NodeState.Blocked]: 117,
      [NodeState.Partial]: 18,
    })
  })
})

describe('missingGroups', () => {
  it('groups what stands in the way by kind, in a fixed order, names kept', () => {
    expect(
      missingGroups(
        node({
          missing: [
            { kind: 'unknown', label: 'Collect' },
            { kind: 'character', id: 10, name: 'The Lost' },
            { kind: 'unknown', label: 'ending' },
            { kind: 'item', itemKind: 'passive', id: 1, name: 'The Sad Onion' },
          ],
        }),
      ),
    ).toEqual([
      { kind: RequirementKind.Character, names: ['The Lost'] },
      { kind: RequirementKind.Item, names: ['The Sad Onion'] },
      { kind: RequirementKind.Unknown, names: ['Collect', 'ending'] },
    ])
  })

  it('keeps a gate apart from an unknown requirement', () => {
    expect(
      missingGroups(
        node({
          missing: [
            { kind: 'gate', label: 'Greedier' },
            { kind: 'boss', id: 6, name: 'Mom' },
          ],
        }),
      ),
    ).toEqual([
      { kind: RequirementKind.Boss, names: ['Mom'] },
      { kind: RequirementKind.Gate, names: ['Greedier'] },
    ])
  })

  it('has nothing to say when nothing is missing', () => {
    expect(missingGroups(node({}))).toEqual([])
  })
})
