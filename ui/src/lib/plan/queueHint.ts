import type { Translate } from '@/i18n/message'
import { knownText } from '@/lib/graph/achievementNode'
import type { QueueRow } from '@/lib/ipc/types'
import { rowId, stoppedUnder } from './queueRows'

// The last move that landed, as the queue remembers it.
export interface LastMove {
  achievement: number
  after: number | null
}

// What the queue's band says: how to drag, what a drop will do, or where the last move stopped
// and why. The row it stopped under is named by its text, or by its number when the catalog has
// none.
export const queueHint = (
  t: Translate,
  dragging: boolean,
  rows: QueueRow[],
  last: LastMove | null,
): string => {
  if (dragging) return t('plan.hint.dragging')
  const wall = last ? stoppedUnder(rows, last.achievement, last.after) : null
  if (!wall) return t('plan.hint.idle')
  const name =
    knownText(wall.node) ?? t('plan.achievementNumbered', { id: rowId(wall) })
  return t('plan.hint.stoppedUnder', { name })
}
