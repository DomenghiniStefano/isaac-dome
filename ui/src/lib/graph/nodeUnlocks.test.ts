import { describe, expect, it } from 'vitest'
import type { UnlockNode, UnlockTarget } from '@/lib/ipc/types'
import { RouteName, WikiCategory } from '@/router/routeTable'
import { nodeUnlocks } from './nodeUnlocks'

// The real `t` formats `graph.taintedName`; here the key is enough to see which one was used.
const t = ((key: string, params?: Record<string, unknown>) =>
  params ? `${key}:${String(params.name)}` : key) as never

const nodeWith = (unlocks: UnlockTarget[]): UnlockNode => ({
  achievement: { kind: 'known', id: 1, text: 't', hint: null, iconUrl: null },
  done: false,
  unlocks,
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

describe('nodeUnlocks', () => {
  it('sends an entry to the page its target carries', () => {
    const [entry] = nodeUnlocks(
      nodeWith([
        {
          kind: 'item',
          itemKind: 'passive',
          id: 105,
          name: 'The D6',
          iconUrl: 'isaac://item/passive/105',
          page: { kind: 'item', id: 105 },
        },
      ]),
      t,
    )
    expect(entry).toMatchObject({
      name: 'The D6',
      iconUrl: 'isaac://item/passive/105',
      location: {
        name: RouteName.Wiki,
        query: { category: WikiCategory.Items, page: 'item:105' },
      },
    })
  })

  // Never a link that leads nowhere: the row shows, it just does not go anywhere.
  it('leaves a target with no page without a location', () => {
    const [entry] = nodeUnlocks(
      nodeWith([{ kind: 'boss', id: 3, name: 'Nameless', page: null }]),
      t,
    )
    expect(entry.name).toBe('Nameless')
    expect(entry.location).toBeNull()
  })

  // A boss's page is an entity key taken from its portrait's file name, which the frontend
  // cannot build: it either arrives on the target or the row is text (B35).
  it('follows a boss to its entity page when the target carries one', () => {
    const [entry] = nodeUnlocks(
      nodeWith([
        {
          kind: 'boss',
          id: 1,
          name: 'Monstro',
          page: { kind: 'entity', id: 20, variant: 0, subtype: 0 },
        },
      ]),
      t,
    )
    expect(entry.location).toEqual({
      name: RouteName.Wiki,
      query: { category: WikiCategory.Bosses, page: 'entity:20.0.0' },
    })
  })

  // B28: the two forms share the game's name, so the name alone names both.
  it('names a tainted character by its form', () => {
    const [entry] = nodeUnlocks(
      nodeWith([
        {
          kind: 'character',
          id: 31,
          name: 'The Lost',
          tainted: true,
          page: null,
        },
      ]),
      t,
    )
    expect(entry.name).toBe('graph.taintedName:The Lost')
  })

  it('keys a row by its kind and id, so two targets never collide', () => {
    const entries = nodeUnlocks(
      nodeWith([
        { kind: 'boss', id: 3, name: 'A', page: null },
        { kind: 'character', id: 3, name: 'B', tainted: false, page: null },
      ]),
      t,
    )
    expect(new Set(entries.map((e) => e.key)).size).toBe(2)
  })

  it('has no rows for a node that unlocks nothing catalogued', () => {
    expect(nodeUnlocks(nodeWith([]), t)).toEqual([])
  })
})
