import { Command } from '../constants/commands'
import { call } from './transport'
import type { Settings } from './types'

// The persisted settings. The scale comes back snapped to the ladder: what the backend
// answers is always a size the app was drawn at.
export const settings = (): Promise<Settings> => call(Command.Settings)

export const setScale = (percent: number): Promise<Settings> =>
  call(Command.SetScale, { percent })
