import type { ChallengeRow } from '@/lib/ipc/types'

/**
 * The achievement a challenge's "+" would queue, or `null` when there is nothing to offer.
 * The first reward is the one it queues — the rest are counted beside it — and it is offered
 * while it is neither earned nor already in the queue (card #80, P3: the second half was
 * missing, and "+" stayed on a row the queue already held). Unread is not earned: `done` is
 * `null` when section 1 was not read, and the button stays.
 */
export const queueableReward = (
  row: ChallengeRow,
  queued: readonly number[],
): number | null => {
  const first = row.rewards[0]
  if (first === undefined || first.done === true) return null
  return queued.includes(first.achievement) ? null : first.achievement
}
