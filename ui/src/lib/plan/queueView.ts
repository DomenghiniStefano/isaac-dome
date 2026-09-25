import { sortBy } from 'lodash-es'
import type { QueueView } from '@/lib/ipc/types'
import { queuedIds } from './queueRows'

// What the queue holds, as a value that ignores its order: the suggestions beside it leave out
// what is queued, so they depend on the set and not on where each row sits. `null` is a queue
// that has not been read yet.
export const queueMembership = (view: QueueView | null): string | null =>
  view === null ? null : sortBy([...queuedIds(view)]).join(',')

// Whether the suggestions have to be asked again: only between two reads that differ. A queue
// arriving with the profile arrives with the graph beside it, and neither asks.
export const membershipChanged = (
  now: string | null,
  before: string | null,
): boolean => now !== null && before !== null && now !== before

// The queue card needs a queue that could be read: no database, an unreadable document and no
// catalog each say so in an alert instead of an empty list.
export const queueReadable = (view: QueueView | null): boolean =>
  view !== null &&
  view.storeAvailable &&
  !view.diagnostics.some(
    (d) => d.kind === 'unreadable' || d.kind === 'noCatalog',
  )
