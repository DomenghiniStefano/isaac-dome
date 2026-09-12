import { Command } from '../constants/commands'
import { call } from './transport'
import type { SearchView } from './types'

// How many hits each caller asks for. The palette shows five rows per group and needs enough
// to fill four of them; the screen asks for everything up to a line that says it stopped.
export const SearchLimit = { Palette: 40, Screen: 300 } as const

// One query over the wiki's text and the catalog's names. No profile is a diagnostic inside
// the answer, never a rejection: search works before a save is chosen.
export const search = (query: string, limit: number): Promise<SearchView> =>
  call(Command.Search, { query, limit })
