import { describe, expect, it } from 'vitest'
import type { UnlockNode } from '@/lib/ipc/types'
import { RouteName, WikiCategory } from '@/router/routeTable'
import { goalCard } from './goalCard'

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

describe('goalCard', () => {
  // B28: the file writes the base name for both forms, and the five rows of the reference
  // profile were all Tainted characters labelled as their base form. The headline is what
  // you get, in its own form — never the achievement's own text.
  it('leads with what you get, not with what the file says', () => {
    const card = goalCard(
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
    expect(card.headline).toBe('graph.taintedName:The Lost')
    expect(card.headline).not.toBe('You unlocked "The Lost"')
  })

  it('names every target when an achievement gives more than one', () => {
    const card = goalCard(
      {
        ...base,
        unlocks: [
          { kind: 'boss', id: 1, name: 'Monstro', page: null },
          { kind: 'boss', id: 2, name: 'Larry Jr.', page: null },
        ],
      },
      t,
    )
    expect(card.headline).toContain('Monstro')
    expect(card.headline).toContain('Larry Jr.')
  })

  // 231 nodes of the reference profile unlock nothing the catalog knows: the achievement's
  // own text is the only name left, and it is a fallback, not the headline's first choice.
  it('falls back to the achievement text when it unlocks nothing catalogued', () => {
    expect(goalCard(base, t).headline).toBe('You unlocked "The Lost"')
  })

  it("carries the game's own unlock condition as the one line under it", () => {
    expect(goalCard(base, t).condition).toBe('Arriva a Home e usa la Red Key')
  })

  it('has no condition when the file states none', () => {
    const card = goalCard(
      {
        ...base,
        achievement: { ...base.achievement, condition: null },
      } as UnlockNode,
      t,
    )
    expect(card.condition).toBeNull()
  })

  it('links the whole card to the achievement page', () => {
    expect(goalCard(base, t).location).toEqual({
      name: RouteName.Wiki,
      query: { category: WikiCategory.Achievements, page: 'achievement:484' },
    })
  })

  // A slot the catalog cannot name has no page, no drawing and no condition: it still shows,
  // and it still goes nowhere rather than to a page built from its slot number.
  it('has nowhere to go for a slot the catalog does not name', () => {
    const card = goalCard(
      { ...base, achievement: { kind: 'unknown', slot: 640 } },
      t,
    )
    expect(card.location).toBeNull()
    expect(card.condition).toBeNull()
    expect(card.art).toBeNull()
    expect(card.headline).toContain('640')
  })

  it('carries the fan-out as a number for the sentence to be built from', () => {
    expect(goalCard(base, t).fanOut).toBe(23)
  })
})
