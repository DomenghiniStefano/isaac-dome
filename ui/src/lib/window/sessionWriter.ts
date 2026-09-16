import type { StoredWindow } from './sessionDocument'
import { MainLabel } from './windowPort'

// Which window writes the session down. **Not "main"**, which is what it was until 3.7b and what
// `session.rs`'s own doc comment still said: nothing prevents main from being closed while other
// windows live — `open_or_focus` only re-creates it when there is no window at all — so under
// that rule the session stopped being written the moment the user closed the first window,
// silently, with the app alive in the tray to prove it.
//
// So every window elects, for itself, from the roster it just read. It is a **pure function of
// the set of labels**: two windows holding the same labels in a different order must reach the
// same writer, or two of them write at once. That is also what makes it a Vitest test instead of
// a thing you find out about in a window.

// `win-<Date.now() in base 36>`, which is what `newWindowLabel()` mints. Anything else — `main`,
// a label from a future shape — has no time, and sorts after everything that has one.
const Minted = /^win-([0-9a-z]+)$/

export const mintedAt = (label: string): number | null => {
  const match = Minted.exec(label)
  if (!match) return null
  const at = parseInt(match[1]!, 36)
  return Number.isFinite(at) ? at : null
}

// Main first — it is the first window and the one restored first — then oldest to newest, then
// whatever could not be read, by name so the answer never depends on the input's order.
export const windowOrder = (labels: readonly string[]): string[] =>
  [...labels].sort((a, b) => {
    if (a === b) return 0
    if (a === MainLabel) return -1
    if (b === MainLabel) return 1
    const at = mintedAt(a)
    const bt = mintedAt(b)
    if (at === null && bt === null) return a < b ? -1 : 1
    if (at === null) return 1
    if (bt === null) return -1
    return at === bt ? (a < b ? -1 : 1) : at - bt
  })

export const electWriter = (labels: readonly string[]): string | null =>
  windowOrder(labels)[0] ?? null

// What a window should do when something changed. Pure, and that is the point: "the session keeps
// being written once main is closed" is the whole of this sub-project, and it would otherwise be
// a thing you find out about by closing a window and restarting the app.
export const SessionAction = {
  Write: 'write',
  // Wait: a window in the roster has not said what it holds yet. Writing now would store a
  // session with one window missing.
  Postpone: 'postpone',
  // Not this window's job, or nothing worth storing.
  Nothing: 'nothing',
} as const
export type SessionAction = (typeof SessionAction)[keyof typeof SessionAction]

export type SessionDecision =
  | { kind: typeof SessionAction.Write; windows: StoredWindow[] }
  | { kind: typeof SessionAction.Postpone }
  | { kind: typeof SessionAction.Nothing }

export const decideSessionWrite = (
  self: string,
  roster: readonly string[],
  ledger: ReadonlyMap<string, StoredWindow>,
  patient: boolean,
): SessionDecision => {
  if (electWriter(roster) !== self) return { kind: SessionAction.Nothing }
  if (patient && roster.some((label) => !ledger.has(label)))
    return { kind: SessionAction.Postpone }
  const windows = windowOrder(roster)
    .map((label) => ledger.get(label))
    // A window mid-tear-off holds nothing for an instant. Storing that would restore an app with
    // no tabs, which is not what the user left.
    .filter((held): held is StoredWindow => !!held && held.tabs.length > 0)
  return windows.length === 0
    ? { kind: SessionAction.Nothing }
    : { kind: SessionAction.Write, windows }
}
