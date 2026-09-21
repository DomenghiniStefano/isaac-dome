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
          :label="item.label"
          @click="current = item.id"
        >
          <template #icon><component :is="item.icon" /></template>
        </SidebarItem>
      </SectionSidebar>
      <!-- The collapsed state, drawn without touching the window: a 384px `shell` container is
           below --container-compact, so what is inside it folds for real. That is what container
           queries buy and media queries could not (spec 3.13a §11). -->
      <div class="group/shell @container/shell flex w-96 items-start">
        <SectionSidebar
          v-model:width="settingsWidth"
          title="Impostazioni"
          hint="Come l'app trova gioco e salvataggi."
        >
          <template #icon><CogIcon /></template>
          <SidebarItem :active="false" label="Profilo di gioco">
            <template #icon><SaveIcon /></template>
          </SidebarItem>
          <SidebarItem :active="true" label="Tab">
            <template #icon><Grid2x2Icon /></template>
          </SidebarItem>
        </SectionSidebar>
      </div>
    </div>
  </KitSection>
</template>
