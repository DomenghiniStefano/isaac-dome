import { Command } from '../constants/commands'
import { call } from './transport'
import type { RunsView } from './types'

// The run archive: every source the app has read, with the runs folded out of it. A read, and
// only a read — the archive fills itself in the background and says so with `runs-changed`.
export const runs = (): Promise<RunsView> => call(Command.Runs)
