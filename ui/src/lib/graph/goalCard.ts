import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import type { UnlockNode } from '@/lib/ipc/types'
import { pageLocation } from '@/lib/wiki/category'
import type { TabLocation } from '@/router/routeTable'
import { targetName } from './characterName'

type Translate = (
  key: MessageKey<MessageSchema>,
  params?: Record<string, unknown>,
) => string

// More than one thing out of one achievement is rare and real — a challenge's rewards — so
// the names are listed, never counted.
const Separator = ' · '

// A row of the landing page, top to bottom: what you get, how you get it, how much it is
// worth, and where to read more. The component draws it and decides nothing
// (`DESIGN-BRIEF.md` §7.1), which is what makes the decisions testable.
export interface GoalCardModel {
  headline: string
  /** The game's own `unlock_condition`. `null` when the file states none. */
  condition: string | null
  art: string | null
  /** The raw count: the sentence around it belongs to the component and its messages. */
  fanOut: number
  /** The achievement's page — the detail. `null` for a slot the catalog cannot name. */
  location: TabLocation | null
}

export const goalCard = (node: UnlockNode, t: Translate): GoalCardModel => {
  const a = node.achievement
  const known = a.kind === 'known'
  // What you get comes first, and the achievement's text is only the fallback: the file
  // names a Tainted character by its base form, which is where the owner's confusion with
  // the reference profile's five rows started (`docs/BACKLOG.md` B28, B32).
  const unlocked = node.unlocks.map((u) => targetName(t, u)).join(Separator)
  const fallback = known
    ? a.text
    : `${t('graph.unknownAchievement')} · ${t('graph.slot')} ${a.slot}`
  return {
    headline: unlocked === '' ? fallback : unlocked,
    condition: known ? a.condition : null,
    art: known ? a.iconUrl : null,
    fanOut: node.graph.fanOut,
    location: known ? pageLocation({ kind: 'achievement', id: a.id }) : null,
  }
}
