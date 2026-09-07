import { invoke } from '@tauri-apps/api/core'
import { Command } from '../constants/commands'
import type { MarksMatrix, SaveSummary } from './types'

export const saveSummary = (): Promise<SaveSummary> =>
  invoke(Command.SaveSummary)

export const completion = (): Promise<MarksMatrix> => invoke(Command.Completion)
