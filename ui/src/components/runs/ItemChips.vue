<script setup lang="ts">
import { useMessages } from '@/i18n'
import type { RunItemRef } from '@/lib/ipc/types'
import EntityChip from './EntityChip.vue'

// The run's items as the game draws them, each opening its wiki page. An item the catalog
// cannot name is still drawn — by its number — because the id is the one thing the log gave
// us and a blank would hide it.
defineProps<{ items: RunItemRef[]; held: RunItemRef | null }>()
const { t } = useMessages()

const isHeld = (item: RunItemRef, held: RunItemRef | null) =>
  held !== null && held.id === item.id
</script>

<template>
  <div v-if="items.length > 0" class="flex flex-wrap gap-2">
    <EntityChip
      v-for="(item, i) in items"
      :key="`${item.id}-${i}`"
      :target="{ kind: 'item', id: item.id }"
      :name="item.name ?? t('runs.unnamedItem', { id: item.id })"
      :detail="t('runs.itemId', { id: item.id })"
      :icon-url="item.iconUrl"
      :highlighted="isHeld(item, held)"
    />
  </div>
</template>
