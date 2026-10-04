import { assertNever } from '@/lib/assertNever'
import type { RunSource, RunView } from '@/lib/ipc/types'

// A source's identity, written once: the key below and the diary's order both read it. A launch
// of `log.txt` has no name, which is why the store keys it `NULL`, so it is told apart by its row
// id — and the latest launch, `live`, carries the same id it keeps when a newer one arrives, so
// its runs keep their keys. A session could be called anything at all, so the kind is always
// part of the key.
export const sourcePart = (source: RunSource): string => {
  switch (source.kind) {
    case 'live':
    case 'launch':
      return `log:${source.id}`
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
