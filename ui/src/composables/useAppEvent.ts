import { onScopeDispose } from 'vue'
import { watchAppEvent } from '@/lib/window/appEvents'
import type { AppEvent } from '@/lib/window/appEvents'
import { subscription } from '@/lib/window/subscriptions'

/**
 * One app event for as long as the calling view lives (card #80, R8). The listener is detached
 * with the view even when the view goes before `listen` has answered, which the
 * `stop = await watchAppEvent(…)` in `onMounted` it replaces could not do.
 */
export const useAppEvent = (name: AppEvent, handler: () => void): void => {
  onScopeDispose(subscription(() => watchAppEvent(name, handler)))
}
