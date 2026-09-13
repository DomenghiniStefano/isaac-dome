import { onBeforeUnmount, onMounted, watch } from 'vue'
import { assertNever } from '@/lib/assertNever'
import { setWindowSession, windowSession } from '@/lib/ipc/session'
import type { Session } from '@/stores/tabModel'
import { useTabsStore } from '@/stores/tabs'
import { watchWindowFocus } from './appWindow'
import { focusOrder, rememberFocus } from './focusOrder'
import { WindowMessageKind } from './messages'
import type { WindowMessage } from './messages'
import { takeSeed } from './seeds'
import { readSession, writeSession } from './sessionDocument'
import { windowPort } from './windowPort'

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

// A window's whole cross-window life: one listener, one exhaustive switch. Mounted once, by
// App.vue. Docking and hovering fill the arms that are empty here.
export const useWindowSession = (): void => {
  const tabs = useTabsStore()
  let stop: (() => void) | null = null
  let stopFocus: (() => void) | null = null
  let timer: number | null = null
  let saving: number | null = null

  const forget = () => {
    if (timer !== null) window.clearTimeout(timer)
    timer = null
  }

  // **Only `main`, and only as the tabs change.** Not on close: the webview is being torn down
  // at that moment, and a write that races the teardown is a write that sometimes doesn't
  // happen. A torn-off window's tabs are not the session.
  //
  // The debounce is for the burst — opening a tab moves the bar and the active index in the
  // same breath — not for the cost, which is one small row. A failed write is swallowed: the
  // tabs are on screen either way, and a dialog because a session didn't save would be worse
  // than the session not saving.
  const remember = (): void => {
    if (!windowPort.isMain()) return
    if (saving !== null) window.clearTimeout(saving)
    saving = window.setTimeout(() => {
      const { tabs: seeds, activeIndex } = tabs.session
      // A window mid-tear-off holds nothing for an instant. Storing that would restore an app
      // with no tabs, which is not what the user left.
      if (seeds.length === 0) return
      void setWindowSession(writeSession({ tabs: seeds, activeIndex })).catch(
        () => undefined,
      )
    }, SaveDelay)
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
      default:
        return assertNever(m)
    }
  }

  onMounted(async () => {
    // Deep: a tab navigating changes an entry inside the array, not the array itself, and a
    // shallow watch would save the bar's shape and never what it is showing.
    watch(() => tabs.session, remember, { deep: true })
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
    if (!tabs.pending) return
    // The deadline is the same for both: whoever is owed nothing ends up with an empty seed,
    // which `seedState` turns into the landing tab.
    timer = window.setTimeout(() => tabs.seed([], 0), SeedTimeout)
    if (windowPort.isMain()) {
      // **Nobody owes the first window a seed**: what it holds is its own last session.
      // Everything that can go wrong — no session, the setting off, a document we can't read,
      // a read that throws — ends at the same empty seed.
      let restored: Session | null = null
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
      tabs.seed(restored?.tabs ?? [], restored?.activeIndex ?? 0)
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
    if (saving !== null) window.clearTimeout(saving)
    forget()
  })
}
