import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import { assertNever } from '@/lib/assertNever'
import type { LockView, UnlockNode } from '@/lib/ipc/types'
import { pageLocation } from '@/lib/wiki/category'
import type { TabLocation } from '@/router/routeTable'
import { RequirementKind, missingGroups } from './nodeState'

type Translate = (
  key: MessageKey<MessageSchema>,
  params?: Record<string, unknown>,
) => string

// The menu behind a badge, as data: one group per kind, one entry per thing in the way, and
// for each the place that says how *it* is unlocked. Pure, so the model is what gets tested
// and the component that draws it holds no decision (`DESIGN-BRIEF.md` §7.1).
export interface WhyEntry {
  key: string
  name: string
  /** `null` = nowhere to go: the entry shows, disabled. Never a link that leads nowhere. */
  location: TabLocation | null
}

export interface WhyGroup {
  label: MessageKey<MessageSchema>
  entries: WhyEntry[]
}

const kindLabel: Record<RequirementKind, MessageKey<MessageSchema>> = {
  [RequirementKind.Character]: 'graph.why.character',
  [RequirementKind.Boss]: 'graph.why.boss',
  [RequirementKind.Challenge]: 'graph.why.challenge',
  [RequirementKind.Item]: 'graph.why.item',
  [RequirementKind.Gate]: 'graph.why.gate',
  [RequirementKind.Mark]: 'graph.why.mark',
  [RequirementKind.Counter]: 'graph.why.counter',
  [RequirementKind.Threshold]: 'graph.why.threshold',
  [RequirementKind.Unknown]: 'graph.why.unknown',
}

// No groups means no menu, whatever the node's state: a badge with nothing to say stays the
// inert badge it is, and a `partial` node keeps its why like any other.
export const nodeWhy = (node: UnlockNode, t: Translate): WhyGroup[] =>
  missingGroups(node, t).map((group) => ({
    label: kindLabel[group.kind],
    entries: group.entries,
  }))

// The Collection's lock: one group, one entry — the achievement that opens the item. `free`
// has nothing to say, and says nothing.
export const lockWhy = (lock: LockView, t: Translate): WhyGroup[] => {
  switch (lock.kind) {
    case 'free':
      return []
    case 'unlocked':
    case 'locked':
    case 'unknown':
      return [
        {
          label: 'collection.lockedBy',
          entries: [
            {
              key: `achievement-${lock.achievement}`,
              name:
                lock.text ??
                `${t('collection.achievement')} ${lock.achievement}`,
              location: lock.page ? pageLocation(lock.page) : null,
            },
          ],
        },
      ]
    default:
      return assertNever(lock)
  }
}
