import { assertNever } from '@/lib/assertNever'
import type { PageProgress, Target } from '@/lib/ipc/types'
import type { QueueTarget } from '@/lib/plan/queueAction'

// What a wiki row's Actions button puts in the Plan's queue: the achievement the page is, or the
// one that unlocks what the page is about while it is still locked — or while the save could not
// say, since unread is never "earned", as in the Collection. Pages with nothing to earn
// behind them — articles, monsters, a character's marks — offer nothing.
export const pageQueueTarget = (
  target: Target,
  progress: PageProgress | null,
): QueueTarget | null => {
  if (target.kind === 'achievement')
    return {
      achievement: target.id,
      done: progress?.kind === 'achievement' && progress.done,
    }
  if (progress === null) return null
  switch (progress.kind) {
    case 'item':
      return progress.unlocked !== true && progress.unlockedBy !== null
        ? { achievement: progress.unlockedBy, done: false }
        : null
    case 'unlockable':
      return progress.unlocked
        ? null
        : { achievement: progress.unlockedBy, done: false }
    case 'achievement':
    case 'character':
    case 'challenge':
    case 'bestiary':
      return null
    default:
      return assertNever(progress)
  }
}
