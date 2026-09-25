import { computed } from 'vue'
import { queuedIds } from '@/lib/plan/queueRows'
import { useQueueStore } from '@/stores/queue'

// What a list beside the Plan may offer: which of its rows are already in the queue, and
// whether a `+` can be drawn at all. A queue that couldn't be read or saved offers nothing —
// the rows still show, without "in coda" or the button. Unlock, Challenges, Goals and the wiki's
// profile block all read it here.
export const useQueueOffer = () => {
  const queue = useQueueStore()
  const queued = computed(() => queuedIds(queue.view))
  const canWrite = computed(() => queue.view?.storeAvailable === true)
  return { queue, queued, canWrite }
}
