import { EventKey } from '@/lib/constants/eventKeys'

// Who owns a key inside a screen — the second entry on that question after B65, and written
// here rather than inside a component for the same reason: a rule buried in a `@keydown` is a
// rule nobody can check. B65 was `Ctrl+Enter` being swallowed by a listbox that read the key
// first; the lesson is that **one gesture has one owner**, declared somewhere it can be read.
//
// The split this bar takes:
//
// - `Ctrl+F` belongs to the **shell**, like the palette's `Ctrl+K`: it can arrive from
//   anywhere, so it is claimed once and high up.
// - `Enter`, `Shift+Enter` and `Escape` belong to the **bar**, and only while the bar has the
//   keyboard. They are bare keys, which is only safe because the bar is a text field: the
//   moment it is closed it stops listening and the screen has them back.
// - Everything else goes through untouched — including `Ctrl+Enter`, which is the palette's
//   gesture. A find bar that claimed it would be B65 happening a second time.

// A bare `f` has to keep working, or the bar could not be typed into; `Ctrl+Shift+F` is left
// alone because it is the "find in all files" of other tools and this bar cannot do that.
export const opensFind = (event: KeyboardEvent): boolean =>
  event.ctrlKey &&
  !event.shiftKey &&
  !event.altKey &&
  event.key.toLowerCase() === EventKey.F

export const FindAction = {
  Next: 'next',
  Previous: 'previous',
  Close: 'close',
} as const
export type FindAction = (typeof FindAction)[keyof typeof FindAction]

export const findAction = (event: KeyboardEvent): FindAction | null => {
  if (event.ctrlKey || event.altKey) return null
  if (event.key === EventKey.Escape) return FindAction.Close
  if (event.key !== EventKey.Enter) return null
  return event.shiftKey ? FindAction.Previous : FindAction.Next
}
