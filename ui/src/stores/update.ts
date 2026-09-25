import { defineStore } from 'pinia'
import { ref } from 'vue'
import { StoreId } from '@/lib/constants/stores'
import { asIpcError } from '@/lib/ipc/errors'
import { checkUpdate, installUpdate, updateStatus } from '@/lib/ipc/update'
import type { IpcError, UpdateView } from '@/lib/ipc/types'
import { UpdateReason } from '@/lib/ipc/types'

// Where the update stands. **The backend holds the phase**, not this store: a download started
// in one window is the same download in every other, and the `update-changed` event is what
// says "read again". So everything here is `read()` plus two things a button asks for.
export const useUpdateStore = defineStore(StoreId.Update, () => {
  // Until the backend answers, nothing is offered. The alternative is a check button that is
  // live for an instant in a build that has no updater at all.
  const view = ref<UpdateView>({
    currentVersion: '',
    phase: { kind: 'idle' },
    unavailable: UpdateReason.NotSupported,
  })
  const error = ref<IpcError | null>(null)

  // The whole of what the event does, and what the screen does when it mounts. No network:
  // this is the held phase.
  const read = async (): Promise<void> => {
    try {
      view.value = await updateStatus()
    } catch {
      // Nothing answered, so nothing is offered — the same position the store starts in
      // rather than a phase invented out of a failed call.
      view.value = {
        currentVersion: view.value.currentVersion,
        phase: { kind: 'idle' },
        unavailable: UpdateReason.NotSupported,
      }
    }
  }

  // The button. It returns when the whole thing is over — the phases in between arrive through
  // the event, which is also what keeps a second window's bar moving.
  const check = async (): Promise<void> => {
    error.value = null
    try {
      view.value = await checkUpdate()
    } catch (e) {
      error.value = asIpcError(e)
      await read()
    }
  }

  // **When this works it does not return**: the installer is launched and the process ends.
  // Reaching the line after it means the bytes were not there, which is a defect of ours and
  // the only error this screen can raise.
  const install = async (): Promise<void> => {
    error.value = null
    try {
      await installUpdate()
    } catch (e) {
      error.value = asIpcError(e)
    }
    await read()
  }

  return { view, error, read, check, install }
})
