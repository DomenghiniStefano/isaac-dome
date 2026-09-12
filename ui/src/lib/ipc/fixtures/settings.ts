import { DefaultScale, snapPercent } from '@/lib/scale/steps'
import type { Settings } from '../types'

// Development only: the settings the app would have persisted. There is no file on the
// development server, so the value lives for the page's life — enough to look at the slider
// and see the whole interface follow it.
let scale = DefaultScale

export const resetSettingsFixture = (): void => {
  scale = DefaultScale
}

export const settingsAnswer = (): Settings => ({
  activeProfileId: null,
  scale,
})

// The backend snaps before it writes, so the fixture does too: what comes back is always a
// size the app was drawn at.
export const setScaleAnswer = (percent: number): Settings => {
  scale = snapPercent(percent)
  return settingsAnswer()
}
