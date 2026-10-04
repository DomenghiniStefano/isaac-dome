import { computed } from 'vue'
import { membershipOf } from '@/lib/plan/queueAction'
import { queuedIds } from '@/lib/plan/queueRows'
import { useQueueStore } from '@/stores/queue'

// What a list beside the Plan may offer: which of its rows are already in the queue, which of
// those you asked for, and whether the queue can be written at all. A queue that couldn't be
// read or saved offers nothing — the rows still show, with every Actions button disabled. The
// list tables, Goals and the wiki's profile block all read it here.
export const useQueueOffer = () => {
  const queue = useQueueStore()
  const queued = computed(() => queuedIds(queue.view))
  const membership = computed(() => membershipOf(queue.view))
  const canWrite = computed(() => queue.view?.storeAvailable === true)
  return { queue, queued, membership, canWrite }
}
