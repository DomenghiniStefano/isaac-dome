import { describe, expect, it } from 'vitest'
import type { UnlockNode, WantState, WantView } from '@/lib/ipc/types'
import { WantBlockKind, wantBanner, wantBlocks } from './wantBlocks'

const node = (id: number): UnlockNode => ({
  achievement: {
    kind: 'known',
    id,
    text: `t${id}`,
    condition: null,
    iconUrl: null,
  },
  done: false,
  unlocks: [],
  origin: null,
  missing: [],
  graph: {
    kind: 'computed',
    availableNow: false,
    blockedBy: 1,
    fanOut: 0,
    stepsMissing: 1,
  },
})

const view = (state: WantState): WantView => ({
  wanted: { kind: 'achievement', achievement: node(9).achievement },
  routes: [{ node: node(9), state }],
  diagnostics: [],
})

describe('wantBlocks', () => {
  it('a chain draws its steps and offers the queue', () => {
    const b = wantBlocks(
      view({ kind: 'chain', steps: [node(1), node(2)], unknown: 0 }),
      new Set(),
    )
    expect(b).toHaveLength(1)
    expect(b[0]?.kind).toBe(WantBlockKind.Chain)
    expect(b[0]?.steps).toHaveLength(2)
    expect(b[0]?.queueable).toBe(true)
  })

  it('a want already queued is not offered again', () => {
    const b = wantBlocks(
      view({ kind: 'chain', steps: [node(1)], unknown: 0 }),
      new Set([9]),
    )
    expect(b[0]?.queueable).toBe(false)
  })

  // Done and availableNow are answers, not empty chains: neither draws a step list.
  it('a want you have draws no steps and no button', () => {
    const b = wantBlocks(view({ kind: 'done' }), new Set())
    expect(b[0]?.kind).toBe(WantBlockKind.Done)
    expect(b[0]?.steps).toEqual([])
    expect(b[0]?.queueable).toBe(false)
  })

  it('a want with nothing in the way can still go in the plan', () => {
    const b = wantBlocks(view({ kind: 'availableNow' }), new Set())
    expect(b[0]?.kind).toBe(WantBlockKind.AvailableNow)
    expect(b[0]?.steps).toEqual([])
    expect(b[0]?.queueable).toBe(true)
  })

  it('without a profile the route is named and nothing is claimed', () => {
    const b = wantBlocks(view({ kind: 'noProfile' }), new Set())
    expect(b[0]?.kind).toBe(WantBlockKind.NoProfile)
    expect(b[0]?.queueable).toBe(false)
  })

  it('the unknown count travels with the chain', () => {
    const b = wantBlocks(
      view({ kind: 'chain', steps: [node(1)], unknown: 2 }),
      new Set(),
    )
    expect(b[0]?.unknown).toBe(2)
  })

  it('the banner is the first diagnostic, or nothing', () => {
    expect(wantBanner(view({ kind: 'done' }))).toBeNull()
    expect(
      wantBanner({
        ...view({ kind: 'done' }),
        diagnostics: [{ kind: 'nothingUnlocks' }],
      }),
    ).toEqual({ kind: 'nothingUnlocks' })
  })
})
