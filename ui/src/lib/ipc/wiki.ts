import { Command } from '../constants/commands'
import { call } from './transport'
import type { Entry, Target, WikiIndex } from './types'

// `null` means "the dataset doesn't know this target": not an error, a page
// that doesn't exist. `wikiUnavailable` (dataset not loaded at all) remains an `IpcError`.
export const wikiEntry = (target: Target): Promise<Entry | null> =>
  call(Command.WikiEntry, { target })

// Every page the dataset has, once per window; a dataset that didn't load is an empty index
// whose `info` says why, not a rejection.
export const wikiIndex = (): Promise<WikiIndex> => call(Command.WikiIndex)
