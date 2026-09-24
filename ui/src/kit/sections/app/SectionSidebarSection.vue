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
import SidebarEdgeTab from '@/components/shell/SidebarEdgeTab.vue'
import SidebarItem from '@/components/shell/SidebarItem.vue'
import { SidebarWidth } from '@/components/shell/sidebarWidth'
import KitSection from '../../KitSection.vue'

const progressWidth = ref<number>(SidebarWidth.Default)
const progressFolded = ref(false)
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
      <!-- The asked-for fold (card #54): a 600px `shell`, above --container-sidebar-room, so the
           tab on the edge has room to open it again. -->
      <div
        :data-sidebar="progressFolded ? 'collapsed' : undefined"
        class="group/shell @container/shell flex w-150 items-start"
      >
        <SectionSidebar
          v-model:width="progressWidth"
          title="Progressi"
          hint="Dipende dal profilo attivo."
          class="min-h-48"
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
          <template #edge>
            <SidebarEdgeTab
              :collapsed="progressFolded"
              @toggle="progressFolded = !progressFolded"
            />
          </template>
        </SectionSidebar>
      </div>
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
