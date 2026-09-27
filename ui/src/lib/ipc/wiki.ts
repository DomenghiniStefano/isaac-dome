import { Command } from '../constants/commands'
import { call } from './transport'
import type {
  Entry,
  PoolMembershipView,
  Target,
  WikiIndex,
  WikiProgress,
} from './types'

// `null` means "the dataset doesn't know this target": not an error, a page
// that doesn't exist. `wikiUnavailable` (dataset not loaded at all) remains an `IpcError`.
export const wikiEntry = (target: Target): Promise<Entry | null> =>
  call(Command.WikiEntry, { target })

// The pools the installed game lists an item in (design decision 4): `null` without the
// game, or for a target with no pools concept at all — a trinket included, since
// `itempools.xml` never covers those (`ipc::item_pools`'s doc comment has the measurement).
export const wikiItemPools = (
  target: Target,
): Promise<PoolMembershipView[] | null> =>
  call(Command.WikiItemPools, { target })

// Every page the dataset has, once per window; a dataset that didn't load is an empty index
// whose `info` says why, not a rejection.
export const wikiIndex = (): Promise<WikiIndex> => call(Command.WikiIndex)

// The save's state for every wiki page that has one (design decision 6). `null` when no save
// is chosen — never an error: the rest of the wiki still shows.
export const wikiProgress = (): Promise<WikiProgress | null> =>
  call(Command.WikiProgress)
