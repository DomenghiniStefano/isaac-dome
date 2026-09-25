import { once } from 'lodash-es'

// What a fixture made up, said on the console the first time it is used and never again: once
// per page load is enough for somebody looking at the screen, and a warning per call would bury
// the one that matters.
export const warnOnce = (message: string): (() => void) =>
  once(() => console.warn(message))
