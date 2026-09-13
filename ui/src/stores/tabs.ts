import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { StoreId } from '@/lib/constants/stores'
import { defaultLocation } from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'
import {
  backTab,
  canGoBack,
  canGoForward,
  closeTab,
  firstState,
  forwardTab,
  moveTab,
  navigateTab,
  openTab,
  refineTab,
  selectTab,
  tabLocation,
} from './tabModel'
import type { Tab } from './tabModel'

// The open tabs, window-wide. The rules are tabModel's; this holds the result. Nothing is
// saved yet: tabs surviving a restart is sub-project 7.
export const useTabsStore = defineStore(StoreId.Tabs, () => {
  let counter = 0
  const nextId = (): string => `tab-${++counter}`
  const fresh = (): Tab => ({
    id: nextId(),
    entries: [defaultLocation],
    index: 0,
  })

  const state = ref(firstState(nextId(), defaultLocation))

  const tabs = computed(() => state.value.tabs)
  const activeId = computed(() => state.value.activeId)
  const active = computed(() =>
    state.value.tabs.find((tab) => tab.id === state.value.activeId),
  )
  // Where the active tab is: the entry its history is showing, not the last one it reached.
  const location = computed(() =>
    active.value ? tabLocation(active.value) : undefined,
  )
  const canBack = computed(() => canGoBack(active.value))
  const canForward = computed(() => canGoForward(active.value))

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
  const refine = (location: TabLocation): void => {
    state.value = refineTab(state.value, location)
  }
  const back = (): void => {
    state.value = backTab(state.value)
  }
  const forward = (): void => {
    state.value = forwardTab(state.value)
  }

  return {
    tabs,
    activeId,
    active,
    location,
    canBack,
    canForward,
    open,
    select,
    close,
    move,
    navigate,
    refine,
    back,
    forward,
  }
})
