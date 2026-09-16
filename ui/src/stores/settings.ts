import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { StoreId } from '@/lib/constants/stores'
import { isIpcError } from '@/lib/ipc/errors'
import {
  setResumeTabs as saveResumeTabs,
  setScale as saveScale,
  setStayInBackground as saveStayInBackground,
  settings,
  autostart as readAutostart,
  setAutostart as saveAutostart,
} from '@/lib/ipc/settings'
import type { AutostartView, IpcError } from '@/lib/ipc/types'
import { AutostartReason } from '@/lib/ipc/types'
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
  // The backend's own defaults, so a `load()` that fails leaves the switches saying what the
  // app actually does.
  const stayInBackground = ref(true)
  const resumeTabs = ref(true)
  // The registry is the truth and nothing here mirrors it: until it answers, the switch is not
  // offered at all.
  const autostart = ref(false)
  const autostartUnavailable = ref<AutostartView['unavailable']>(
    AutostartReason.NotSupported,
  )
  const autostartAvailable = computed(() => autostartUnavailable.value === null)

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
      const stored = await settings()
      apply(stored.scale)
      stayInBackground.value = stored.stayInBackground
      resumeTabs.value = stored.resumeTabs
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

  // The two switches of the Background screen. Same rule as the size: **applied first, saved
  // after**, and a failed write is said rather than undone — except that here the app's actual
  // behaviour is the backend's, so the answer is what the switch ends up showing.
  const setStayInBackground = async (stay: boolean): Promise<void> => {
    stayInBackground.value = stay
    saveFailed.value = false
    saveError.value = null
    try {
      stayInBackground.value = (
        await saveStayInBackground(stay)
      ).stayInBackground
    } catch (e) {
      saveFailed.value = true
      saveError.value = isIpcError(e) ? e : null
    }
  }

  // Starting with Windows, and it is the one switch here that **moves after the answer**.
  //
  // The three above it change what the app is doing *now*, so the honest thing is to do it and
  // report a failed write. This one changes what happens at the next login: there is nothing to
  // already be doing, the truth is in the registry and not in `settings.json`, and a switch left
  // on because it was clicked would be a promise nothing kept.
  const refreshAutostart = async (): Promise<void> => {
    try {
      const view = await readAutostart()
      autostart.value = view.enabled
      autostartUnavailable.value = view.unavailable
    } catch {
      // Nothing answered, so nothing is offered. The switch showing "off" here would be a
      // position the registry never took.
      autostart.value = false
      autostartUnavailable.value = AutostartReason.NotSupported
    }
  }

  const setAutostart = async (on: boolean): Promise<void> => {
    saveFailed.value = false
    saveError.value = null
    try {
      const view = await saveAutostart(on)
      autostart.value = view.enabled
      autostartUnavailable.value = view.unavailable
    } catch (e) {
      saveFailed.value = true
      saveError.value = isIpcError(e) ? e : null
    }
  }

  const setResumeTabs = async (resume: boolean): Promise<void> => {
    resumeTabs.value = resume
    saveFailed.value = false
    saveError.value = null
    try {
      resumeTabs.value = (await saveResumeTabs(resume)).resumeTabs
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

  return {
    scale,
    stayInBackground,
    resumeTabs,
    saveFailed,
    saveError,
    load,
    setScale,
    setStayInBackground,
    setResumeTabs,
    autostart,
    autostartUnavailable,
    autostartAvailable,
    refreshAutostart,
    setAutostart,
    step,
  }
})
