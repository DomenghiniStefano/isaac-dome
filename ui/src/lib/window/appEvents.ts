import { listen } from '@tauri-apps/api/event'

// Events emitted by Rust when a command has just written. They carry no payload: they say
// "read again", and each window answers with the commands it already calls. Nothing new
// crosses the IPC boundary, so nothing new can leak across it — no path, no id, no `Debug`
// string in a channel that has no view-models.
//
// The three names are mirrored by hand in `crates/app/src/events.rs`, as the IPC types are:
// change one and change the other.
export const AppEvent = {
  ProfileChanged: 'profile-changed',
  SettingsChanged: 'settings-changed',
  PlanChanged: 'plan-changed',
} as const
export type AppEvent = (typeof AppEvent)[keyof typeof AppEvent]

export const watchAppEvents = async (
  handlers: Record<AppEvent, () => void>,
): Promise<() => void> => {
  const stops = await Promise.all(
    Object.values(AppEvent).map((name) => listen(name, () => handlers[name]())),
  )
  return () => {
    for (const stop of stops) stop()
  }
}
