<script setup lang="ts">
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import PixelSprite from '@/components/sprite/PixelSprite.vue'
import { useMessages } from '@/i18n'
import type { RunView } from '@/lib/ipc/types'

defineProps<{ run: RunView }>()
const { t } = useMessages()
</script>

<template>
  <!-- The face keeps its box when there is none to draw — no catalog, a form the co-op menu
       does not draw — so the names stay in one column down the list. -->
  <span class="flex min-w-0 items-center gap-2">
    <PixelSprite
      :url="run.characterHeadUrl"
      placeholder
      class="size-icon-compact shrink-0"
    />
    <span v-if="run.character !== null" class="truncate text-row">{{
      run.character
    }}</span>
    <EmptyValue v-else>{{ t('runs.noCharacter') }}</EmptyValue>
  </span>
</template>
