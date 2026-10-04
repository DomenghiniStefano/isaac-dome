import { assertNever } from '@/lib/assertNever'
import type { CollectionItem } from '@/lib/ipc/types'
import type { QueueTarget } from '@/lib/plan/queueAction'

// What an item's Actions button puts in the Plan's queue: the achievement that unlocks it, while
// it is locked — or while the save could not say, since unread is never "earned". An unlocked or
// free item has nothing left to earn.
export const itemQueueTarget = (item: CollectionItem): QueueTarget | null => {
  const lock = item.lock
  switch (lock.kind) {
    case 'locked':
    case 'unknown':
      return { achievement: lock.achievement, done: false }
    case 'unlocked':
    case 'free':
      return null
    default:
      return assertNever(lock)
  }
}
