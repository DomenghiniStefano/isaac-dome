import { nodeSlot } from '@/lib/graph/unlockFilter'
import type { QueueRow, QueueView, UnlockNode } from '@/lib/ipc/types'

// A queue row is always a known achievement (the view leaves unresolved ones out), so its slot
// is its id.
export const rowId = (row: QueueRow): number => nodeSlot(row.node)

export const knownText = (node: UnlockNode): string | null =>
  node.achievement.kind === 'known' ? node.achievement.text : null

export const achievementText = (
  nodes: UnlockNode[],
  id: number,
): string | null => {
  const found = nodes.find(
    (n) => n.achievement.kind === 'known' && n.achievement.id === id,
  )
  return found ? knownText(found) : null
}

export const queuedIds = (view: QueueView | null): Set<number> =>
  new Set(view?.rows.map(rowId) ?? [])

export const isQueued = (node: UnlockNode, queued: Set<number>): boolean =>
  node.achievement.kind === 'known' && queued.has(node.achievement.id)

// What can be put in the queue: a known achievement, not done, not already there.
export const canQueue = (node: UnlockNode, queued: Set<number>): boolean =>
  node.achievement.kind === 'known' && !node.done && !isQueued(node, queued)

export interface QueueSummary {
  rows: number
  wanted: number
  pulledIn: number
}

export const queueSummary = (rows: QueueRow[]): QueueSummary => {
  const wanted = rows.filter((r) => r.wanted).length
  return { rows: rows.length, wanted, pulledIn: rows.length - wanted }
}

export interface OriginRow {
  id: number
  row: QueueRow | null
}

// The wishes a row serves, each with its row when the view shows it.
export const originRows = (row: QueueRow, rows: QueueRow[]): OriginRow[] =>
  row.origins.map((id) => ({
    id,
    row: rows.find((r) => rowId(r) === id) ?? null,
  }))

// The row a moved one stopped under. A move stops short only on the way up — dependents are
// dragged, never jumped — so it stopped short when it sits below the gap it was dropped in; the
// wall is then the row right above it.
export const stoppedUnder = (
  rows: QueueRow[],
  achievement: number,
  after: number | null,
): QueueRow | null => {
  const index = rows.findIndex((r) => rowId(r) === achievement)
  const anchor = rows.findIndex((r) => rowId(r) === after)
  if (index < 0 || (after !== null && anchor < 0)) return null
  const asked = after === null ? 0 : anchor + 1
  return index > asked ? (rows[index - 1] ?? null) : null
}
