// Delays in milliseconds that live in TypeScript rather than in a class: a debounce is not a
// visual constant, and the value is written once, here.
export const Timing = {
  SearchDebounce: 120,
  // Coalescing a burst — a facet click moves a pick and a count in the same breath. The same
  // 120 as the search and for the same reason, named separately because the day one moves the
  // other has no reason to. It costs nothing to be small: what it feeds is the session's own
  // 400ms write, so anything under that is invisible.
  ViewWrite: 120,
} as const
