import { filter, isEqual, maxBy, sortBy } from 'lodash-es'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import { assertNever } from '@/lib/assertNever'
import { StatusView } from '@/lib/ipc/types'
import type { DeckView, RollRowView, SelectionView } from '@/lib/ipc/types'
import type { FacetOption } from '@/lib/facets/facetOptions'

type Message = MessageKey<MessageSchema>

// Every judgment the Roll screen makes, so a test can see it and no component computes it
// inline.

export const statusText = (status: StatusView): Message => {
  switch (status) {
    case StatusView.Missing:
      return 'roll.status.missing'
    case StatusView.Taken:
      return 'roll.status.taken'
    case StatusView.Unreadable:
      return 'roll.status.unreadable'
    default:
      return assertNever(status)
  }
}

// Which of the four exclusions is worth explaining when the deck is empty. Fieldless: a bare
// string, like every other value on this boundary — the values are `DeckView`'s own field
// names, so a reason indexes the deck it came from without a second mapping.
export const EmptyDeckReason = {
  Taken: 'taken',
  Unreadable: 'unreadable',
  Locked: 'locked',
  Filtered: 'filtered',
} as const
export type EmptyDeckReason =
  (typeof EmptyDeckReason)[keyof typeof EmptyDeckReason]

// `null` while the deck holds something, or while the space itself held nothing to exclude:
// an empty space is not an empty deck with a reason, and inventing one would put a sentence
// on the screen the numbers do not support. Otherwise, the exclusion that took the most.
export const emptyDeckReason = (deck: DeckView): EmptyDeckReason | null => {
  if (deck.size > 0) return null
  const exclusions = [
    { reason: EmptyDeckReason.Taken, count: deck.taken },
    { reason: EmptyDeckReason.Unreadable, count: deck.unreadable },
    { reason: EmptyDeckReason.Locked, count: deck.locked },
    { reason: EmptyDeckReason.Filtered, count: deck.filtered },
  ]
  const largest = maxBy(exclusions, (exclusion) => exclusion.count)
  return largest && largest.count > 0 ? largest.reason : null
}

// The picked ids as the strings `MultiSelect` speaks: it never sees a row, only its own value.
export const selectionOf = (rows: RollRowView[]): string[] =>
  filter(rows, 'selected').map((row) => String(row.id))

// `All` when every id of the axis is picked — "every character" and "these N characters" are
// different answers the day a patch adds one more — otherwise the ids themselves, sorted
// numerically. Empty stays `only { ids: [] }`, never `All`: ticking nothing means nothing, and
// `All` would draw from everything, the opposite.
export const toggledSelection = (
  allIds: number[],
  picked: string[],
): SelectionView => {
  const ids = sortBy(picked.map(Number))
  return isEqual(sortBy(allIds), ids) ? { kind: 'all' } : { kind: 'only', ids }
}

// `FacetOption[]`, carrying `targets` straight through as `count`. It never recounts: the
// number is the one `roll::contributions` already computed, and a second definition here
// would drift from it.
export const rowOptions = (rows: RollRowView[]): FacetOption[] =>
  rows.map((row) => ({
    value: String(row.id),
    label: row.name,
    count: row.targets,
    picked: row.selected,
  }))
