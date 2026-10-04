import type { RunView } from '@/lib/ipc/types'
import { runTime } from './runOrder'

// What a run's date *is*, and not only when: a session is dated by when it started and a launch
// by when its file was last written. Shown side by side they would read as the same thing, so
// the kind travels with the instant and the tooltip names it.
export const RunDateKind = {
  Started: 'started',
  Written: 'written',
  Undated: 'undated',
} as const
export type RunDateKind = (typeof RunDateKind)[keyof typeof RunDateKind]

export type RunDate =
  | { kind: typeof RunDateKind.Started; at: Date }
  | { kind: typeof RunDateKind.Written; at: Date }
  | { kind: typeof RunDateKind.Undated }

export const runDate = (run: RunView): RunDate => {
  const time = runTime(run)
  if (time === null) return { kind: RunDateKind.Undated }
  const at = new Date(time)
  return run.source.kind === 'session'
    ? { kind: RunDateKind.Started, at }
    : { kind: RunDateKind.Written, at }
}
