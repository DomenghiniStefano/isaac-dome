import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import { StepsBasis } from '@/lib/ipc/types'

// Why these rows are here. A record over the whole set, so a basis added in Rust is a build
// error here and not a heading that quietly comes out blank.
export const sectionTitle: Record<StepsBasis, MessageKey<MessageSchema>> = {
  [StepsBasis.FanOut]: 'goals.fanOut',
  [StepsBasis.Closeness]: 'goals.closeness',
}
