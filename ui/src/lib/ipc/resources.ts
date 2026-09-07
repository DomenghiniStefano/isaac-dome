import { invoke } from '@tauri-apps/api/core'
import { Command } from '../constants/commands'
import type { ExtractionReport } from './types'

export const extractionReport = (): Promise<ExtractionReport> =>
  invoke(Command.ExtractionReport)
