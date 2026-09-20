// Find-in-page: which rows match what you typed, and which of them you are on.
//
// This is not the palette. The palette (`lib/search/`) **goes somewhere** — it leaves the
// screen and opens a result in a tab. This one **stays here**, so it works on the rows the
// screen already holds and never on the document.
//
// That distinction is the whole reason it is custom. Five lists are virtualized
// (`CollectionTable`, `UnlockTable`, `RunsTable`, `SearchResults`, `WikiCategoryList`), so of
// 733 items only the twenty-odd rows in view are in the DOM at all. A find that walked the
// document would answer a question about the scroll position, not about the list: it would say
// "not found" for a row that exists three thousand pixels further down, which is worse than
// having no find. So the screen hands over its rows, and the matching happens here — where it
// can be checked without a window.
//
// What it deliberately does not do: regex, a case option, or searching a screen you are not on.

export interface FindRow {
  readonly key: string
  readonly text: string
}

export const Direction = {
  Next: 'next',
  Previous: 'previous',
} as const
export type Direction = (typeof Direction)[keyof typeof Direction]

export interface FindState {
  /** The keys of the matching rows, in the order the screen gave them. */
  readonly matches: readonly string[]
  /** The row the bar is on, or `null` when nothing matches. */
  readonly current: string | null
  /** One-based, and `0` when nothing matches: the "3" of "3 di 17". */
  readonly position: number
  /** The "17" of "3 di 17". */
  readonly total: number
}

const empty: FindState = { matches: [], current: null, position: 0, total: 0 }

const settle = (
  matches: readonly string[],
  current: string | null,
): FindState => {
  const kept =
    current !== null && matches.includes(current)
      ? current
      : (matches[0] ?? null)
  return {
    matches,
    current: kept,
    position: kept === null ? 0 : matches.indexOf(kept) + 1,
    total: matches.length,
  }
}

// The count is a position and a total, never a formatted string: "3 di 17" is the template's
// sentence, and a pure function that returned it would have to know the language.
export const findMatches = (
  rows: readonly FindRow[],
  query: string,
  current: string | null,
): FindState => {
  const needle = query.trim().toLowerCase()
  // An empty query matches nothing rather than everything: a bar reporting "1 di 733" the
  // moment it opens has told you the length of the list, which you did not ask for.
  if (!needle) return empty
  const matches = rows
    .filter((row) => row.text.toLowerCase().includes(needle))
    .map((row) => row.key)
  // Keeping the current row when it survives is the lesson B65 paid for: the state is
  // recomputed on every keystroke, and one more letter must not drag you back to the top of a
  // list you had already walked into.
  return settle(matches, current)
}

export const stepMatch = (
  state: FindState,
  direction: Direction,
): FindState => {
  if (state.total === 0) return empty
  const at = state.current === null ? -1 : state.matches.indexOf(state.current)
  const step = direction === Direction.Next ? 1 : -1
  // Wrapping in both directions, so the last hit leads back to the first and the first back to
  // the last. A find that stops at the end makes you guess whether it is over or broken.
  const next =
    at < 0
      ? step === 1
        ? 0
        : state.total - 1
      : (at + step + state.total) % state.total
  return { ...state, current: state.matches[next] ?? null, position: next + 1 }
}
