import { asIpcError } from '@/lib/ipc/errors'
import { LoadStatus } from './loadStatus'
import type { Ref } from 'vue'
import type { IpcError } from '@/lib/ipc/types'

// One read, with its status and its error written the same way everywhere. `defineViewStore`
// is the whole store for the ones whose shape is just the triad; this is the half underneath
// it, for the four that carry more — the profile loads two things, the queue clears its
// mutation state first, the wiki skips a read it has already made, and the roll writes through
// the same read.
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
    error.value = asIpcError(e)
    status.value = LoadStatus.Failed
  }
}

// How the last write ended, for a store that says a failure rather than throwing it. `failed`
// is `null` for a store that keeps only the error.
const settle = (
  failed: Ref<boolean> | null,
  error: Ref<IpcError | null>,
  outcome: { failed: boolean; error: IpcError | null },
): void => {
  if (failed) failed.value = outcome.failed
  error.value = outcome.error
}

// Forgets what the last write said. When a store does it — as the write starts, or only once
// it lands — is that store's decision, which is why it is not part of `attempt`.
export const clearFailure = (
  failed: Ref<boolean> | null,
  error: Ref<IpcError | null>,
): void => settle(failed, error, { failed: false, error: null })

// One write whose failure is said, never thrown: clean once it lands, set with its reason when
// it is refused — the same foreign-error rule as `tracked` — and answered as whether it landed,
// for the caller that goes on only then. Nothing is touched while the write is in flight.
export const attempt = async (
  failed: Ref<boolean> | null,
  error: Ref<IpcError | null>,
  run: () => Promise<void>,
): Promise<boolean> => {
  try {
    await run()
    clearFailure(failed, error)
    return true
  } catch (e) {
    settle(failed, error, { failed: true, error: asIpcError(e) })
    return false
  }
}
