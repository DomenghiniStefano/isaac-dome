import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import { UnlockKind } from '@/lib/graph/unlockFilter'

// What a node unlocks, in words. A record over the whole set: a kind with no words fails to
// compile.
export const unlockKindText: Record<UnlockKind, MessageKey<MessageSchema>> = {
  [UnlockKind.Passive]: 'graph.kinds.passive',
  [UnlockKind.Active]: 'graph.kinds.active',
  [UnlockKind.Familiar]: 'graph.kinds.familiar',
  [UnlockKind.Trinket]: 'graph.kinds.trinket',
  [UnlockKind.Character]: 'graph.kinds.character',
  [UnlockKind.Boss]: 'graph.kinds.boss',
  [UnlockKind.Challenge]: 'graph.kinds.challenge',
  [UnlockKind.Nothing]: 'graph.kinds.nothing',
}
