import { describe, expect, it } from 'vitest'
import { packIconUrl } from './graphArt'
import { graphAnswers } from './graph'

// The design pack's committed payloads (contracts/payload/unlock.json and next_steps.json),
// the reference profile on 2026-09-08. The numbers are read off those files, not off our code.
describe('graphAnswers with the game installed', () => {
  const { unlock, steps } = graphAnswers({ withArt: false, withCatalog: true })

  it('answers every node of the reference profile', () => {
    expect(unlock.nodes).toHaveLength(641)
    expect(unlock.totals).toEqual({
      slots: 642,
      done: 387,
      known: 637,
      unknown: 4,
    })
    expect(unlock.diagnostics).toEqual([
      { kind: 'slotsBeyondCatalog', count: 4 },
    ])
  })

  it('answers the five steps, most fan-out first', () => {
    expect(steps.basis).toBe('fanOut')
    expect(
      steps.steps.map((s) =>
        s.achievement.kind === 'known' ? s.achievement.id : null,
      ),
    ).toEqual([484, 488, 489, 479, 480])
  })

  it('answers no image without art', () => {
    const urls = unlock.nodes.flatMap((n) => [
      n.achievement.kind === 'known' ? n.achievement.iconUrl : null,
      ...n.unlocks.map((t) => (t.kind === 'item' ? t.iconUrl : null)),
    ])
    expect(urls.every((u) => u === null)).toBe(true)
  })

  it("answers the pack's image files with art", () => {
    const withArt = graphAnswers({ withArt: true, withCatalog: true })
    const first = withArt.unlock.nodes[0]?.achievement
    expect(first?.kind === 'known' ? first.iconUrl : null).toMatch(
      /\/0001_you_unlocked_magdalene\.png$/,
    )
  })
})

// What crates/ipc/src/graph.rs `unlock_view` sends without a catalog: one node per slot, an
// unknown achievement with the save's done, a partial graph with one unknown, and no steps.
describe('graphAnswers without the game', () => {
  const { unlock, steps } = graphAnswers({ withArt: true, withCatalog: false })

  it('keeps one unknown, partial node per slot, with the save’s done', () => {
    const withCatalog = graphAnswers({ withArt: false, withCatalog: true })
    expect(unlock.nodes).toHaveLength(641)
    expect(unlock.nodes.map((n) => n.done)).toEqual(
      withCatalog.unlock.nodes.map((n) => n.done),
    )
    expect(unlock.nodes[0]).toMatchObject({
      achievement: { kind: 'unknown', slot: 1 },
      unlocks: [],
      origin: null,
      missing: [],
      graph: { kind: 'partial', blockedBy: 0, fanOut: 0, unknown: 1 },
    })
    expect(unlock.nodes.filter((n) => n.done)).toHaveLength(387)
    expect(unlock.nodes[640]?.achievement).toEqual({
      kind: 'unknown',
      slot: 641,
    })
  })

  it('says why, and proposes nothing', () => {
    expect(unlock.totals).toEqual({
      slots: 642,
      done: 387,
      known: 0,
      unknown: 641,
    })
    expect(unlock.diagnostics).toEqual([{ kind: 'noCatalog' }])
    expect(steps).toEqual({ steps: [], basis: 'fanOut' })
  })
})

describe('packIconUrl', () => {
  it('finds an achievement drawing by its four-digit id', () => {
    expect(packIconUrl('isaac://achievement/1')).toMatch(
      /\/0001_you_unlocked_magdalene\.png$/,
    )
  })

  it('finds an item sprite by its kind and id', () => {
    expect(packIconUrl('isaac://item/familiar/73')).toMatch(
      /\/familiar_0073_[a-z0-9_]+\.png$/,
    )
  })

  it('answers nothing for a link that is not an icon of the pack', () => {
    expect(packIconUrl('isaac://mark/0/hard')).toBeNull()
    expect(packIconUrl('isaac://achievement/9999')).toBeNull()
    expect(packIconUrl(null)).toBeNull()
  })
})
