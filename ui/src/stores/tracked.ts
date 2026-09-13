import { isIpcError } from '@/lib/ipc/errors'
import { LoadStatus } from './loadStatus'
import type { Ref } from 'vue'
import type { IpcError } from '@/lib/ipc/types'

// One read, with its status and its error written the same way everywhere.
//
// Every store wrote this `try` / `catch` by hand and only the call in the middle changed.
// `defineViewStore` is the whole store for the three whose shape is just the triad; this is
// the half underneath it, for the three that carry more — the profile loads two things, the
// queue clears its mutation state first, the wiki skips a read it has already made.
//
// An error that is not ours becomes `null`: the status already says it failed, and a foreign
// object is not something the UI can say anything about.
export const tracked = async (
  status: Ref<LoadStatus>,
  error: Ref<IpcError | null>,
  read: () => Promise<void>,
): Promise<void> => {
  status.value = LoadStatus.Loading
  error.value = null
  try {
    await read()
    status.value = LoadStatus.Ready
  } catch (e) {
    error.value = isIpcError(e) ? e : null
    status.value = LoadStatus.Failed
  }
}
