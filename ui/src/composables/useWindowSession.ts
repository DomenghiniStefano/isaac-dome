import { onBeforeUnmount, onMounted, watch } from 'vue'
import { assertNever } from '@/lib/assertNever'
import { windowSession } from '@/lib/ipc/session'
import { withOptional } from '@/lib/withOptional'
import { useTabsStore } from '@/stores/tabs'
import { watchWindowBox, watchWindowFocus } from '@/lib/window/appWindow'
import { focusOrder, rememberFocus } from '@/lib/window/focusOrder'
import { WindowMessageKind } from '@/lib/window/messages'
import type { WindowMessage } from '@/lib/window/messages'
import { sidebarCollapsed, sidebarWidth } from '@/lib/window/layout'
import { layoutEcho } from '@/lib/window/layoutEcho'
import { replaceableTimeout } from '@/lib/window/replaceableTimeout'
import { reopenWindows } from '@/lib/window/reopen'
import { takeSeed } from '@/lib/window/seeds'
import { readSession } from '@/lib/window/sessionDocument'
import type {
  StoredBox,
  StoredSession,
  StoredWindow,
} from '@/lib/window/sessionDocument'
import { createSessionLedger } from '@/lib/window/sessionLedger'
import { sessionSaver } from '@/lib/window/sessionSaver'
import { subscriptions } from '@/lib/window/subscriptions'
import { windowPort } from '@/lib/window/windowPort'
import { boxOf } from '@/lib/drag/dragList'

// How long a newborn window waits for the seed that says what it holds before falling back to
// its landing tab. **It is a deadline, not a delay**: whoever owes the seed is another window of
// the same process and answers in the time it takes to deliver one event, so this number is only
// ever paid by a window nobody owes anything to — which is every window that *reloads*, because
// its creator settled the debt the first time. At three seconds that was three seconds of empty
// bar on every reload (owner, 2026-09-13). Short enough not to be noticed, long enough that an
// answer in flight is never cut off.
const SeedTimeout = 700

// Where this window is, is the other half of what the session stores about it. A window that
// cannot say where it is still has tabs worth storing, so this never throws upward: the
// document simply carries no box for it, and the restore cascades it instead.
const measuredBox = async (): Promise<StoredBox | undefined> => {
  try {
    return boxOf(await windowPort.self())
  } catch {
    return undefined
  }
}

// **Nobody owes the first window a seed**: what it holds is its own last session. Everything
// that can go wrong — no session, the setting off, a document we can't read, a read that
// throws — ends at the same empty seed.
const readStoredSession = async (): Promise<StoredSession | null> => {
  try {
    return readSession(await windowSession())
  } catch {
    return null
  }
}

// A window's whole cross-window life: one listener, one exhaustive switch. Mounted once, by
// App.vue. What every window holds is the ledger's, when it is written is the saver's, and the
// sidebar's one layout is the echo's; this wires them to the messages and the watchers.
export const useWindowSession = (): void => {
  const tabs = useTabsStore()
  // Made one after another across the awaits below, and a window can close between any two.
  const listening = subscriptions()
  const seedDeadline = replaceableTimeout()
  const ledger = createSessionLedger()
  const saver = sessionSaver(ledger)
  const layout = layoutEcho()
  const place: { box: StoredBox | undefined } = { box: undefined }

  // What this window holds, for the ledger and for the broadcast.
  const mine = (): StoredWindow => {
    const { tabs: seeds, activeIndex } = tabs.session
    return { tabs: seeds, activeIndex, ...withOptional('box', place.box) }
  }

  const announce = (): void => {
    const held = mine()
    ledger.hold(windowPort.label(), held)
    void windowPort.broadcast({
      kind: WindowMessageKind.Holding,
      label: windowPort.label(),
      tabs: held.tabs,
      activeIndex: held.activeIndex,
      ...withOptional('box', held.box),
    })
  }

  // What this window holds changed: everybody hears it, and the writer writes it down.
  const held = (): void => {
    announce()
    saver.remember()
  }

  // A newborn asked for what it is owed. Somebody new exists, and the layout is one value for
  // the app: a torn-off window must open with the sidebar its creator has, not with the default.
  const answerReady = (label: string): void => {
    const seed = takeSeed(label)
    if (!seed) return
    void windowPort.send(label, {
      kind: WindowMessageKind.Seed,
      tabs: seed.tabs,
      activeIndex: seed.activeIndex,
    })
    layout.tell()
  }

  const onMessage = (m: WindowMessage) => {
    switch (m.kind) {
      case WindowMessageKind.Ready:
        answerReady(m.label)
        return
      case WindowMessageKind.Seed:
        if (!tabs.pending) return
        seedDeadline.clear()
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
        ledger.hold(m.label, {
          tabs: m.tabs,
          activeIndex: m.activeIndex,
          ...withOptional('box', m.box),
        })
        saver.remember()
        return
      case WindowMessageKind.Closing:
        ledger.leave(m.label)
        saver.remember()
        return
      case WindowMessageKind.Layout:
        layout.take({
          sidebarWidth: m.sidebarWidth,
          sidebarCollapsed: m.sidebarCollapsed,
        })
        saver.remember()
        return
      default:
        return assertNever(m)
    }
  }

  const readBox = async (): Promise<void> => {
    place.box = await measuredBox()
  }

  // Everything this window listens to for as long as it lives: its own tabs and layout, the
  // other windows, its focus and its place on the desktop.
  const installWatchers = async (): Promise<void> => {
    // Deep: a tab navigating changes an entry inside the array, not the array itself, and a
    // shallow watch would save the bar's shape and never what it is showing.
    watch(() => tabs.session, held, { deep: true })
    // The sidebar is not a tab and not a window: one layout, beside them in the document, and the
    // others hear it change.
    watch([sidebarWidth, sidebarCollapsed], () => {
      layout.changed()
      saver.remember()
    })
    await listening.add(() => windowPort.listen(onMessage))
    // Who is in front, told by the only thing that observes it: this window's own focus.
    // Broadcast, so every window keeps the same order and the hit test agrees everywhere.
    await listening.add(() =>
      watchWindowFocus((focused) => {
        if (!focused) return
        void windowPort.broadcast({
          kind: WindowMessageKind.Focused,
          label: windowPort.label(),
        })
      }),
    )
    await readBox()
    await listening.add(() =>
      watchWindowBox(() => {
        void readBox().then(held)
      }),
    )
  }

  const restoreMain = async (): Promise<void> => {
    const restored = await readStoredSession()
    // The deadline may have fired while the read was in flight. It has already seeded the bar,
    // and replacing it now would swap the user's tabs under them a beat after they appeared.
    if (!tabs.pending) return
    seedDeadline.clear()
    // The sidebar you sized and folded is the sidebar you get back. Set before the seeding, so
    // the first paint is already at the right width rather than snapping to it.
    layout.take({
      sidebarWidth: restored?.sidebarWidth ?? null,
      sidebarCollapsed: restored?.sidebarCollapsed === true,
    })
    // `main` takes the first window of the document and reopens the rest — a restored window is
    // a torn-off window that nobody dragged, so this is the tear-off's own machinery.
    const windows = restored?.windows ?? []
    tabs.seed(windows[0]?.tabs ?? [], windows[0]?.activeIndex ?? 0)
    held()
    void reopenWindows(windows.slice(1))
  }

  onMounted(async () => {
    await installWatchers()
    if (!tabs.pending) {
      held()
      return
    }
    // The deadline is the same for every window: whoever is owed nothing ends up with an empty
    // seed, which `seedState` turns into the landing tab.
    seedDeadline.set(() => {
      tabs.seed([], 0)
      held()
    }, SeedTimeout)
    if (windowPort.isMain()) {
      await restoreMain()
      return
    }
    // Any other window was born from a tear-off or a restore, and asks the window that owes it.
    await windowPort.broadcast({
      kind: WindowMessageKind.Ready,
      label: windowPort.label(),
    })
  })

  onBeforeUnmount(() => {
    listening.stop()
    saver.stop()
    seedDeadline.clear()
    // Not a write — the webview is being torn down. A word to the survivors, whose own write is
    // what records that this window has gone.
    void windowPort.broadcast({
      kind: WindowMessageKind.Closing,
      label: windowPort.label(),
    })
  })
}
