// What the palette keeps when it closes, to show again — selected — the next time it opens.
//
// The last search is the likeliest next one: Enter opens its first row again, and the first
// key typed replaces it, so keeping it costs nothing to someone who wanted a new search. Only
// spaces is not a search, and reopening on them would look empty while not being empty.
export const queryToRecall = (typed: string): string =>
  typed.trim() === '' ? '' : typed
