import { defineStore } from 'pinia'
import { ref } from 'vue'
import { StoreId } from '@/lib/constants/stores'
import {
  queue as readQueue,
  queueAdd,
  queueImportGoals,
  queueMove,
  queueRemove,
} from '@/lib/ipc/queue'
import type { IpcError, QueueView } from '@/lib/ipc/types'
import { LoadStatus } from './loadStatus'
import { attempt, tracked } from './tracked'

export interface QueueMove {
  achievement: number
  after: number | null
}

// The plan queue of the active profile, read by the Plan, Next steps and Unlock. Every write
// answers with the whole view, which replaces what is shown: a move can reorder much of the
// queue, and the backend's order is the truth. A refused write leaves the view as it was.
export const useQueueStore = defineStore(StoreId.Queue, () => {
  const view = ref<QueueView | null>(null)
  const status = ref<LoadStatus>(LoadStatus.Idle)
  const error = ref<IpcError | null>(null)
  const busy = ref(false)
  const mutationFailed = ref(false)
  const mutationError = ref<IpcError | null>(null)
  // The last move that succeeded, so the Plan can say where the row stopped.
  const lastMove = ref<QueueMove | null>(null)

  // A reload clears what the last edit left behind as well as the rows: a failure from the
  // previous queue must not be read as a failure of this one.
  const load = (): Promise<void> => {
    view.value = null
    mutationFailed.value = false
    mutationError.value = null
    lastMove.value = null
    return tracked(status, error, async () => {
      view.value = await readQueue()
    })
  }

  // The last refusal stays on screen until this write has answered: the answer clears it, not
  // the asking.
  const write = async (run: () => Promise<QueueView>): Promise<boolean> => {
    busy.value = true
    lastMove.value = null
    const landed = await attempt(mutationFailed, mutationError, async () => {
      view.value = await run()
    })
    busy.value = false
    return landed
  }

  const add = async (achievement: number): Promise<void> => {
    await write(() => queueAdd(achievement))
  }

  const remove = async (achievement: number): Promise<void> => {
    await write(() => queueRemove(achievement))
  }

  const importGoals = async (): Promise<void> => {
    await write(() => queueImportGoals())
  }

  const move = async (
    achievement: number,
    after: number | null,
  ): Promise<void> => {
    const moved = await write(() => queueMove(achievement, after))
    if (moved) lastMove.value = { achievement, after }
  }

  return {
    view,
    status,
    error,
    busy,
    mutationFailed,
    mutationError,
    lastMove,
    load,
    add,
    remove,
    move,
    importGoals,
  }
})
