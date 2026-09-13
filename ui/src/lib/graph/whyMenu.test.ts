import { describe, expect, it } from 'vitest'
import type { LockView, RequirementView, UnlockNode } from '@/lib/ipc/types'
import { RouteName } from '@/router/routeTable'
import { lockWhy, nodeWhy } from './whyMenu'

// The key is what the group carries: a menu names its groups through the messages, so a test
// that translated them would be testing the dictionary instead of the model.
const t = (key: string) => key

const node = (missing: RequirementView[]): UnlockNode => ({
  achievement: {
    kind: 'known',
    id: 1,
    text: 'Dad’s Note',
    condition: null,
    iconUrl: null,
  },
  done: false,
  unlocks: [],
  origin: null,
  missing,
  graph: {
    kind: 'computed',
    availableNow: false,
    blockedBy: missing.length,
    fanOut: 0,
    stepsMissing: 1,
  },
})

describe('nodeWhy', () => {
  it('groups by kind, in the order the why is told', () => {
    const groups = nodeWhy(
      node([
        { kind: 'boss', id: 19, name: 'Gish', page: null },
        {
          kind: 'character',
          id: 1,
          name: 'Magdalene',
          tainted: false,
          page: null,
        },
      ]),
      t,
    )
    expect(groups.map((g) => g.label)).toEqual([
      'graph.why.character',
      'graph.why.boss',
    ])
  })

  it('an entry with a page carries the location that opens it', () => {
    const [group] = nodeWhy(
      node([
        {
          kind: 'item',
          itemKind: 'passive',
          id: 105,
          name: 'The D6',
          page: { kind: 'item', id: 105 },
        },
      ]),
      t,
    )
    expect(group?.entries[0]?.location).toEqual({
      name: RouteName.Wiki,
      query: { category: 'items', page: 'item:105' },
    })
  })

  it('an entry with no page is still an entry, with nowhere to go', () => {
    const [group] = nodeWhy(node([{ kind: 'gate', label: 'greedmode' }]), t)
    expect(group?.entries).toEqual([
      { key: 'gate-greedmode', name: 'greedmode', location: null },
    ])
  })

  it('a node with nothing missing has no groups: the badge is not a trigger', () => {
    expect(nodeWhy(node([]), t)).toEqual([])
  })

  it('a partial node keeps its why: the state does not decide whether there is a menu', () => {
    const partial: UnlockNode = {
      ...node([{ kind: 'unknown', label: 'Guppy' }]),
      graph: { kind: 'partial', blockedBy: 0, fanOut: 0, unknown: 1 },
    }
    expect(nodeWhy(partial, t)).toEqual([
      {
        label: 'graph.why.unknown',
        entries: [{ key: 'unknown-Guppy', name: 'Guppy', location: null }],
      },
    ])
  })
})

describe('lockWhy', () => {
  it('one group, one entry: the achievement that opens the item', () => {
    const lock: LockView = {
      kind: 'locked',
      achievement: 1,
      text: 'Dad’s Note',
      page: { kind: 'achievement', id: 1 },
    }
    expect(lockWhy(lock, t)).toEqual([
      {
        label: 'collection.lockedBy',
        entries: [
          {
            key: 'achievement-1',
            name: 'Dad’s Note',
            location: {
              name: RouteName.Wiki,
              query: { category: 'achievements', page: 'achievement:1' },
            },
          },
        ],
      },
    ])
  })

  it('nothing unlocks a free item: no menu at all', () => {
    expect(lockWhy({ kind: 'free' }, t)).toEqual([])
  })

  it('names the achievement by its number when the catalog has no text for it', () => {
    const lock: LockView = {
      kind: 'unknown',
      achievement: 7,
      text: null,
      page: null,
    }
    expect(lockWhy(lock, t)[0]?.entries[0]?.name).toBe(
      'collection.achievement 7',
    )
  })
})

describe('a threshold', () => {
  const guppy: RequirementView = {
    kind: 'threshold',
    transformation: 0,
    label: 'Guppy',
    current: 1,
    atLeast: 3,
    of: [
      {
        itemKind: 'passive',
        id: 211,
        name: "Guppy's Head",
        unlocked: true,
        page: null,
      },
      {
        itemKind: 'passive',
        id: 212,
        name: "Guppy's Tail",
        unlocked: false,
        page: null,
      },
      {
        itemKind: 'trinket',
        id: 46,
        name: "Kid's Drawing",
        unlocked: false,
        page: null,
      },
    ],
    unresolved: 0,
    page: null,
  }

  it('answers with its own row and one per item still locked', () => {
    const [group] = nodeWhy(node([guppy]), t)
    expect(group.label).toBe('graph.why.threshold')
    expect(group.entries.map((e) => e.name)).toEqual([
      'graph.thresholdName',
      "Guppy's Tail",
      "Kid's Drawing",
    ])
  })

  it('leaves out the items already unlocked: they are not in the way', () => {
    const [group] = nodeWhy(node([guppy]), t)
    expect(group.entries.map((e) => e.name)).not.toContain("Guppy's Head")
  })

  it('gives every row a key of its own', () => {
    const [group] = nodeWhy(node([guppy]), t)
    const keys = group.entries.map((e) => e.key)
    expect(new Set(keys).size).toBe(keys.length)
  })
})
