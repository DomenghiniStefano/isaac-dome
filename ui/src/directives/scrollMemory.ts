import type { Directive } from 'vue'
import { Timing } from '@/lib/constants/timing'
import { RestoreStep, restoreStep } from '@/lib/scroll/restore'
import { useTabsStore } from '@/stores/tabs'
import { entryScroll, tabLocation } from '@/stores/tabModel'
import type { EntryAddress } from '@/stores/tabModel'

// `v-scroll-memory="'page'"`: this region remembers where it was scrolled to, per tab and per
// history entry, and goes back there when the entry is shown again — after a tab switch, a back,
// a tear-off, or a restored session. The owner's words (2026-09-24): going back loses *nothing*.
//
// **Every region that scrolls on a screen carries it**, and `scan-conventions.mjs` says so: a
// scrolling box without a name is a position the next tab switch throws away. The name is the
// region's within its screen ('page', 'queue', 'pane'), which is how two regions of one entry keep
// two positions.
//
// The lists built on `VirtualRows` do not need it: they keep their own offset in the screen's
// reading, measured against a row count so that a list that changed underneath opens at the top.
//
// Why it is so careful about *when*:
// - **It restores only once the content can reach the position** (`restoreStep`). A screen is
//   mounted before its data, and a position set on an empty page is clamped to nothing.
// - **It never saves while a restore is pending.** A tab switched away before the data arrived
//   would otherwise write the empty page's 0 over the position it was about to go back to.
// - **It gives up on restoring the moment you scroll yourself**, and after a few seconds: what
//   you do wins over what you did.
// - **The address is taken when it mounts.** The last position is saved as the region is taken
//   down, and it is taken down *because* the tab or the entry changed — the store's `setScroll`
//   is addressed for exactly that reason.

const GiveUpMs = 4000

// How long the content must stay still after the position is reached before the restore lets go.
// Reaching it is not the end: what arrives *above* it afterwards — a diagnostic, a banner — pushes
// the page down, the browser's scroll anchoring follows the content, and the page ends up lower
// than it was left. Measured on Floor (2026-09-24): restored to 250, the rules' answer landed a
// moment later and the page settled at 283. So the position is put back on every change until
// the page is quiet.
const QuietMs = 400

// A hand on the region: from here on, the position is the user's and not the restore's.
const HandEvents = ['wheel', 'pointerdown', 'keydown', 'touchstart'] as const

interface Memory {
  address: EntryAddress
  region: string
  pending: boolean
  timer: ReturnType<typeof setTimeout> | undefined
  stop: () => void
}

const memories = new WeakMap<HTMLElement, Memory>()

export const vScrollMemory: Directive<HTMLElement, string> = {
  mounted(el, binding) {
    const tabs = useTabsStore()
    const tab = tabs.active
    if (!tab) return
    const region = binding.value
    const address: EntryAddress = {
      tabId: tab.id,
      index: tab.index,
      location: tabLocation(tab),
    }
    const target = entryScroll(tab)[region]

    const save = (): void => {
      if (!memory.pending) tabs.setScroll(address, region, el.scrollTop)
    }
    const onScroll = (): void => {
      clearTimeout(memory.timer)
      memory.timer = setTimeout(save, Timing.ViewWrite)
    }

    // Content arrives as nodes (a `v-if` turning true) or as growth (rows filling in); either one
    // is a moment the position may have become reachable.
    const content = new MutationObserver(() => attempt())
    const growth = new ResizeObserver(() => attempt())
    let quiet: ReturnType<typeof setTimeout> | undefined
    const settle = (): void => {
      memory.pending = false
      content.disconnect()
      growth.disconnect()
      clearTimeout(giveUp)
      clearTimeout(quiet)
      for (const kind of HandEvents) el.removeEventListener(kind, settle)
    }
    const attempt = (): void => {
      if (!memory.pending) return
      const step = restoreStep(target, el)
      if (step.kind === RestoreStep.Wait) return
      if (step.kind === RestoreStep.Done) {
        settle()
        return
      }
      el.scrollTop = step.top
      clearTimeout(quiet)
      quiet = setTimeout(settle, QuietMs)
    }

    const memory: Memory = {
      address,
      region,
      pending: true,
      timer: undefined,
      stop: () => {
        settle()
        el.removeEventListener('scroll', onScroll)
        clearTimeout(memory.timer)
      },
    }
    memories.set(el, memory)

    const giveUp = setTimeout(settle, GiveUpMs)
    for (const kind of HandEvents)
      el.addEventListener(kind, settle, { passive: true })
    el.addEventListener('scroll', onScroll, { passive: true })
    content.observe(el, { childList: true, subtree: true, characterData: true })
    growth.observe(el)
    if (el.firstElementChild) growth.observe(el.firstElementChild)
    attempt()
  },
  beforeUnmount(el) {
    const memory = memories.get(el)
    if (!memory) return
    // The last position, written while the element is still there to be read — unless it never
    // got back to where it was, in which case the stored one is the truer of the two.
    if (!memory.pending)
      useTabsStore().setScroll(memory.address, memory.region, el.scrollTop)
    memory.stop()
    memories.delete(el)
  },
}
