import { assertNever } from '@/lib/assertNever'
import { setWindowSession } from '@/lib/ipc/session'
import { currentLayout } from './layout'
import { replaceableTimeout } from './replaceableTimeout'
import { storedSession, writeSession } from './sessionDocument'
import { noteSessionError } from './sessionHealth'
import type { SessionLedger } from './sessionLedger'
import { SessionAction, decideSessionWrite } from './sessionWriter'
import { windowPort } from './windowPort'

// How long the tabs have to settle before what they are is written down. Long enough that
// opening a tab is one write and not three, short enough that closing the window a moment
// later still finds it saved.
const SaveDelay = 400

// How many times a write waits for a window that has not said what it holds yet before going
// ahead without it. It exists because **the roster can name a window that is already gone**:
// `getAllWebviewWindows` keeps listing a webview for a while after it closes — the same fact the
// hit test carries a comment about — so a window that crashed without saying `Closing` would
// otherwise postpone every write for ever, and the session would silently stop being saved. Which
// is the exact failure this whole sub-project is here to remove. Three attempts is 1.2 s, well
// past the seed deadline that bounds an honest wait.
const PostponeLimit = 3

export interface SessionSaver {
  /** Something in the session changed: it is written once things settle, by the writer. */
  remember: () => void
  /** The window is going: nothing pending is written. */
  stop: () => void
}

// **The session is every window, and the window that writes it is elected** — not `main`, which
// is what it was until 3.7b. Nothing prevents main from being closed while other windows live,
// so "only main writes" meant the session stopped being written the moment the user closed the
// first window, silently, with the app alive in the tray to prove it.
//
// Still not on close: the webview is being torn down at that moment, and a write that races the
// teardown is a write that sometimes doesn't happen. What a closing window does is say so, and
// the survivors write.
export const sessionSaver = (ledger: SessionLedger): SessionSaver => {
  const pending = replaceableTimeout()
  const waits = { postponed: 0 }

  const save = async (): Promise<void> => {
    const labels = ledger.alive(await windowPort.labels())
    const decision = decideSessionWrite(
      windowPort.label(),
      labels,
      ledger.held,
      waits.postponed < PostponeLimit,
    )
    switch (decision.kind) {
      case SessionAction.Nothing:
        return
      case SessionAction.Postpone:
        waits.postponed += 1
        remember()
        return
      case SessionAction.Write:
        waits.postponed = 0
        try {
          await setWindowSession(
            writeSession(storedSession(decision.windows, currentLayout())),
          )
        } catch (e) {
          // Swallowed, with one exception: `SessionTooLarge` is the only error that means the
          // session has stopped being saved, and the only one the user has to be told about.
          noteSessionError(e)
        }
        return
      default:
        return assertNever(decision)
    }
  }

  // The debounce is for the burst — opening a tab moves the bar and the active index in the same
  // breath — not for the cost, which is one small row.
  const remember = (): void => {
    pending.set(() => void save(), SaveDelay)
  }

  return { remember, stop: pending.clear }
}
