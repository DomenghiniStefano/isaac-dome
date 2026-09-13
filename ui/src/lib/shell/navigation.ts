// Going back and forward inside a tab, with the two gestures a browser has taught: the
// side buttons of the mouse and `Alt` with an arrow. Both end up on the same action, so the
// shell binds one handler and the rules stay testable without an event loop.
export const HistoryAction = {
  Back: 'back',
  Forward: 'forward',
} as const
export type HistoryAction = (typeof HistoryAction)[keyof typeof HistoryAction]

const keys: Record<string, HistoryAction> = {
  ArrowLeft: HistoryAction.Back,
  ArrowRight: HistoryAction.Forward,
}

// A bare arrow belongs to whatever table or list has the focus, and Alt is the only
// modifier the shortcut carries: anything else is somebody's else key combination.
export const historyAction = (event: KeyboardEvent): HistoryAction | null =>
  event.altKey && !event.ctrlKey && !event.shiftKey
    ? (keys[event.key] ?? null)
    : null

// `MouseEvent.button` for the two side buttons. 0, 1 and 2 are the ones the app already
// listens to elsewhere: opening, closing a tab, the context menu.
const buttons: Record<number, HistoryAction> = {
  3: HistoryAction.Back,
  4: HistoryAction.Forward,
}

export const pointerHistoryAction = (event: MouseEvent): HistoryAction | null =>
  buttons[event.button] ?? null
