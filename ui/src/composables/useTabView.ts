import { useDebounceFn } from '@vueuse/core'
import { ref, watch } from 'vue'
import type { Ref } from 'vue'
import { Timing } from '@/lib/constants/timing'
import type { TabViewSpec } from '@/lib/tabs/tabView'
import { entryView } from '@/stores/tabModel'
import { useTabsStore } from '@/stores/tabs'

// How a screen remembers how it was being read, and the only way it does: a screen never reaches
// into the tab store, exactly as it never calls a Tauri command by hand. What comes back is an
// ordinary ref — the screen writes to it the way it wrote to its own — and where it lands is
// this composable's business.
//
// The debounce is for the burst, not for the cost: a facet click moves a pick and a count in the
// same breath.
//
// `update` moves part of the reading and keeps the rest, which is how a screen writes one field
// of it — a query, a scroll offset, a selection.
export const useTabView = <T extends object>(
  spec: TabViewSpec<T>,
): { reading: Ref<T>; update: (patch: Partial<T>) => void } => {
  const tabs = useTabsStore()

  const current = (): T => {
    const tab = tabs.active
    return (tab ? spec.read(entryView(tab)) : null) ?? spec.empty()
  }

  const reading = ref(current()) as Ref<T>

  // Which entry we are on: the tab, and its position in its own history. A tab going back to the
  // same route keeps the screen mounted, so the component's lifecycle cannot notice the move —
  // this can.
  const entryId = (): string => `${tabs.activeId}#${tabs.active?.index ?? 0}`

  let at = entryId()
  watch(entryId, (id) => {
    at = id
    reading.value = current()
  })

  const remember = useDebounceFn(() => {
    const location = tabs.location
    // The entry moved under the write while it waited. `setEntryView` would refuse it anyway;
    // this only saves asking.
    if (!location || entryId() !== at) return
    tabs.setView(location, reading.value)
  }, Timing.ViewWrite)

  watch(reading, () => void remember(), { deep: true })

  const update = (patch: Partial<T>): void => {
    reading.value = { ...reading.value, ...patch }
  }

  return { reading, update }
}
