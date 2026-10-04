import type { ChallengeRow } from '@/lib/ipc/types'
import type { QueueTarget } from '@/lib/plan/queueAction'

/**
 * The achievement a challenge's Actions button puts in the queue: its first reward — the rest
 * are counted beside it. Unread is not earned: `done` is `null` when section 1 was not read,
 * and the button stays offered.
 */
export const challengeQueueTarget = (row: ChallengeRow): QueueTarget | null => {
  const first = row.rewards[0]
  if (first === undefined) return null
  return { achievement: first.achievement, done: first.done === true }
}

/** Whether the row reads "in coda": the same reward its Actions button adds or removes. */
export const isChallengeQueued = (
  row: ChallengeRow,
  queued: ReadonlySet<number>,
): boolean => {
  const target = challengeQueueTarget(row)
  return target !== null && queued.has(target.achievement)
}
