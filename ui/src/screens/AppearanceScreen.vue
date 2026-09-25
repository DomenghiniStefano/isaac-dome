<script setup lang="ts">
import { vScrollMemory } from '@/directives/scrollMemory'
import { SlidersHorizontalIcon, TriangleAlertIcon } from '@lucide/vue'
import {
  Alert,
  AlertDescription,
  AlertTitle,
  AlertLive,
  AlertVariant,
} from '@/components/ui/alert'
import { useMessages } from '@/i18n'
import { useSettingsStore } from '@/stores/settings'
import ScreenHeader from '@/components/screen/ScreenHeader.vue'
import ScalePreview from './appearance/ScalePreview.vue'
import ScaleSlider from './appearance/ScaleSlider.vue'

const settings = useSettingsStore()
const { t } = useMessages()
</script>

<template>
  <div
    v-scroll-memory="'page'"
    class="flex h-full flex-col gap-4 overflow-y-auto px-5.5 pt-5 pb-15"
  >
    <ScreenHeader
      :icon="SlidersHorizontalIcon"
      :title="t('routes.appearance')"
      >{{ t('appearance.intro') }}</ScreenHeader
    >
    <ScalePreview />
    <!-- The size is applied first and saved after, so a failed write leaves the interface
         where the user put it and says what didn't happen. -->
    <Alert
      v-if="settings.saveFailed"
      :variant="AlertVariant.Destructive"
      :live="AlertLive.Assertive"
    >
      <TriangleAlertIcon />
      <AlertTitle>{{ t('appearance.saveFailedTitle') }}</AlertTitle>
      <AlertDescription>{{ t('appearance.saveFailed') }}</AlertDescription>
    </Alert>
    <ScaleSlider :percent="settings.scale" @pick="settings.setScale" />
  </div>
</template>
