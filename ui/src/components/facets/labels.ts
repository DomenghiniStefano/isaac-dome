import { dlcNames } from '@/components/wiki/dlcNames'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import { Dlc } from '@/lib/ipc/types'
import { OriginValue } from '@/lib/ipc/values'
import { oneOf } from '@/lib/oneOf'

export type Label = MessageKey<MessageSchema>

export type Translate = (key: Label, params?: Record<string, unknown>) => string

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

// The origin DLC's names are game data, the same as the wiki's editions; only "not stated" is
// ours to say. Both screens carry an origin facet over the same value set, so they word it the
// same way or they disagree with each other on the same row.
const originName: Record<OriginValue, string | null> = {
  [OriginValue.Rebirth]: dlcNames[Dlc.Rebirth],
  [OriginValue.Afterbirth]: dlcNames[Dlc.Afterbirth],
  [OriginValue.AfterbirthPlus]: dlcNames[Dlc.AfterbirthPlus],
  [OriginValue.Repentance]: dlcNames[Dlc.Repentance],
  [OriginValue.None]: null,
}

export const originLabel = (t: Translate, value: string): string => {
  const origin = oneOf(OriginValue, value)
  if (!origin) return value
  return originName[origin] ?? t('graph.originNone')
}
