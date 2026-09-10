import { Command } from '../constants/commands'
import { call } from './transport'
import type { ExtractionReport } from './types'

export const extractionReport = (): Promise<ExtractionReport> =>
  call(Command.ExtractionReport)
