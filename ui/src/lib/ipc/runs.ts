import { Command } from '../constants/commands'
import { call } from './transport'
import type { LiveView, RunsView } from './types'

// The run archive: every source the app has read, with the runs folded out of it. A read, and
// only a read — the archive fills itself in the background and says so with `runs-changed`.
export const runs = (): Promise<RunsView> => call(Command.Runs)

// What the run being watched would open: the archive and the graph joined in one read, because
// two commands cannot promise that two answers describe the same profile (N8).
export const live = (): Promise<LiveView> => call(Command.Live)
