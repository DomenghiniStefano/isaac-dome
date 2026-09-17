// How many picks the trigger spells out before it starts counting. Two fits a wrapping row of
// controls; the chips under the bar spell out every one of them anyway.
const SPELLED_OUT = 2

// From how many options the search field appears inside the menu. A behaviour, not a visual
// value, so it is a named constant here and not a token in `@theme` — and it is read off the
// list the control was actually given: how many pools or characters exist is a property of the
// user's install, not something a screen may assert on their behalf.
export const SEARCHABLE_FROM = 10

/** The trigger's tail: `Qualità · <this>`. `null` when nothing is picked. */
export const pickedSummary = (labels: string[]): string | null => {
  if (labels.length === 0) return null
  if (labels.length <= SPELLED_OUT) return labels.join(', ')
  return `${labels.slice(0, SPELLED_OUT).join(', ')} +${labels.length - SPELLED_OUT}`
}
