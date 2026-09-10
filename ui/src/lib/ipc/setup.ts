import { Command } from '../constants/commands'
import { call } from './transport'
import type { SetupState } from './types'

export const setupState = (): Promise<SetupState> => call(Command.SetupState)

export const selectProfile = (id: string): Promise<SetupState> =>
  call(Command.SelectProfile, { id })
