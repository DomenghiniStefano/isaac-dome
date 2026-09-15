import type { RunView } from '@/lib/ipc/types'

// A run is `(source, ordinal)`: that pair is its identity in the archive and therefore the only
// thing that can be written down about *which* run. The kind is part of the key — a launch of
// `log.txt` has no name, which is why the store keys it `NULL`, and a session could be called
// anything at all.
export const runKey = (run: RunView): string =>
  `${run.source.kind === 'live' ? 'live:' : `session:${run.source.name}`}#${run.ordinal}`
