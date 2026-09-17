import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import { assertNever } from '@/lib/assertNever'
import type { CandidatePreview, PreviewCount } from '@/lib/ipc/types'

type Message = MessageKey<MessageSchema>

/**
 * One line of a welcome card. `done` and `of` are **both null** when the file did not let us
 * read that section: the card then says so, rather than drawing a zero that reads as a
 * profile nobody has played.
 */
export type PreviewLine = {
  label: Message
  done: number | null
  of: number | null
}

const line = (label: Message, count: PreviewCount): PreviewLine => {
  switch (count.kind) {
    case 'read':
      return { label, done: count.done, of: count.of }
    case 'unread':
      return { label, done: null, of: null }
    default:
      return assertNever(count)
  }
}

/** Empty when the file could not be parsed at all: `unreadableNote` is what speaks there. */
export const previewLines = (
  preview: CandidatePreview | null,
): PreviewLine[] =>
  preview === null
    ? []
    : [
        line('welcome.card.achievements', preview.achievements),
        line('welcome.card.items', preview.items),
        line('welcome.card.marks', preview.marks),
      ]

export type UnreadableNote = { key: Message; count: number }

/**
 * The ⚠ line, or nothing. Three cases and not two: a file that could not be parsed at all is
 * not the same as one whose matrix has holes, and neither is a whole file.
 */
export const unreadableNote = (
  preview: CandidatePreview | null,
): UnreadableNote | null => {
  if (preview === null) return { key: 'welcome.card.unreadable', count: 0 }
  if (preview.unreadableCells === 0) return null
  return {
    key: 'welcome.card.unreadableCells',
    count: preview.unreadableCells,
  }
}
