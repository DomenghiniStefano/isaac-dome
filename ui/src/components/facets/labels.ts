import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'

export type Label = MessageKey<MessageSchema>

// The words a faceted list needs, passed as message keys and never as a prefix to build them
// from. `t(`${prefix}.rows`)` would be a key neither the i18n types nor `pnpm scan` can see: a
// screen naming a key that does not exist would ship, and show the key.

export interface ToolbarLabels {
  // The noun for a row of this list: "obiettivi", "oggetti".
  rows: Label
  search: Label
  sortBy: Label
  activeFilters: Label
}

export interface DrawerLabels {
  facets: Label
  activeFilters: Label
  noFilters: Label
  reset: Label
}
