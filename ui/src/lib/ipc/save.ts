import { Command } from '../constants/commands'
import { call } from './transport'
import type { MarksMatrix, SaveSummary } from './types'

export const saveSummary = (): Promise<SaveSummary> => call(Command.SaveSummary)

export const completion = (): Promise<MarksMatrix> => call(Command.Completion)
