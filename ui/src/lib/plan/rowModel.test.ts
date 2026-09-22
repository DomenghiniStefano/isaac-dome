import { describe, expect, it } from 'vitest'
import { NodeState } from '@/lib/graph/nodeState'
import type { UnlockNode } from '@/lib/ipc/types'
import { RouteName, WikiCategory } from '@/router/routeTable'
import { rowModel } from './rowModel'

const t = ((key: string, params?: Record<string, unknown>) =>
  params ? `${key}:${String(params.name)}` : key) as never

const base: UnlockNode = {
  achievement: {
    kind: 'known',
    id: 484,
    text: 'You unlocked "The Lost"',
    condition: 'Arriva a Home e usa la Red Key',
    iconUrl: 'isaac://achievement/484',
  },
  done: false,
  unlocks: [],
  origin: null,
  missing: [],
  graph: {
    kind: 'computed',
    availableNow: true,
    blockedBy: 0,
    fanOut: 23,
    stepsMissing: 0,
  },
}

// The eight below came from `lib/graph/goalCard.test.ts`, which this module replaces. They
// are carried rather than rewritten: the reasons they exist — B28, the 231 nameless nodes,
// the slot that must go nowhere — did not stop being true because the function moved.
describe('what a row is called', () => {
  // B28: the file writes the base name for both forms, and the five rows of the reference
  // profile were all Tainted characters labelled as their base form. The text is what you
  // get, in its own form — never the achievement's own sentence. The queue used to use that
  // sentence, which is the disagreement this module ends.
  it('leads with what you get, not with what the file says', () => {
    const model = rowModel(
      {
        ...base,
        unlocks: [
          {
            kind: 'character',
            id: 31,
            name: 'The Lost',
            tainted: true,
            page: null,
          },
        ],
      },
      t,
    )
    expect(model.text).toBe('graph.taintedName:The Lost')
    expect(model.text).not.toBe('You unlocked "The Lost"')
  })

  it('names every target when an achievement gives more than one', () => {
    const model = rowModel(
      {
        ...base,
        unlocks: [
          { kind: 'boss', id: 1, name: 'Monstro', page: null },
          { kind: 'boss', id: 2, name: 'Larry Jr.', page: null },
        ],
      },
      t,
    )
    expect(model.text).toContain('Monstro')
    expect(model.text).toContain('Larry Jr.')
  })

  // 231 nodes of the reference profile unlock nothing the catalog knows: the achievement's
  // own text is the only name left, and it is a fallback, not the first choice.
  it('falls back to the achievement text when it unlocks nothing catalogued', () => {
    expect(rowModel(base, t).text).toBe('You unlocked "The Lost"')
  })
})

describe('what the row carries', () => {
  it("carries the game's own unlock condition as the one line under it", () => {
    expect(rowModel(base, t).condition).toBe('Arriva a Home e usa la Red Key')
  })

  it('never invents a condition the file does not state', () => {
    const silent = {
      ...base,
      achievement: { ...base.achievement, condition: null },
    } as UnlockNode
    expect(rowModel(silent, t).condition).toBeNull()
  })

  it('links the row to the achievement page', () => {
    expect(rowModel(base, t).location).toEqual({
      name: RouteName.Wiki,
      query: { category: WikiCategory.Achievements, page: 'achievement:484' },
    })
  })

  // A slot the catalog cannot name has no page, no drawing and no condition: it still shows,
  // and it still goes nowhere rather than to a page built from its slot number.
  it('has nowhere to go for a slot the catalog does not name', () => {
    const model = rowModel(
      { ...base, achievement: { kind: 'unknown', slot: 640 } },
      t,
    )
    expect(model.location).toBeNull()
    expect(model.condition).toBeNull()
    expect(model.art).toBeNull()
    expect(model.text).toContain('640')
  })

  it('carries the fan-out as a number for the sentence to be built from', () => {
    expect(rowModel(base, t).fanOut).toBe(23)
  })
})

// The three this module adds: spec §4.1, where `apre N` takes its colour from whether the
// row can be played tonight.
describe('whether the row can be played tonight', () => {
  it('is playable when the graph says it is available now', () => {
    expect(rowModel(base, t).playable).toBe(true)
  })

  it('is not playable when something is in the way', () => {
    const blocked = {
      ...base,
      graph: { ...base.graph, availableNow: false, blockedBy: 2 },
    } as UnlockNode
    expect(rowModel(blocked, t).playable).toBe(false)
  })

  // The guard §4.1 asks for: the colour is never the only carrier, so the state has to be
  // here as a value the expansion can print as a word.
  it('carries the state as a value, not only as the playable flag', () => {
    expect(rowModel(base, t).state).toBe(NodeState.Now)
    const blocked = {
      ...base,
      graph: { ...base.graph, availableNow: false, blockedBy: 2 },
    } as UnlockNode
    expect(rowModel(blocked, t).state).toBe(NodeState.Blocked)
  })
})
