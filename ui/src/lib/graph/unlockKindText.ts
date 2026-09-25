import type { Message, Translate } from '@/i18n/message'
import { TargetKind } from '@/lib/ipc/values'
import { oneOf } from '@/lib/oneOf'

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

// A kind facet's value in words: Unlock's "what it unlocks" and the Collection's kind carry the
// same set. A value outside it is shown as it came rather than dropped.
export const unlockKindLabel = (t: Translate, value: string): string => {
  const kind = oneOf(TargetKind, value)
  return kind ? t(unlockKindText[kind]) : value
}
