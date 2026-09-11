<script setup lang="ts">
import TabStrip from './TabStrip.vue'
import WindowControls from './WindowControls.vue'
import type { TabView } from './tabs'

defineProps<{
  tabs: TabView[]
  activeId: string | null
  focused: boolean
}>()
const emit = defineEmits<{
  select: [id: string]
  close: [id: string]
  move: [from: number, to: number]
  add: []
  minimize: []
  toggleMaximize: []
  closeWindow: []
}>()
</script>

<template>
  <!-- Which pages are open (Chrome e Stati.dc.html, "Finestra"). The drag region is an
       element with a minimum width, not the space left over: with nine tabs the names
       truncate and the region stays whole. Unfocused, accents dim and data doesn't. -->
  <header
    :data-focused="focused"
    class="group flex h-titlebar items-stretch border-b border-hairline bg-titlebar"
  >
    <TabStrip
      :tabs="tabs"
      :active-id="activeId"
      @select="emit('select', $event)"
      @close="emit('close', $event)"
      @move="(from, to) => emit('move', from, to)"
      @add="emit('add')"
    />
    <div data-tauri-drag-region class="min-w-drag-region flex-1" />
    <WindowControls
      @minimize="emit('minimize')"
      @toggle-maximize="emit('toggleMaximize')"
      @close-window="emit('closeWindow')"
    />
  </header>
</template>
