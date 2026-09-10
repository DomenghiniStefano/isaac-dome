import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { StoreId } from '@/lib/constants/stores'
import { defaultLocation } from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'
import {
  closeTab,
  firstState,
  moveTab,
  navigateTab,
  openTab,
  selectTab,
} from './tabModel'
import type { Tab } from './tabModel'

// The open tabs, window-wide. The rules are tabModel's; this holds the result. Nothing is
// saved yet: tabs surviving a restart is sub-project 7.
export const useTabsStore = defineStore(StoreId.Tabs, () => {
  let counter = 0
  const nextId = (): string => `tab-${++counter}`
  const fresh = (): Tab => ({ id: nextId(), location: defaultLocation })

  const state = ref(firstState(nextId(), defaultLocation))

  const tabs = computed(() => state.value.tabs)
  const activeId = computed(() => state.value.activeId)
  const active = computed(() =>
    state.value.tabs.find((tab) => tab.id === state.value.activeId),
  )

  const open = (location: TabLocation = defaultLocation): void => {
    state.value = openTab(state.value, nextId(), location)
  }
  const select = (id: string): void => {
    state.value = selectTab(state.value, id)
  }
  const close = (id: string): void => {
    state.value = closeTab(state.value, id, fresh)
  }
  const move = (from: number, to: number): void => {
    state.value = moveTab(state.value, from, to)
  }
  const navigate = (location: TabLocation): void => {
    state.value = navigateTab(state.value, location)
  }

  return { tabs, activeId, active, open, select, close, move, navigate }
})
