// What the browser's own zoom shortcuts mean here. The app takes them over — `Ctrl` `+`,
// `Ctrl` `-`, `Ctrl` `0` — and they move the same persisted value the slider does: a
// shortcut is not a second scale (`docs/BACKLOG.md` B26).
export const ScaleAction = {
  In: 'in',
  Out: 'out',
  Reset: 'reset',
} as const
export type ScaleAction = (typeof ScaleAction)[keyof typeof ScaleAction]

// `=` is here because on the main row a plus needs Shift, and browsers report the unshifted
// key for the zoom shortcut; the numpad reports `+`.
const actions: Record<string, ScaleAction> = {
  '+': ScaleAction.In,
  '=': ScaleAction.In,
  '-': ScaleAction.Out,
  '0': ScaleAction.Reset,
}

export const shortcutAction = (event: KeyboardEvent): ScaleAction | null =>
  event.ctrlKey ? (actions[event.key] ?? null) : null
