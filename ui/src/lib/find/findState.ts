// The find bar (B67) as a tab keeps it: what it is looking for and which match it is on, or
// `null` for the bar closed. Pure, so the moves a screen makes on it can be read in a test.
export interface FindState {
  query: string
  current: string | null
}

// Ctrl+F opens the bar, and on a bar already open changes nothing: the words stay.
export const openFind = (find: FindState | null): FindState =>
  find ?? { query: '', current: null }

export const findWithQuery = (
  find: FindState | null,
  query: string,
): FindState => ({ query, current: find?.current ?? null })

export const findWithCurrent = (
  find: FindState | null,
  current: string | null,
): FindState => ({ query: find?.query ?? '', current })
