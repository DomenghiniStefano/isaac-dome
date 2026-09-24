import { DefaultScale, snapPercent } from '@/lib/scale/steps'
import type { AutostartView, IpcError, Settings } from '../types'
import { AutostartFailure, AutostartReason } from '../types'

// Development only: the settings the app would have persisted, and the session it would have
// stored. There is no file and no database on the development server, so these live for the
// page's life — enough to look at the slider and see the whole interface follow it, and to
// reload the page and find the tabs where they were.
let scale = DefaultScale
let stayInBackground = true
let resumeTabs = true
let autoUpdate = true
let session: string | null = null

export const resetSettingsFixture = (): void => {
  scale = DefaultScale
  stayInBackground = true
  resumeTabs = true
  autoUpdate = true
  session = null
  autostart = null
}

export const settingsAnswer = (): Settings => ({
  activeProfileId: null,
  scale,
  stayInBackground,
  resumeTabs,
  backgroundNoticeShown: false,
  autoUpdate,
})

// The backend snaps before it writes, so the fixture does too: what comes back is always a
// size the app was drawn at.
export const setScaleAnswer = (percent: number): Settings => {
  scale = snapPercent(percent)
  return settingsAnswer()
}

export const setStayInBackgroundAnswer = (stay: boolean): Settings => {
  stayInBackground = stay
  return settingsAnswer()
}

// Off clears what was stored, as the backend does: one rule, not two.
export const setResumeTabsAnswer = (resume: boolean): Settings => {
  resumeTabs = resume
  if (!resume) session = null
  return settingsAnswer()
}

// `null` when the setting is off, whatever is held — again the backend's own rule.
export const sessionAnswer = (): string | null => (resumeTabs ? session : null)

export const setSessionAnswer = (document: string | null): void => {
  if (resumeTabs) session = document
}

// Starting with Windows. `null` is **no registry at all**, which is what the development server
// has and what a development build of the app reports: the plugin is not registered there, on
// purpose, because `current_exe()` in development is `target\debug\app.exe`. A test hands it a
// registry to reach the two cases that only exist where one answers.
let autostart: { enabled: boolean; refuses: AutostartFailure | null } | null =
  null

export const useAutostartRegistry = (
  registry: { enabled: boolean; refuses: AutostartFailure | null } | null,
): void => {
  autostart = registry
}

export const autostartAnswer = (): AutostartView =>
  autostart === null
    ? { enabled: false, unavailable: AutostartReason.NotSupported }
    : { enabled: autostart.enabled, unavailable: null }

// Writes, reads back, answers the read — and throws where the backend does: the registry was
// asked and it still says something else.
export const setAutostartAnswer = (on: boolean): AutostartView => {
  if (autostart === null) return autostartAnswer()
  // `refuses: null` is a registry that takes writes; `WriteIgnored` is one that takes the write
  // and goes on saying the opposite — the shape of an entry switched off in the Startup tab.
  if (autostart.refuses === null) autostart.enabled = on
  if (autostart.enabled !== on) {
    const refused: IpcError = {
      kind: 'autostartNotWritable',
      reason: autostart.refuses ?? AutostartFailure.WriteRefused,
    }
    throw refused
  }
  return autostartAnswer()
}

// The auto-update switch, which the fixture carried and nothing could write (card #80,
// item 10).
export const setAutoUpdateAnswer = (on: boolean): Settings => {
  autoUpdate = on
  return settingsAnswer()
}
