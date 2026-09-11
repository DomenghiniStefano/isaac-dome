<script setup lang="ts">
import {
  CogIcon,
  Grid2x2Icon,
  ListXIcon,
  SaveIcon,
  StarIcon,
} from '@lucide/vue'
import { ref } from 'vue'
import SectionSidebar from '@/components/shell/SectionSidebar.vue'
import SidebarItem from '@/components/shell/SidebarItem.vue'
import { SidebarWidth } from '@/components/shell/sidebarWidth'
import KitSection from '../../KitSection.vue'

const progressWidth = ref<number>(SidebarWidth.Default)
const settingsWidth = ref<number>(SidebarWidth.Default)
const current = ref('completion')
const progressItems = [
  { id: 'steps', label: 'Prossimi passi', icon: ListXIcon },
  { id: 'completion', label: 'Completamento', icon: Grid2x2Icon },
  { id: 'unlock', label: 'Unlock', icon: StarIcon },
]
</script>

<template>
  <KitSection title="SectionSidebar" class="col-span-2">
    <div class="flex items-start gap-4">
      <SectionSidebar
        v-model:width="progressWidth"
        title="Progressi"
        hint="Dipende dal profilo attivo."
      >
        <template #icon><ListXIcon /></template>
        <SidebarItem
          v-for="item in progressItems"
          :key="item.id"
          :active="item.id === current"
          @click="current = item.id"
        >
          <template #icon><component :is="item.icon" /></template>
          {{ item.label }}
        </SidebarItem>
      </SectionSidebar>
      <SectionSidebar
        v-model:width="settingsWidth"
        title="Impostazioni"
        hint="Come l'app trova gioco e salvataggi."
      >
        <template #icon><CogIcon /></template>
        <SidebarItem :active="false">
          <template #icon><SaveIcon /></template>
          Profilo di gioco
        </SidebarItem>
        <SidebarItem :active="true">
          <template #icon><Grid2x2Icon /></template>
          Tab
        </SidebarItem>
      </SectionSidebar>
    </div>
  </KitSection>
</template>
