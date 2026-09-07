import { invoke } from '@tauri-apps/api/core'
import { Command } from '../constants/commands'
import type { Entry, Target } from './types'

// `null` means "the dataset doesn't know this target": not an error, a page
// that doesn't exist. `wikiUnavailable` (dataset not loaded at all) remains an `IpcError`.
export const wikiEntry = (target: Target): Promise<Entry | null> =>
  invoke(Command.WikiEntry, { target })
