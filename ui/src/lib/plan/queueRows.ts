import { knownId, knownText, nodeWithId } from '@/lib/graph/achievementNode'
import { nodeSlot } from '@/lib/graph/unlockFacets'
import type { QueueRow, QueueView, UnlockNode } from '@/lib/ipc/types'

// A queue row is always a known achievement (the view leaves unresolved ones out), so its slot
// is its id.
export const rowId = (row: QueueRow): number => nodeSlot(row.node)

export const achievementText = (
  nodes: UnlockNode[],
  id: number,
): string | null => {
  const found = nodeWithId(nodes, id)
  return found ? knownText(found) : null
}

// A step in a narrow column: what it unlocks ("The Lost") reads where "You unlocked…" is cut to
// "You …"; the achievement's own text when it unlocks nothing catalogued.
export const proposalLabel = (node: UnlockNode): string | null =>
  node.unlocks[0]?.name ?? knownText(node)

export const queuedIds = (view: QueueView | null): Set<number> =>
  new Set(view?.rows.map(rowId) ?? [])

export const isQueued = (node: UnlockNode, queued: Set<number>): boolean => {
  const id = knownId(node)
  return id !== null && queued.has(id)
}

// What can be put in the queue: a known achievement, not done, not already there.
export const canQueue = (node: UnlockNode, queued: Set<number>): boolean =>
  knownId(node) !== null && !node.done && !isQueued(node, queued)

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
