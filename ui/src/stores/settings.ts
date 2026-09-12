import { defineStore } from 'pinia'
import { ref } from 'vue'
import { StoreId } from '@/lib/constants/stores'
import { isIpcError } from '@/lib/ipc/errors'
import { setScale as saveScale, settings } from '@/lib/ipc/settings'
import type { IpcError } from '@/lib/ipc/types'
import { applyScale } from '@/lib/scale/apply'
import { ScaleAction } from '@/lib/scale/shortcut'
import {
  DefaultScale,
  nextPercent,
  previousPercent,
  snapPercent,
} from '@/lib/scale/steps'
import { assertNever } from '@/lib/assertNever'

// The interface's size, window-wide and profile-free. The size is **applied first and saved
// after**: the user asked for it, so the app is already that size while the file is written,
// and a failed write is said rather than undone.
export const useSettingsStore = defineStore(StoreId.Settings, () => {
  const scale = ref<number>(DefaultScale)
  const saveError = ref<IpcError | null>(null)
  const saveFailed = ref(false)

  const apply = (percent: number): number => {
    const wanted = snapPercent(percent)
    scale.value = wanted
    applyScale(wanted)
    return wanted
  }

  // `main.ts` has already applied what the backend answered before the app mounted; this
  // brings the store to the same value without a second paint.
  const load = async (): Promise<void> => {
    try {
      apply((await settings()).scale)
    } catch {
      apply(DefaultScale)
    }
  }

  const setScale = async (percent: number): Promise<void> => {
    const wanted = apply(percent)
    saveFailed.value = false
    saveError.value = null
    try {
      apply((await saveScale(wanted)).scale)
    } catch (e) {
      saveFailed.value = true
      saveError.value = isIpcError(e) ? e : null
    }
  }

  // `Ctrl` `+`, `Ctrl` `-`, `Ctrl` `0`: the same ladder and the same saved value as the
  // slider, which is what keeps it from being a second scale.
  const step = async (action: ScaleAction): Promise<void> => {
    switch (action) {
      case ScaleAction.In:
        return setScale(nextPercent(scale.value))
      case ScaleAction.Out:
        return setScale(previousPercent(scale.value))
      case ScaleAction.Reset:
        return setScale(DefaultScale)
      default:
        return assertNever(action)
    }
  }

  return { scale, saveFailed, saveError, load, setScale, step }
})
