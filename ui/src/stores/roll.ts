import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Ref } from 'vue'
import { StoreId } from '@/lib/constants/stores'
import { roll, rollDraw, setRollPreset } from '@/lib/ipc/roll'
import type { IpcError, PresetView, RollView } from '@/lib/ipc/types'
import { LoadStatus } from './loadStatus'
import { tracked } from './tracked'

// The draw. Every command answers with the whole view, so a write is a read: there is no
// second path by which this store can learn what changed, and therefore no way for the card
// and the panel to disagree.
export const useRollStore = defineStore(StoreId.Roll, () => {
  const view = ref<RollView | null>(null) as Ref<RollView | null>
  const status = ref<LoadStatus>(LoadStatus.Idle)
  const error = ref<IpcError | null>(null)

  const run = (command: () => Promise<RollView>): Promise<void> =>
    tracked(status, error, async () => {
      view.value = await command()
    })

  // The view belongs to one profile: it is cleared before a read, never after, so the old
  // one is not shown under a profile it was never read from.
  const load = (): Promise<void> => {
    view.value = null
    return run(roll)
  }

  // A draw and a preset change keep the card on screen while they run: they are the same
  // profile, and blanking it would flicker the thing the user is looking at.
  const draw = (): Promise<void> => run(rollDraw)
  const setPreset = (preset: PresetView): Promise<void> =>
    run(() => setRollPreset(preset))

  return { view, status, error, load, draw, setPreset }
})
