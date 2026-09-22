import type { Entry, Inline } from '@/lib/ipc/types'
import { refsOf } from './infoboxRefs'

/**
 * The line under a page's title.
 *
 * The entry's own description when it has one — the wiki's, or one written by hand in
 * `corrections.json`, which wins over the wiki. An achievement has none from the wiki: what
 * the wiki files there is the unlock paper's line ("Just Stop!", "???"), which the dataset
 * moved to the quote. So for an achievement the line is composed from what it gives —
 * "Unlocks Tainted Isaac", the verb passed in so it follows the reader's language — and,
 * where it unlocks no page the wiki names, from what it asks for.
 */
export const summaryOf = (
  entry: Entry,
  unlocksWord: string,
  titleOf: (key: string) => string | null,
): Array<Inline> => {
  if (entry.description.length > 0) return entry.description
  const box = entry.infobox
  if (box.kind !== 'achievement') return []
  if (box.unlocks !== null) {
    return [
      { kind: 'text', text: `${unlocksWord} `, style: 'plain' },
      ...refsOf([box.unlocks], titleOf),
    ]
  }
  return box.requirements
}
