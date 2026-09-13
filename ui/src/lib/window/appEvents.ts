import { isTauri } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

// Events emitted by Rust when a command has just written. They carry no payload: they say
// "read again", and each window answers with the commands it already calls. Nothing new
// crosses the IPC boundary, so nothing new can leak across it — no path, no id, no `Debug`
// string in a channel that has no view-models.
//
// The four names are mirrored by hand in `crates/app/src/events.rs`, as the IPC types are:
// change one and change the other.
export const AppEvent = {
  ProfileChanged: 'profile-changed',
  SettingsChanged: 'settings-changed',
  PlanChanged: 'plan-changed',
  RunsChanged: 'runs-changed',
} as const
export type AppEvent = (typeof AppEvent)[keyof typeof AppEvent]

// One event, for a screen that cares about one. Degrades outside Tauri exactly like the
// plural below, and for the same measured reason.
export const watchAppEvent = async (
  name: AppEvent,
  handler: () => void,
): Promise<() => void> => {
  if (!isTauri()) return () => undefined
  return await listen(name, () => handler())
}

export const watchAppEvents = async (
  handlers: Record<AppEvent, () => void>,
): Promise<() => void> => {
  // Outside Tauri — `pnpm ui:dev` in a browser — there is nobody to be told by, and `listen`
  // does not degrade on its own: it reaches into internals that are not there and throws
  // `transformCallback of undefined` from inside a mounted hook, which takes the shell down
  // with it. Measured on 2026-09-13, on the development server.
  if (!isTauri()) return () => undefined
  const stops = await Promise.all(
    Object.values(AppEvent).map((name) => listen(name, () => handlers[name]())),
  )
  return () => {
    for (const stop of stops) stop()
  }
}
