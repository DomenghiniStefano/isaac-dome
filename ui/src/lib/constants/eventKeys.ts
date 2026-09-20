// KeyboardEvent.key values components react to. Not key caps (those are KeyName, shown
// to the user): these are what the browser reports, and never displayed.
export const EventKey = {
  F: 'f',
  K: 'k',
  Enter: 'Enter',
  Escape: 'Escape',
  ArrowLeft: 'ArrowLeft',
  ArrowRight: 'ArrowRight',
  ArrowUp: 'ArrowUp',
  ArrowDown: 'ArrowDown',
} as const
export type EventKey = (typeof EventKey)[keyof typeof EventKey]
