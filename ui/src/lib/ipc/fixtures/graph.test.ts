import { describe, expect, it } from 'vitest'
import { graphAnswers } from './graph'

// The committed payloads under ui/fixtures/ (payload/unlock.json and next_steps.json), the
// reference profile on 2026-09-08. The numbers are read off those files, not off our code.
describe('graphAnswers with the game installed', () => {
  const { unlock, steps } = graphAnswers({ withCatalog: true })

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

  it('answers the five steps, most fan-out first, as one section', () => {
    // The pack's payload predates the sections, so it reads as the one basis it carries.
    expect(steps.sections.map((s) => s.basis)).toEqual(['fanOut'])
    expect(
      steps.sections[0].steps.map((s) =>
        s.achievement.kind === 'known' ? s.achievement.id : null,
      ),
    ).toEqual([484, 488, 489, 479, 480])
  })

  // The payload's `isaac://` links resolve to nothing here: the app cuts its sprites from the
  // user's own copy of the game at runtime, and the development server has no copy.
  it('answers no image', () => {
    const urls = unlock.nodes.flatMap((n) => [
      n.achievement.kind === 'known' ? n.achievement.iconUrl : null,
      ...n.unlocks.map((t) => (t.kind === 'item' ? t.iconUrl : null)),
    ])
    expect(urls.every((u) => u === null)).toBe(true)
  })

  it('fills the page a payload written before the field existed has not', () => {
    const { unlock } = graphAnswers({ withCatalog: true })
    const missing = unlock.nodes.flatMap((n) => n.missing)
    expect(missing.length).toBeGreaterThan(0)
    // An absent key and an explicit null read the same in a template and type-check
    // differently: the fixture has to answer null, not nothing.
    missing.forEach((r) => {
      if (
        r.kind === 'gate' ||
        r.kind === 'unknown' ||
        r.kind === 'mark' ||
        r.kind === 'counter'
      )
        return
      expect(r.page === null || typeof r.page === 'object').toBe(true)
    })
  })
})

// What crates/ipc/src/graph.rs `unlock_view` sends without a catalog: one node per slot, an
// unknown achievement with the save's done, a partial graph with one unknown, and no steps.
describe('graphAnswers without the game', () => {
  const { unlock, steps } = graphAnswers({ withCatalog: false })

  it('keeps one unknown, partial node per slot, with the save’s done', () => {
    const withCatalog = graphAnswers({ withCatalog: true })
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
    // No sections at all, not an empty section: a heading over nothing is not a state.
    expect(steps).toEqual({ sections: [] })
  })
})
