import type { StoredWindow } from './sessionDocument'

// What every window holds, as each of them last said so — this window's own entry included.
// **Every window keeps the whole ledger**, not only the one that writes, because the writer
// changes with a single close and a window that had kept nothing would have nothing to write
// with.
export interface SessionLedger {
  /** Everything held, by window label: what the writer writes. */
  held: ReadonlyMap<string, StoredWindow>
  /** A window said what it holds. */
  hold: (label: string, window: StoredWindow) => void
  /** A window said it was going: nothing is waited for from it again, nor is it elected. */
  leave: (label: string) => void
  /**
   * The windows that are really there, out of what the roster names, and the ledger cut down
   * to them. The roster decides, not the ledger: a window that has gone leaves nothing behind,
   * and a `Closing` lost to the teardown must not cost the document its accuracy. What the
   * roster cannot be trusted about is the other direction — it keeps naming a webview after it
   * has closed — so a window that said it was going is taken out of it here.
   */
  alive: (roster: readonly string[]) => string[]
}

export const createSessionLedger = (): SessionLedger => {
  const held = new Map<string, StoredWindow>()
  const gone = new Set<string>()
  return {
    held,
    hold: (label, window) => {
      held.set(label, window)
    },
    leave: (label) => {
      gone.add(label)
      held.delete(label)
    },
    alive: (roster) => {
      const labels = roster.filter((label) => !gone.has(label))
      ;[...held.keys()]
        .filter((label) => !labels.includes(label))
        .forEach((label) => held.delete(label))
      return labels
    },
  }
}
