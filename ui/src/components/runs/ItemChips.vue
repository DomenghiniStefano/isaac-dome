<script setup lang="ts">
import PixelSprite from '@/components/sprite/PixelSprite.vue'
import { useMessages } from '@/i18n'
import type { RunItemRef } from '@/lib/ipc/types'

// The run's items as the game draws them. An item the catalog cannot name is still drawn —
// by its number — because the id is the one thing the log gave us and a blank would hide it.
defineProps<{ items: RunItemRef[]; held: RunItemRef | null }>()
const { t } = useMessages()

const isHeld = (item: RunItemRef, held: RunItemRef | null) =>
  held !== null && held.id === item.id
</script>

<template>
  <div v-if="items.length > 0" class="flex flex-wrap gap-2">
    <span
      v-for="(item, i) in items"
      :key="`${item.id}-${i}`"
      class="flex items-center gap-1.5 rounded-input border px-2 py-1"
      :class="isHeld(item, held) ? 'border-highlight' : 'border-hairline'"
    >
      <PixelSprite
        :url="item.iconUrl"
        placeholder
        class="size-icon-compact shrink-0"
      />
      <span class="text-label">{{
        item.name ?? t('runs.unnamedItem', { id: item.id })
      }}</span>
    </span>
  </div>
</template>
