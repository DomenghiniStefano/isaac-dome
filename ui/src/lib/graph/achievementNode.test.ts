import { describe, expect, it } from 'vitest'
import type { UnlockNode, UnlockView } from '@/lib/ipc/types'
import { achievementNode } from './achievementNode'

const node = (id: number): UnlockNode => ({
  achievement: { kind: 'known', id, text: `t${id}`, hint: null, iconUrl: null },
  done: false,
  unlocks: [],
  origin: null,
  missing: [],
  graph: {
    kind: 'computed',
    availableNow: true,
    blockedBy: 0,
    fanOut: 0,
    stepsMissing: 0,
  },
})

const view = (
  nodes: UnlockNode[],
  diagnostics: UnlockView['diagnostics'] = [],
): UnlockView => ({
  nodes,
  totals: { slots: 0, done: 0, known: 0, unknown: 0 },
  diagnostics,
})

describe('achievementNode', () => {
  it('finds the node the page names', () => {
    const found = achievementNode(view([node(1), node(19)]), {
      kind: 'achievement',
      id: 19,
    })
    expect(found?.achievement).toMatchObject({ id: 19 })
  })

  it('answers nothing for a page that is not an achievement', () => {
    expect(achievementNode(view([node(1)]), { kind: 'item', id: 1 })).toBeNull()
  })

  it('answers nothing for an achievement the catalog does not know', () => {
    expect(
      achievementNode(view([node(1)]), { kind: 'achievement', id: 999 }),
    ).toBeNull()
  })

  it('answers nothing without a view', () => {
    expect(achievementNode(null, { kind: 'achievement', id: 1 })).toBeNull()
  })

  it('answers nothing without a page', () => {
    expect(achievementNode(view([node(1)]), null)).toBeNull()
  })

  // Every node would read "not done", which is not a fact about the profile: section 1 of
  // the save was never read. A block that says it is worse than no block at all.
  it('answers nothing when the achievement section was not read', () => {
    const v = view([node(1)], [{ kind: 'noAchievementSection' }])
    expect(achievementNode(v, { kind: 'achievement', id: 1 })).toBeNull()
  })

  // A slot the catalog cannot name has no id to match, and must never be matched by
  // position: slot 640 is not achievement 640.
  it('never matches an unknown slot by its number', () => {
    const unknown: UnlockNode = {
      ...node(1),
      achievement: { kind: 'unknown', slot: 640 },
    }
    expect(
      achievementNode(view([unknown]), { kind: 'achievement', id: 640 }),
    ).toBeNull()
  })
})
