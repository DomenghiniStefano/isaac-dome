<script setup lang="ts">
import { computed } from 'vue'
import EntityChip from '@/components/runs/EntityChip.vue'
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import { useMessages } from '@/i18n'
import { opensRows } from '@/lib/live/opensRows'
import type { LiveOpen } from '@/lib/ipc/types'

// Everything the run could open, in one table instead of a card per cell: the cell is a
// column, so the boss is still said once per row and the rows can be read against each other
// — which a stack of cards cannot do.
const props = defineProps<{ opens: LiveOpen[] }>()
const { t } = useMessages()

const rows = computed(() => opensRows(props.opens, t))
</script>

<template>
  <div class="flex flex-col">
    <div
      class="grid grid-cols-live-opens items-center border-b border-hairline bg-muted text-label text-subtle-foreground"
    >
      <span class="px-2 py-1.5">{{ t('live.column.achievement') }}</span>
      <span class="px-2 py-1.5">{{ t('live.column.cell') }}</span>
      <span class="px-2 py-1.5">{{ t('live.column.condition') }}</span>
      <span class="px-2 py-1.5 text-right">{{ t('live.column.opens') }}</span>
    </div>
    <div
      v-for="(row, i) in rows"
      :key="row.key"
      class="grid grid-cols-live-opens items-center border-b border-hairline"
      :class="i % 2 === 1 ? 'bg-row-alt' : undefined"
    >
      <span class="px-2 py-1">
        <EntityChip
          :target="row.target"
          :name="row.name"
          :detail="row.condition"
          :icon-url="row.iconUrl"
        />
      </span>
      <span class="truncate px-2 text-row">{{ row.cell }}</span>
      <span class="px-2 text-label text-subtle-foreground">
        <span v-if="row.condition" class="line-clamp-2">{{
          row.condition
        }}</span>
        <EmptyValue v-else>{{ t('live.noCondition') }}</EmptyValue>
      </span>
      <span class="px-2 text-right text-label tabular-nums">{{
        row.fanOut > 0 ? row.fanOut : t('live.opensNothingMore')
      }}</span>
    </div>
  </div>
</template>
