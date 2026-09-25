import { once } from 'lodash-es'

// What `listen`, `onFocusChanged` and every other Tauri subscription hand back, a beat late.
export type Stop = () => void

interface Begun {
  started: Promise<Stop>
  stop: Stop
}

// The stop is chained onto the start, so it lands whenever the start does, and it lands once.
// A start that failed has nothing to detach.
const begin = (start: () => Promise<Stop>): Begun => {
  const started = start()
  const stop = once(() => {
    void started.then(
      (unlisten) => unlisten(),
      () => undefined,
    )
  })
  return { started, stop }
}

/**
 * A subscription that can be stopped before it has started. The pattern it replaces —
 * `stop = await listen(…)` in `onMounted`, `stop?.()` in `onUnmounted` — leaks whenever the
 * view unmounts while `listen` is still resolving: `stop` is still unset when the teardown
 * runs, and the listener outlives the view it was for.
 */
export const subscription = (start: () => Promise<Stop>): Stop =>
  begin(start).stop

export interface Subscriptions {
  /** Resolves once the subscription has started, for a caller whose next step needs it. */
  add: (start: () => Promise<Stop>) => Promise<void>
  stop: Stop
}

/**
 * Several subscriptions made one after another, across awaits, and stopped together. One added
 * after the set was stopped — the window closed between two of the awaits — is stopped the
 * moment it starts, instead of being kept by a set nobody will stop again.
 */
export const subscriptions = (): Subscriptions => {
  const held: Stop[] = []
  const state = { stopped: false }
  const add = async (start: () => Promise<Stop>): Promise<void> => {
    const { started, stop } = begin(start)
    if (state.stopped) stop()
    else held.push(stop)
    await started
  }
  const stop = (): void => {
    state.stopped = true
    held.splice(0).forEach((s) => s())
  }
  return { add, stop }
}
