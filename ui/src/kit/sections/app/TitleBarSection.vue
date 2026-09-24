<script setup lang="ts">
import { ref } from 'vue'
import TitleBar from '@/components/shell/TitleBar.vue'
import type { TabView } from '@/lib/shell/tabs'
import { TabOrigin } from '@/lib/shell/tabs'
import KitSection from '../../KitSection.vue'

const tabs = ref<TabView[]>([
  { id: 'd6', label: 'The D6', origin: TabOrigin.Wiki },
  { id: 'steps', label: 'Prossimi passi', origin: TabOrigin.Progress },
  { id: 'profile', label: 'Profilo di gioco', origin: TabOrigin.Settings },
])
const active = ref<string | null>('d6')
let added = 0

const crowded: TabView[] = [
  'The D6',
  'False PHD',
  'Isaac',
  'Monstro',
  'Passi',
  'Unlock',
  'Run',
  'Profilo',
  'Tab',
].map((label, i) => ({
  id: `crowded-${i}`,
  label,
  origin: i < 4 ? TabOrigin.Wiki : TabOrigin.Progress,
}))

const move = (from: number, to: number) => {
  const next = [...tabs.value]
  const [tab] = next.splice(from, 1)
  if (tab) next.splice(to, 0, tab)
  tabs.value = next
}

const close = (id: string) => {
  tabs.value = tabs.value.filter((tab) => tab.id !== id)
}

const add = () => {
  added += 1
  const id = `new-${added}`
  tabs.value = [
    ...tabs.value,
    { id, label: `Nuova ${added}`, origin: TabOrigin.Settings },
  ]
  active.value = id
}
</script>

<template>
  <KitSection title="TitleBar" class="col-span-3">
    <TitleBar
      :tabs="tabs"
      :active-id="active"
      :focused="true"
      @select="active = $event"
      @close="close"
      @move="move"
      @add="add"
    />
    <TitleBar :tabs="tabs" :active-id="active" :focused="false" />
    <TitleBar
      :tabs="crowded"
      active-id="crowded-0"
      :focused="true"
      class="w-160"
    />
  </KitSection>
</template>
