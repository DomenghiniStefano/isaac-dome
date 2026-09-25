import type { Message } from '@/i18n/message'
import { TargetKind } from '@/lib/ipc/values'

// What a node unlocks, in words. A record over the whole set: a kind with no words fails to
// compile.
export const unlockKindText: Record<TargetKind, Message> = {
  [TargetKind.Passive]: 'graph.kinds.passive',
  [TargetKind.Active]: 'graph.kinds.active',
  [TargetKind.Familiar]: 'graph.kinds.familiar',
  [TargetKind.Trinket]: 'graph.kinds.trinket',
  [TargetKind.Character]: 'graph.kinds.character',
  [TargetKind.Boss]: 'graph.kinds.boss',
  [TargetKind.Challenge]: 'graph.kinds.challenge',
  [TargetKind.Nothing]: 'graph.kinds.nothing',
}
