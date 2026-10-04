<script setup lang="ts">
import { cn } from '@/lib/cn'
import { ACTIONS_KEY, cellClass, cellStyle } from '@/lib/table/gridColumn'
import type { GridColumn } from '@/lib/table/gridColumn'

// One row of a `GridTable`: a cell per column, each wrapped in the containment, the content
// handed down from the table's own slots. A control in the Actions cell keeps its click: in a
// clickable row it must not also open the row — a Ctrl-click would open two tabs. The cell's
// empty room around it is still the row.
defineProps<{
  columns: GridColumn[]
  rowClass: string
}>()
const emit = defineEmits<{ click: [event: MouseEvent] }>()

const keepOnActions = (column: GridColumn, event: MouseEvent): void => {
  const control =
    event.target instanceof Element &&
    event.target.closest('button, a, [role="button"]') !== null
  if (column.key === ACTIONS_KEY && control) event.stopPropagation()
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
