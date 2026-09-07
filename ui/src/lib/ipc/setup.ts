import { invoke } from '@tauri-apps/api/core'
import { Command } from '../constants/commands'
import type { SetupState } from './types'

export const setupState = (): Promise<SetupState> => invoke(Command.SetupState)

export const selectProfile = (id: string): Promise<SetupState> =>
  invoke(Command.SelectProfile, { id })
