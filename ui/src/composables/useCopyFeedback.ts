import { onScopeDispose, ref } from 'vue'
import type { Ref } from 'vue'

// What a copy button says for a moment after it is clicked, then back to what it does. One timer at
// a time: a second click restarts the wait instead of being cut short by the first, and a page
// closed before the message is over leaves no timer behind.
export const CopyFeedback = {
  Idle: 'idle',
  Copied: 'copied',
  Failed: 'failed',
} as const
export type CopyFeedback = (typeof CopyFeedback)[keyof typeof CopyFeedback]

export const useCopyFeedback = (
  delay: number,
): { state: Ref<CopyFeedback>; report: (copied: boolean) => void } => {
  const state = ref<CopyFeedback>(CopyFeedback.Idle)
  let timer: ReturnType<typeof setTimeout> | undefined
  const clear = () => {
    if (timer !== undefined) clearTimeout(timer)
    timer = undefined
  }
  const report = (copied: boolean) => {
    clear()
    state.value = copied ? CopyFeedback.Copied : CopyFeedback.Failed
    timer = setTimeout(() => {
      state.value = CopyFeedback.Idle
      timer = undefined
    }, delay)
  }
  onScopeDispose(clear)
  return { state, report }
}
