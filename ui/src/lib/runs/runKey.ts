import { assertNever } from '@/lib/assertNever'
import type { RunSource, RunView } from '@/lib/ipc/types'

// A source's identity, written once: the key below and the diary's order both read it. A launch
// of `log.txt` has no name, which is why the store keys it `NULL`: a past launch is told apart by
// the row id it carries, and the live one is the only one of its kind. A session could be called
// anything at all, so the kind is always part of it.
export const sourcePart = (source: RunSource): string => {
  switch (source.kind) {
    case 'live':
      return 'live:'
    case 'launch':
      return `launch:${source.id}`
    case 'session':
      return `session:${source.name}`
    default:
      return assertNever(source)
  }
}

// A run is `(source, ordinal)`: that pair is its identity in the archive and therefore the only
// thing that can be written down about *which* run.
export const runKey = (run: RunView): string =>
  `${sourcePart(run.source)}#${run.ordinal}`
