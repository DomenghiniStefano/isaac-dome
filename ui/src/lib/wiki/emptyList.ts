import type { MessageSchema } from '@/i18n/messages/it'
import type { MessageKey } from '@/i18n/messageKey'

// What an empty list of pages is, and what there is to undo. The same split the Runs screen
// makes, with the second half read off the query instead of the total: the button says "clear
// the search", so it belongs where there is a search, not where there are pages.
export interface EmptyList {
  text: MessageKey<MessageSchema>
  reset: boolean
}

// `total` is the category's pages before the filter: 0 means the category is empty, and no query
// can be the reason for it — what was never there must not be drawn as a search that failed. The
// query is trimmed the way `filterPages` trims it, so spaces are nothing typed here too.
export const emptyList = (total: number, query: string): EmptyList => ({
  text: total === 0 ? 'wiki.emptyCategory' : 'wiki.noResults',
  reset: query.trim() !== '',
})
