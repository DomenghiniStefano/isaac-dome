import { describe, expect, it } from 'vitest'
import { graphAnswers } from '@/lib/ipc/fixtures/graph'
import type { QueueRow, UnlockNode } from '@/lib/ipc/types'
import { queueHint } from './queueHint'

const t = (key: string, args?: Record<string, unknown>): string =>
  args ? `«${key}:${JSON.stringify(args)}»` : `«${key}»`

const nodes = graphAnswers({ withCatalog: true }).unlock.nodes
const node = (id: number): UnlockNode => {
  const found = nodes.find(
    (n) => n.achievement.kind === 'known' && n.achievement.id === id,
  )
  if (!found) throw new Error(`no node ${id}`)
  return found
}
const row = (achievement: UnlockNode): QueueRow => ({
  node: achievement,
  wanted: true,
  origins: [],
  stepsNotQueued: 0,
})
// 55 moved to the top, and stopped under 480, which it depends on.
const rows = [row(node(480)), row(node(55)), row(node(69))]
const textOf = (id: number): string => {
  const a = node(id).achievement
  return a.kind === 'known' ? a.text : ''
}

describe('queueHint', () => {
  it('says how a drag works while one is under way', () => {
    expect(queueHint(t, true, rows, { achievement: 55, after: null })).toBe(
      '«plan.hint.dragging»',
    )
  })

  it('says how to drag when no move stopped short', () => {
    expect(queueHint(t, false, rows, null)).toBe('«plan.hint.idle»')
    expect(queueHint(t, false, rows, { achievement: 69, after: 55 })).toBe(
      '«plan.hint.idle»',
    )
  })

  it('names the row a move stopped under', () => {
    expect(queueHint(t, false, rows, { achievement: 55, after: null })).toBe(
      `«plan.hint.stoppedUnder:${JSON.stringify({ name: textOf(480) })}»`,
    )
  })

  it('numbers the wall when the catalog cannot name it', () => {
    const slot = row({
      ...node(480),
      achievement: { kind: 'unknown', slot: 7 },
    })
    const named = queueHint(t, false, [slot, rows[1] as QueueRow], {
      achievement: 55,
      after: null,
    })
    expect(named).toBe(
      `«plan.hint.stoppedUnder:${JSON.stringify({ name: '«plan.achievementNumbered:{"id":7}»' })}»`,
    )
  })
})
