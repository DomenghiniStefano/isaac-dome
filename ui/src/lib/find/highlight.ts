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

export const segments = (text: string, query: string): Segment[] => {
  if (text === '') return []
  const needle = query.trim().toLowerCase()
  if (needle === '') return [{ text, match: false }]

  const haystack = text.toLowerCase()
  const pieces: Segment[] = []
  let at = 0
  for (;;) {
    const hit = haystack.indexOf(needle, at)
    if (hit < 0) break
    if (hit > at) pieces.push({ text: text.slice(at, hit), match: false })
    pieces.push({ text: text.slice(hit, hit + needle.length), match: true })
    at = hit + needle.length
  }
  if (pieces.length === 0) return [{ text, match: false }]
  if (at < text.length) pieces.push({ text: text.slice(at), match: false })
  return pieces
}
