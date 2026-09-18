import type { SearchRow } from './rows'

// Which row the highlight belongs on once an answer lands.
//
// The listbox highlights the first item on **every keystroke**, and the rows for that
// keystroke arrive 120 ms later (`Timing.SearchDebounce`): the row it highlighted is the one
// the previous answer had, and it unmounts when the new one is drawn. A highlight on a
// detached row is not visible and does not open — which is what makes Enter do nothing.
//
// So the palette puts it back itself, and keeps the user's own position when the row it was
// on survived the new answer: arrowing down to the third result and typing one more letter
// should not throw you back to the top if that result is still there.
export const keyAfterAnswer = (
  rows: readonly Pick<SearchRow, 'key'>[],
  highlighted: string | null,
): string | null => {
  if (rows.some((row) => row.key === highlighted)) return highlighted
  return rows[0]?.key ?? null
}
