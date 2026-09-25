// Painting the match inside the row, which is what tells you *why* a row is the one you
// landed on. Splitting is a pure function for the same reason the matching is: it can be
// checked without a window, and it is the piece where a mistake would be silent — a highlight
// that drops or duplicates a character corrupts the page it is drawing on.
//
// The match is case-insensitive and what is drawn is the **original** text: a row that wrote
// "Heart" must keep writing "Heart" even when you typed "heart".

export interface Segment {
  readonly text: string
  readonly match: boolean
}

// The pieces from `at` to the end: the text before the next match, the match, and the rest the
// same way. What follows the last match is one plain piece, or nothing when the match closes the
// text.
const piecesFrom = (
  text: string,
  haystack: string,
  needle: string,
  at: number,
): Segment[] => {
  const hit = haystack.indexOf(needle, at)
  if (hit < 0)
    return at < text.length ? [{ text: text.slice(at), match: false }] : []
  const end = hit + needle.length
  return [
    ...(hit > at ? [{ text: text.slice(at, hit), match: false }] : []),
    { text: text.slice(hit, end), match: true },
    ...piecesFrom(text, haystack, needle, end),
  ]
}

export const segments = (text: string, query: string): Segment[] => {
  if (text === '') return []
  const needle = query.trim().toLowerCase()
  if (needle === '') return [{ text, match: false }]
  return piecesFrom(text, text.toLowerCase(), needle, 0)
}
