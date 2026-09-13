import { NodeState, nodeState } from '@/lib/graph/nodeState'
import type { QueueRow, QueueView } from '@/lib/ipc/types'

// What you had already decided, on the page that suggests. It filters and never re-sorts:
// the queue's order is the user's own, and a landing page that reordered it would be
// contradicting the screen that owns it.
//
// A store that would not open answers nothing rather than an empty list: "you have planned
// nothing" and "we could not look" are different sentences, and the Plan is where the second
// one is said (spec §4.3).
export const planNow = (view: QueueView | null, limit: number): QueueRow[] =>
  view === null || !view.storeAvailable
    ? []
    : view.rows
        // `nodeState` answers `Done` before it answers `Now`, so a row already done is out
        // without a second check.
        .filter((row) => nodeState(row.node) === NodeState.Now)
        .slice(0, limit)
