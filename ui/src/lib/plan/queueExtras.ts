import type { QueueRow } from '@/lib/ipc/types'
import { knownText } from '@/lib/graph/achievementNode'
import { originRows } from './queueRows'

/** A wish this row serves. `text` is `null` when that wish is no longer in the queue. */
export interface Serves {
  id: number
  text: string | null
}

// What a queue row has and a recommendation does not: you asked for it, it serves somebody
// else's wish, and some of what it still needs is outside the queue. Everything a row says
// about *itself* is `rowModel`'s; this is only what the queue around it adds.
export interface QueueExtras {
  wanted: boolean
  serves: Serves[]
  stepsNotQueued: number
}

// No `t`: the id fallback is a number, and the words around it belong to the component and
// its messages. Every name here is the game's, already written in the view.
export const queueExtras = (row: QueueRow, rows: QueueRow[]): QueueExtras => ({
  wanted: row.wanted,
  serves: originRows(row, rows).map((origin) => ({
    id: origin.id,
    text: origin.row ? knownText(origin.row.node) : null,
  })),
  stepsNotQueued: row.stepsNotQueued,
})
