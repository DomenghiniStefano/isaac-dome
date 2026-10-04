<script setup lang="ts">
import { computed } from 'vue'
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import QueueActionButton from '@/components/plan/QueueActionButton.vue'
import EntityChip from '@/components/runs/EntityChip.vue'
import { GridTable } from '@/components/ui/grid-table'
import { useMessages } from '@/i18n'
import type { LiveOpen } from '@/lib/ipc/types'
import { opensRows } from '@/lib/live/opensRows'
import type { OpenRow } from '@/lib/live/opensRows'
import type { QueueTarget } from '@/lib/plan/queueAction'
import { liveColumns } from '@/lib/table/columns'

// Everything the run could open, in one table instead of a card per cell: the cell is a
// column, so the boss is still said once per row and the rows can be read against each other
// — which a stack of cards cannot do. Live lists only what is missing, so nothing here is done.
const props = defineProps<{ opens: LiveOpen[] }>()
const { t } = useMessages()

const rows = computed(() => opensRows(props.opens, t))

const target = (row: OpenRow): QueueTarget | null =>
  row.achievement === null
    ? null
    : { achievement: row.achievement, done: false }
</script>

<template>
  <GridTable :columns="liveColumns" :rows="rows" :row-key="(row) => row.key">
    <template #cell-achievement="{ row }">
      <EntityChip
        :target="row.target"
        :name="row.name"
        :detail="row.condition"
        :icon-url="row.iconUrl"
      />
    </template>
    <template #cell-cell="{ row }">
      <span class="truncate text-row">{{ row.cell }}</span>
    </template>
    <template #cell-condition="{ row }">
      <span
        v-if="row.condition"
        class="line-clamp-2 text-label text-subtle-foreground"
        >{{ row.condition }}</span
      >
      <EmptyValue v-else>{{ t('live.noCondition') }}</EmptyValue>
    </template>
    <template #cell-opens="{ row }">
      <span class="text-label tabular-nums">{{
        row.fanOut > 0 ? row.fanOut : t('live.opensNothingMore')
      }}</span>
    </template>
    <template #actions="{ row }">
      <QueueActionButton :target="target(row)" />
    </template>
  </GridTable>
</template>
