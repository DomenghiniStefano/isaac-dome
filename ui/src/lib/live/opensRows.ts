import type { Message, Translate } from '@/i18n/message'
import {
  refCondition,
  refIcon,
  refNumber,
  refTarget,
  refText,
} from '@/lib/graph/achievementNode'
import { columnName } from '@/lib/graph/nodeState'
import { SecondLevelView } from '@/lib/ipc/types'
import type { LiveOpen, Target } from '@/lib/ipc/types'

// One row of what the run could open: an achievement, the cell of the matrix that opens it,
// and how much it opens in turn.
export interface OpenRow {
  key: string
  /** The achievement's text, or its slot number when the catalog cannot name it. */
  name: string
  target: Target | null
  condition: string | null
  iconUrl: string | null
  /** The cell, in the column's own words. */
  cell: string
  fanOut: number
}

// The second level is said only where it is a different thing to go and do, and in the
// column's own word (B66): Ultra Greedier in Greed, hard elsewhere. Which is which arrives
// from Rust in `secondLevel`, `null` at the base level.
const secondLevelWord: Record<SecondLevelView, Message> = {
  [SecondLevelView.Hard]: 'live.secondLevel.hard',
  [SecondLevelView.UltraGreedier]: 'live.secondLevel.ultraGreedier',
}

export const cellName = (t: Translate, open: LiveOpen): string =>
  open.secondLevel === null
    ? columnName[open.column]
    : `${columnName[open.column]} · ${t(secondLevelWord[open.secondLevel])}`

// Everything the run could open, one row per achievement of every cell.
export const opensRows = (opens: LiveOpen[], t: Translate): OpenRow[] =>
  opens.flatMap((open) =>
    open.achievements.map(({ achievement, fanOut }) => {
      const name = refText(achievement) ?? String(refNumber(achievement))
      return {
        key: `${open.character}-${open.column}-${name}`,
        name,
        target: refTarget(achievement),
        condition: refCondition(achievement),
        iconUrl: refIcon(achievement),
        cell: cellName(t, open),
        fanOut,
      }
    }),
  )
