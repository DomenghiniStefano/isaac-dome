<script setup lang="ts">
import { cn } from '@/lib/cn'
import { ACTIONS_KEY, cellClass, cellStyle } from '@/lib/table/gridColumn'
import type { GridColumn } from '@/lib/table/gridColumn'

// One row of a `GridTable`: a cell per column, each wrapped in the containment, the content
// handed down from the table's own slots. The Actions cell keeps its clicks: in a clickable row
// a button there must not also open the row — a Ctrl-click would open two tabs.
defineProps<{
  columns: GridColumn[]
  rowClass: string
}>()
const emit = defineEmits<{ click: [event: MouseEvent] }>()

const keepOnActions = (column: GridColumn, event: MouseEvent): void => {
  if (column.key === ACTIONS_KEY) event.stopPropagation()
}
</script>

<template>
  <div :class="cn('flex', rowClass)" @click="emit('click', $event)">
    <span
      v-for="column in columns"
      :key="column.key"
      :class="cellClass(column)"
      :style="cellStyle(column)"
      @click="keepOnActions(column, $event)"
    >
      <slot :name="column.key" />
    </span>
  </div>
</template>
