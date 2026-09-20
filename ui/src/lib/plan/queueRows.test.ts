import { describe, expect, it } from 'vitest'
import { graphAnswers } from '@/lib/ipc/fixtures/graph'
import type { QueueRow, UnlockNode } from '@/lib/ipc/types'
import {
  achievementText,
  canQueue,
  isQueued,
  originRows,
  proposalLabel,
  queueSummary,
  queuedIds,
  rowId,
  stoppedUnder,
} from './queueRows'

const nodes = graphAnswers({ withCatalog: true }).unlock.nodes
const node = (id: number): UnlockNode => {
  const found = nodes.find(
    (n) => n.achievement.kind === 'known' && n.achievement.id === id,
  )
  if (!found) throw new Error(`no node ${id}`)
  return found
}
const row = (
  id: number,
  wanted: boolean,
  origins: number[] = [],
): QueueRow => ({
  node: node(id),
  wanted,
  origins,
  stepsNotQueued: id === 69 ? 1 : 0,
})
// contracts/payload/queue.with_rows.json: 480 pulled in by 55, 55 wanted, 69 wanted.
const rows = [row(480, false, [55]), row(55, true), row(69, true)]
const order = (...ids: number[]): QueueRow[] =>
  ids.flatMap((id) => rows.filter((r) => rowId(r) === id))

describe('the queue, read', () => {
  it('knows which achievements are queued', () => {
    const view = { rows, diagnostics: [], storeAvailable: true }
    expect([...queuedIds(view)]).toEqual([480, 55, 69])
    expect(queuedIds(null).size).toBe(0)
    expect(isQueued(node(55), queuedIds(view))).toBe(true)
    expect(isQueued(node(484), queuedIds(view))).toBe(false)
  })

  it('counts rows asked for and rows pulled in', () => {
    expect(queueSummary(rows)).toEqual({ rows: 3, wanted: 2, pulledIn: 1 })
  })

  it("names the wish a step serves, and says when it isn't shown", () => {
    expect(originRows(row(480, false, [55]), rows)).toEqual([
      { id: 55, row: rows[1] },
    ])
    expect(originRows(row(480, false, [1]), rows)).toEqual([
      { id: 1, row: null },
    ])
  })

  it("finds an achievement's text among the nodes", () => {
    expect(achievementText(nodes, 1)).toBe('You unlocked "Magdalene"')
    expect(achievementText(nodes, 9999)).toBeNull()
  })
})

describe('proposalLabel', () => {
  it('names what a step unlocks, which a narrow column can still show', () => {
    expect(proposalLabel(node(484))).toBe('The Lost')
  })

  it("falls back to the achievement's text when it unlocks nothing catalogued", () => {
    expect(proposalLabel(node(69))).toBe('!Platinum God! OMG!')
  })
})

describe('canQueue', () => {
  const queued = new Set([480, 55, 69])

  it('offers a known achievement not done and not queued', () => {
    expect(canQueue(node(484), queued)).toBe(true)
  })

  it('never a done one, an unknown one or one already queued', () => {
    expect(canQueue(node(1), queued)).toBe(false)
    expect(canQueue(node(55), queued)).toBe(false)
    // Slot 640 is the one unknown achievement the reference profile hasn't done.
    const unknown = nodes.find(
      (n) => n.achievement.kind === 'unknown' && !n.done,
    )
    expect(unknown).toBeDefined()
    expect(unknown && canQueue(unknown, queued)).toBe(false)
  })
})

describe('stoppedUnder', () => {
  it('is the row above, when a row sent to the top stopped short', () => {
    expect(stoppedUnder(order(480, 55, 69), 55, null)).toBe(rows[0])
  })

  it('is nothing when the row reached the gap it was dropped in', () => {
    expect(stoppedUnder(order(69, 480, 55), 480, 69)).toBeNull()
    expect(stoppedUnder(order(55, 480, 69), 55, null)).toBeNull()
  })

  it('is nothing when the row was dropped under one of its own dependents', () => {
    // 480 dropped under 55, which needs it: 480 sits right above 55, not below it.
    expect(stoppedUnder(order(69, 480, 55), 480, 55)).toBeNull()
  })

  it('is nothing when the moved row or its anchor is no longer shown', () => {
    expect(stoppedUnder(order(480, 55, 69), 484, null)).toBeNull()
    expect(stoppedUnder(order(480, 55, 69), 55, 1)).toBeNull()
  })
})
