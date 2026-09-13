import { onBeforeUnmount, onMounted } from 'vue'
import { assertNever } from '@/lib/assertNever'
import { useTabsStore } from '@/stores/tabs'
import { watchWindowFocus } from './appWindow'
import { focusOrder, rememberFocus } from './focusOrder'
import { WindowMessageKind } from './messages'
import type { WindowMessage } from './messages'
import { takeSeed } from './seeds'
import { windowPort } from './windowPort'

// How long a newborn window waits for the seed that says what it holds. If the window that
// created it died in between, nobody will ever answer: it opens on the landing page rather
// than showing a bar with no tabs. Degrade, never fail.
const SeedTimeout = 3000

// A window's whole cross-window life: one listener, one exhaustive switch. Mounted once, by
// App.vue. Docking and hovering fill the arms that are empty here.
export const useWindowSession = (): void => {
  const tabs = useTabsStore()
  let stop: (() => void) | null = null
  let stopFocus: (() => void) | null = null
  let timer: number | null = null

  const forget = () => {
    if (timer !== null) window.clearTimeout(timer)
    timer = null
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
    timer = window.setTimeout(() => tabs.seed([], 0), SeedTimeout)
    await windowPort.broadcast({
      kind: WindowMessageKind.Ready,
      label: windowPort.label(),
    })
  })

  onBeforeUnmount(() => {
    stop?.()
    stopFocus?.()
    forget()
  })
}
