<script setup lang="ts">
import { computed } from 'vue'
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import { useMessages } from '@/i18n'
import type { CollectionItem } from '@/lib/ipc/types'

// The first pool an item is in, and how many more.
const props = defineProps<{ item: CollectionItem }>()
const { t } = useMessages()

const firstPool = computed(() => props.item.pools[0] ?? null)
const morePools = computed(() => Math.max(0, props.item.pools.length - 1))
</script>

<template>
  <span class="flex min-w-0 items-center gap-2">
    <template v-if="firstPool">
      <span class="truncate text-caption text-foreground">{{ firstPool }}</span>
      <span
        v-if="morePools > 0"
        class="shrink-0 text-label text-subtle-foreground tabular-nums"
        >+{{ morePools }}</span
      >
    </template>
    <EmptyValue v-else>{{ t('collection.poolNone') }}</EmptyValue>
  </span>
</template>
