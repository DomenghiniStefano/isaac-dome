import type { Translate } from '@/i18n/message'
import { targetName } from '@/lib/graph/characterName'
import { NodeState, nodeState } from '@/lib/graph/nodeState'
import {
  refCondition,
  refIcon,
  refNumber,
  refTarget,
  refText,
} from '@/lib/graph/achievementNode'
import type { AchievementRef, UnlockNode } from '@/lib/ipc/types'
import { pageLocation } from '@/lib/wiki/category'
import type { TabLocation } from '@/router/routeTable'

// More than one thing out of one achievement is rare and real — a challenge's rewards — so
// the names are listed, never counted.
const Separator = ' · '

// Everything a row decides, for both panes of the screen. The components draw this and decide
// nothing (`DESIGN-BRIEF.md` §7.1), which is what makes the decisions testable.
//
// It is also the end of a disagreement: the queue used to call a row by the achievement's own
// sentence while the recommendations called it by what it unlocks. Two names for the same row
// on two halves of one question, and only one of them had a reason.
export interface RowModel {
  /** What you get. The achievement's own sentence only when it unlocks nothing catalogued. */
  text: string
  /** The game's own `unlock_condition`. `null` when the file states none. */
  condition: string | null
  art: string | null
  /** The raw count: the sentence around it belongs to the component and its messages. */
  fanOut: number
  /** Amber, and only that: whether it can be played tonight. */
  playable: boolean
  /** The same reading as a value, so the colour is never the only carrier (spec §4.1). */
  state: NodeState
  /** The achievement's page. `null` for a slot the catalog cannot name. */
  location: TabLocation | null
}

// What you get comes first, and the achievement's text is only the fallback: the file names
// a Tainted character by its base form (`docs/BACKLOG.md` B28, B32).
const rowText = (node: UnlockNode, t: Translate): string => {
  const unlocked = node.unlocks.map((u) => targetName(t, u)).join(Separator)
  if (unlocked !== '') return unlocked
  return (
    refText(node.achievement) ??
    t('graph.unknownAchievementInSlot', { slot: refNumber(node.achievement) })
  )
}

const pageOf = (a: AchievementRef): TabLocation | null => {
  const target = refTarget(a)
  return target === null ? null : pageLocation(target)
}

export const rowModel = (node: UnlockNode, t: Translate): RowModel => {
  const a = node.achievement
  const state = nodeState(node)
  return {
    text: rowText(node, t),
    condition: refCondition(a),
    art: refIcon(a),
    fanOut: node.graph.fanOut,
    playable: state === NodeState.Now,
    state,
    location: pageOf(a),
  }
}
