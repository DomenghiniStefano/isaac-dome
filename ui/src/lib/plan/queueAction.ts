import type { QueueView } from '@/lib/ipc/types'
import { rowId } from './queueRows'

// What the Actions button of a list row does. The rules are the Plan's own: an achievement that
// is neither earned nor queued can be added; only a row you asked for can leave — one dragged
// in as a prerequisite goes when its wish does (`GoalRow.vue`).
export const QueueAction = {
  Add: 'add',
  Remove: 'remove',
  Unavailable: 'unavailable',
} as const
export type QueueAction = (typeof QueueAction)[keyof typeof QueueAction]

export interface QueueMembership {
  wanted: Set<number>
  steps: Set<number>
}

/** The achievement a row would put in the queue, and whether it is already earned. */
export interface QueueTarget {
  achievement: number
  done: boolean
}

export const membershipOf = (view: QueueView | null): QueueMembership => {
  const rows = view?.rows ?? []
  return {
    wanted: new Set(rows.filter((r) => r.wanted).map(rowId)),
    steps: new Set(rows.filter((r) => !r.wanted).map(rowId)),
  }
}

export const queueAction = (
  target: QueueTarget | null,
  membership: QueueMembership,
  canWrite: boolean,
): QueueAction => {
  if (!canWrite || target === null) return QueueAction.Unavailable
  if (membership.wanted.has(target.achievement)) return QueueAction.Remove
  if (target.done || membership.steps.has(target.achievement))
    return QueueAction.Unavailable
  return QueueAction.Add
}
