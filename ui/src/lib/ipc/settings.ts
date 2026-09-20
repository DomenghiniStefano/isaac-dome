import { Command } from '../constants/commands'
import { call } from './transport'
import type { AutostartView, Settings } from './types'

// The persisted settings. The scale comes back snapped to the ladder: what the backend
// answers is always a size the app was drawn at.
export const settings = (): Promise<Settings> => call(Command.Settings)

export const setScale = (percent: number): Promise<Settings> =>
  call(Command.SetScale, { percent })

// The two switches of the Background screen. Both answer the settings as they now are, the
// same shape `setScale` has, so the store never has to guess what the backend kept.
export const setStayInBackground = (stay: boolean): Promise<Settings> =>
  call(Command.SetStayInBackground, { stay })

export const setResumeTabs = (resume: boolean): Promise<Settings> =>
  call(Command.SetResumeTabs, { resume })

// Whether the app asks GitHub for a newer version when it starts. Off means the request does
// not happen — not that it happens quietly — so this is the one switch here that decides
// whether anything leaves the machine at all.
export const setAutoUpdate = (on: boolean): Promise<Settings> =>
  call(Command.SetAutoUpdate, { on })

// Starting with Windows. **The registry is the truth**, so both of these answer what it says
// and never what was asked: `Settings` carries nothing about this and there is nothing to keep
// in step with it.
export const autostart = (): Promise<AutostartView> => call(Command.Autostart)

export const setAutostart = (on: boolean): Promise<AutostartView> =>
  call(Command.SetAutostart, { on })
