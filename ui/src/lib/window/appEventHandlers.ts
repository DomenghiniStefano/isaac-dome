import { AppEvent } from './appEvents'

// What each window does when Rust says something was written. The stores arrive as arguments —
// the smallest shape each one is asked for — so the registry can be tested without a Pinia or
// a Tauri: `App.vue` hands it the real ones.
export interface AppEventStores {
  profile: { stopPicking: () => void; load: () => Promise<void> }
  settings: { load: () => Promise<void> }
  queue: { load: () => Promise<void> }
  runs: { refresh: () => Promise<void> }
  live: { refresh: () => Promise<void> }
}

// A window never learns of a write it did not make, so it is told. The profile carries through
// to every screen that reads the save (`useOnActiveProfile`); the queue store is read again
// wherever it is mounted.
export const appEventHandlers = (
  s: AppEventStores,
): Record<AppEvent, () => void> => ({
  // One answer settles every window: a picker this window opened on purpose closes when
  // another window chooses, because the settled profile is the app's and not the window's.
  [AppEvent.ProfileChanged]: () => {
    s.profile.stopPicking()
    void s.profile.load()
  },
  [AppEvent.SettingsChanged]: () => void s.settings.load(),
  [AppEvent.PlanChanged]: () => void s.queue.load(),
  // The run archive fills itself in the background (card #80, item 06: this was a no-op under
  // a comment calling Live and Runs placeholders). `refresh` reads again only a view some
  // screen has loaded, so a window with neither screen open reads nothing.
  [AppEvent.RunsChanged]: () => {
    void s.runs.refresh()
    void s.live.refresh()
  },
  // The Roll screen listens for this one itself (`RollScreen.vue`'s own `watchAppEvent`), so
  // that two windows agree through a draw made in either. Listed here so that every event is
  // accounted for in this one registry.
  [AppEvent.RollChanged]: () => undefined,
  // The Updates screen listens for this one itself, like Roll: a download tells every window
  // a hundred times, and a window with that screen closed has nothing to draw with it.
  [AppEvent.UpdateChanged]: () => undefined,
})
