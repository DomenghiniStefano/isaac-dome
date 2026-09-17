import { Command } from '../constants/commands'
import { call } from './transport'
import type { SetupState } from './types'

export const setupState = (): Promise<SetupState> => call(Command.SetupState)

export const selectProfile = (id: string): Promise<SetupState> =>
  call(Command.SelectProfile, { id })

// B14: the folder is asked for in Rust and never crosses back — what returns is the state it
// produced, with its path hints already redacted.
export const chooseGameFolder = (): Promise<SetupState> =>
  call(Command.ChooseGameFolder)

export const chooseSavesFolder = (): Promise<SetupState> =>
  call(Command.ChooseSavesFolder)
