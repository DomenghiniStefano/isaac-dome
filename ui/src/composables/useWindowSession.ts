import { onBeforeUnmount, onMounted, watch } from 'vue'
import { assertNever } from '@/lib/assertNever'
import { setWindowSession, windowSession } from '@/lib/ipc/session'
import { useTabsStore } from '@/stores/tabs'
import { watchWindowBox, watchWindowFocus } from '@/lib/window/appWindow'
import { focusOrder, rememberFocus } from '@/lib/window/focusOrder'
import { WindowMessageKind } from '@/lib/window/messages'
import type { WindowMessage } from '@/lib/window/messages'
import {
  currentLayout,
  sameLayout,
  setLayout,
  sidebarCollapsed,
  sidebarWidth,
} from '@/lib/window/layout'
import type { Layout } from '@/lib/window/layout'
import { clampToMonitors } from '@/lib/window/monitorClamp'
import { oweSeed, takeSeed } from '@/lib/window/seeds'
import { readSession, writeSession } from '@/lib/window/sessionDocument'
import type {
  StoredBox,
  StoredSession,
  StoredWindow,
} from '@/lib/window/sessionDocument'
import { noteSessionError } from '@/lib/window/sessionHealth'
import { SessionAction, decideSessionWrite } from '@/lib/window/sessionWriter'
import { newWindowLabel, windowPort } from '@/lib/window/windowPort'
import type { WindowBox } from '@/lib/window/windowPort'

// How long a newborn window waits for the seed that says what it holds before falling back to
// its landing tab. **It is a deadline, not a delay**: whoever owes the seed is another window of
// the same process and answers in the time it takes to deliver one event, so this number is only
// ever paid by a window nobody owes anything to — which is every window that *reloads*, because
// its creator settled the debt the first time. At three seconds that was three seconds of empty
// bar on every reload (owner, 2026-09-13). Short enough not to be noticed, long enough that an
// answer in flight is never cut off.
const SeedTimeout = 700

// How long the tabs have to settle before what they are is written down. Long enough that
// opening a tab is one write and not three, short enough that closing the window a moment
// later still finds it saved.
const SaveDelay = 400

// How far a restored window is stepped from the one before it when the document has no box for
// it — a version 1 session, or a window whose geometry could not be read. Window geometry, like
// `StripBand` and `TearBand`: it is not a visual constant and never reaches a template.
const CascadeStep = 32

// How many times a write waits for a window that has not said what it holds yet before going
// ahead without it. It exists because **the roster can name a window that is already gone**:
// `getAllWebviewWindows` keeps listing a webview for a while after it closes — the same fact the
// hit test carries a comment about — so a window that crashed without saying `Closing` would
// otherwise postpone every write for ever, and the session would silently stop being saved. Which
// is the exact failure this whole sub-project is here to remove. Three attempts is 1.2 s, well
// past the seed deadline that bounds an honest wait.
const PostponeLimit = 3

const boxOf = (w: WindowBox): StoredBox => ({
  left: w.left,
  top: w.top,
  width: w.width,
  height: w.height,
})

// A window's whole cross-window life: one listener, one exhaustive switch. Mounted once, by
// App.vue. Docking and hovering fill the arms that are empty here.
export const useWindowSession = (): void => {
  const tabs = useTabsStore()
  let stop: (() => void) | null = null
  let stopFocus: (() => void) | null = null
  let stopBox: (() => void) | null = null
  let timer: number | null = null
  let saving: number | null = null

  // What every window holds, as each of them last said so — this window's own entry included.
  // **Every window keeps the whole ledger**, not only the one that writes, because the writer
  // changes with a single close and a window that had kept nothing would have nothing to write
  // with.
  const ledger = new Map<string, StoredWindow>()
  // Windows that have said they were going. The roster still names them for a while, and a label
  // in here is one nothing should be waited for — nor elected.
  const gone = new Set<string>()
  let box: StoredBox | undefined
  let postponed = 0
  // The sidebar layout this window has already told the others about, so that hearing it back —
  // or hearing it from somebody else — is not a reason to say it again.
  let announced: Layout | null = null

  // One layout for the app, so a window that changes it tells the rest. Also the answer a newborn
  // gets, which is why it is a function and not a line inside the watcher.
  const tellLayout = (): void => {
    void windowPort.broadcast({
      kind: WindowMessageKind.Layout,
      ...currentLayout(),
    })
  }

  const forget = () => {
    if (timer !== null) window.clearTimeout(timer)
    timer = null
  }

  // What this window holds, for the ledger and for the broadcast.
  const mine = (): StoredWindow => {
    const { tabs: seeds, activeIndex } = tabs.session
    return { tabs: seeds, activeIndex, ...(box ? { box } : {}) }
  }

  const announce = (): void => {
    const held = mine()
    ledger.set(windowPort.label(), held)
    void windowPort.broadcast({
      kind: WindowMessageKind.Holding,
      label: windowPort.label(),
      tabs: held.tabs,
      activeIndex: held.activeIndex,
      ...(held.box ? { box: held.box } : {}),
    })
  }

  // **The session is every window, and the window that writes it is elected** — not `main`, which
  // is what it was until 3.7b. Nothing prevents main from being closed while other windows live,
  // so "only main writes" meant the session stopped being written the moment the user closed the
  // first window, silently, with the app alive in the tray to prove it.
  //
  // Still not on close: the webview is being torn down at that moment, and a write that races the
  // teardown is a write that sometimes doesn't happen. What a closing window does is say so, and
  // the survivors write.
  const write = async (): Promise<void> => {
    // The roster decides, not the ledger: a window that has gone leaves nothing behind, and a
    // `Closing` lost to the teardown must not cost the document its accuracy. What the roster
    // cannot be trusted about is the other direction — it keeps naming a webview after it has
    // closed — so a window that said it was going is taken out of it here.
    const labels = (await windowPort.labels()).filter(
      (label) => !gone.has(label),
    )
    for (const label of [...ledger.keys()])
      if (!labels.includes(label)) ledger.delete(label)
    const decision = decideSessionWrite(
      windowPort.label(),
      labels,
      ledger,
      postponed < PostponeLimit,
    )
    switch (decision.kind) {
      case SessionAction.Nothing:
        return
      case SessionAction.Postpone:
        postponed += 1
        remember()
        return
      case SessionAction.Write:
        postponed = 0
        try {
          await setWindowSession(
            writeSession({
              windows: decision.windows,
              ...(sidebarWidth.value === null
                ? {}
                : { sidebarWidth: sidebarWidth.value }),
              ...(sidebarCollapsed.value ? { sidebarCollapsed: true } : {}),
            }),
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
    if (saving !== null) window.clearTimeout(saving)
    saving = window.setTimeout(() => void write(), SaveDelay)
  }

  // What this window holds changed: everybody hears it, and the writer writes it down.
  const held = (): void => {
    announce()
    remember()
  }

  // The windows this session had, minus the one `main` took for itself. Sequential and not
  // `Promise.all`: each newborn is owed its seed before the next is created, exactly as
  // `openWindowWith` waits — the debt lives in this window's memory and a newborn that asks
  // before it is registered opens with an empty bar.
  const reopen = async (rest: readonly StoredWindow[]): Promise<void> => {
    if (rest.length === 0) return
    // **Only at a launch, and a launch is one window.** `main` re-runs this whole mount whenever
    // its webview reloads — which is every save on the development server — and there it would
    // find the session it wrote a moment ago and open a second copy of every window in it. At a
    // real launch the roster is `main` alone, which is the difference between the two, and
    // reading it costs one call.
    if ((await windowPort.labels()).length > 1) return
    const monitors = await windowPort.monitors()
    const self = await windowPort.self()
    let step = 0
    for (const window of rest) {
      step += 1
      const wanted = window.box ?? {
        left: self.left + CascadeStep * step,
        top: self.top + CascadeStep * step,
        width: self.width,
        height: self.height,
      }
      const placed = clampToMonitors(wanted, monitors)
      const label = newWindowLabel()
      const paid = oweSeed(label, window.tabs, window.activeIndex)
      await windowPort.create(
        label,
        { x: placed.box.left, y: placed.box.top },
        {
          // Physical in, logical out: `create` takes a physical position and a logical size.
          x: placed.box.width / placed.scaleFactor,
          y: placed.box.height / placed.scaleFactor,
        },
      )
      await paid
    }
  }

  const onMessage = (m: WindowMessage) => {
    switch (m.kind) {
      case WindowMessageKind.Ready: {
        const seed = takeSeed(m.label)
        if (!seed) return
        void windowPort.send(m.label, {
          kind: WindowMessageKind.Seed,
          tabs: seed.tabs,
          activeIndex: seed.activeIndex,
        })
        // Somebody new exists, and the layout is one value for the app: a torn-off window must
        // open with the sidebar its creator has, not with the default.
        tellLayout()
        return
      }
      case WindowMessageKind.Seed:
        if (!tabs.pending) return
        forget()
        tabs.seed(m.tabs, m.activeIndex)
        return
      case WindowMessageKind.Docked:
        tabs.dock(m.tab)
        return
      case WindowMessageKind.Focused:
        focusOrder.value = rememberFocus(focusOrder.value, m.label)
        return
      case WindowMessageKind.Hovering:
        void tabs.aimIncoming(m.at)
        return
      case WindowMessageKind.HoverLeft:
        tabs.clearIncoming()
        return
      case WindowMessageKind.Holding:
        ledger.set(m.label, {
          tabs: m.tabs,
          activeIndex: m.activeIndex,
          ...(m.box ? { box: m.box } : {}),
        })
        remember()
        return
      case WindowMessageKind.Closing:
        gone.add(m.label)
        ledger.delete(m.label)
        remember()
        return
      case WindowMessageKind.Layout:
        // **Taken as already said.** The assignment wakes this window's own watcher, which would
        // broadcast it back, which would wake everyone else's: one drag would cost a round of
        // messages per window. Recording it as announced *before* setting it is what stops that,
        // and it does not depend on when the watcher happens to flush — a synchronous flag would.
        announced = {
          sidebarWidth: m.sidebarWidth,
          sidebarCollapsed: m.sidebarCollapsed,
        }
        setLayout(announced)
        remember()
        return
      default:
        return assertNever(m)
    }
  }

  onMounted(async () => {
    // Deep: a tab navigating changes an entry inside the array, not the array itself, and a
    // shallow watch would save the bar's shape and never what it is showing.
    watch(() => tabs.session, held, { deep: true })
    // The sidebar is not a tab and not a window: one layout, beside them in the document, and the
    // others hear it change.
    watch([sidebarWidth, sidebarCollapsed], () => {
      const now = currentLayout()
      if (!sameLayout(announced, now)) {
        announced = now
        tellLayout()
      }
      remember()
    })
    stop = await windowPort.listen(onMessage)
    // Who is in front, told by the only thing that observes it: this window's own focus.
    // Broadcast, so every window keeps the same order and the hit test agrees everywhere.
    stopFocus = await watchWindowFocus((focused) => {
      if (!focused) return
      void windowPort.broadcast({
        kind: WindowMessageKind.Focused,
        label: windowPort.label(),
      })
    })
    // Where this window is, is the other half of what the session stores about it. A window that
    // cannot say where it is still has tabs worth storing, so this never throws upward: the
    // document simply carries no box for it, and the restore cascades it instead.
    const readBox = async (): Promise<void> => {
      try {
        box = boxOf(await windowPort.self())
      } catch {
        box = undefined
      }
    }
    await readBox()
    stopBox = await watchWindowBox(() => {
      void readBox().then(held)
    })
    if (!tabs.pending) {
      held()
      return
    }
    // The deadline is the same for both: whoever is owed nothing ends up with an empty seed,
    // which `seedState` turns into the landing tab.
    timer = window.setTimeout(() => {
      tabs.seed([], 0)
      held()
    }, SeedTimeout)
    if (windowPort.isMain()) {
      // **Nobody owes the first window a seed**: what it holds is its own last session.
      // Everything that can go wrong — no session, the setting off, a document we can't read,
      // a read that throws — ends at the same empty seed.
      let restored: StoredSession | null = null
      try {
        restored = readSession(await windowSession())
      } catch {
        restored = null
      }
      // The deadline may have fired while the read was in flight. It has already seeded the
      // bar, and replacing it now would swap the user's tabs under them a beat after they
      // appeared.
      if (!tabs.pending) return
      forget()
      // The sidebar you sized and folded is the sidebar you get back. Set before the seeding, so
      // the first paint is already at the right width rather than snapping to it.
      const stored: Layout = {
        sidebarWidth: restored?.sidebarWidth ?? null,
        sidebarCollapsed: restored?.sidebarCollapsed === true,
      }
      announced = stored
      setLayout(stored)
      // `main` takes the first window of the document and reopens the rest — a restored window
      // is a torn-off window that nobody dragged, so this is the tear-off's own machinery.
      const windows = restored?.windows ?? []
      tabs.seed(windows[0]?.tabs ?? [], windows[0]?.activeIndex ?? 0)
      held()
      void reopen(windows.slice(1))
      return
    }
    await windowPort.broadcast({
      kind: WindowMessageKind.Ready,
      label: windowPort.label(),
    })
  })

  onBeforeUnmount(() => {
    stop?.()
    stopFocus?.()
    stopBox?.()
    if (saving !== null) window.clearTimeout(saving)
    forget()
    // Not a write — the webview is being torn down. A word to the survivors, whose own write is
    // what records that this window has gone.
    void windowPort.broadcast({
      kind: WindowMessageKind.Closing,
      label: windowPort.label(),
    })
  })
}
