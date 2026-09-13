import { DefaultScale, snapPercent } from '@/lib/scale/steps'
import type { Settings } from '../types'

// Development only: the settings the app would have persisted, and the session it would have
// stored. There is no file and no database on the development server, so these live for the
// page's life — enough to look at the slider and see the whole interface follow it, and to
// reload the page and find the tabs where they were.
let scale = DefaultScale
let stayInBackground = true
let resumeTabs = true
let session: string | null = null

export const resetSettingsFixture = (): void => {
  scale = DefaultScale
  stayInBackground = true
  resumeTabs = true
  session = null
}

export const settingsAnswer = (): Settings => ({
  activeProfileId: null,
  scale,
  stayInBackground,
  resumeTabs,
  backgroundNoticeShown: false,
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
