import type { Translate } from '@/i18n/message'
import type { UnlockNode, UnlockTarget } from '@/lib/ipc/types'
import { pageLocationOf } from '@/lib/wiki/category'
import type { TabLocation } from '@/router/routeTable'
import { targetName } from './characterName'

// What an achievement gives you, with somewhere to read about it. The sibling of
// `missingGroups`: that one says what is in the way, this one what comes out — and both
// answer `null` rather than link to a page the dataset does not have (`docs/BACKLOG.md` B35).
export interface UnlockEntry {
  key: string
  name: string
  iconUrl: string | null
  location: TabLocation | null
}

// Only an item carries a drawing on the contract: a character, a boss and a challenge are
// named, not pictured, in this row.
const iconOf = (target: UnlockTarget): string | null =>
  target.kind === 'item' ? target.iconUrl : null

export const nodeUnlocks = (node: UnlockNode, t: Translate): UnlockEntry[] =>
  node.unlocks.map((target) => ({
    // Kind and id together: a boss and a character can share a number.
    key: `${target.kind}-${target.id}`,
    name: targetName(t, target),
    iconUrl: iconOf(target),
    location: pageLocationOf(target.page),
  }))
