<script setup lang="ts" generic="T">
import { computed, ref } from 'vue'
import { VirtualRows } from '@/components/ui/virtual'
import { useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import type { ScrollOffset } from '@/lib/scale/scrollOffset'
import {
  ACTIONS_KEY,
  RowHeight,
  cellClass,
  cellStyle,
  rowHeightClass,
  rowHeightPx,
  withActions,
} from '@/lib/table/gridColumn'
import type { GridColumn } from '@/lib/table/gridColumn'
import GridRow from './GridRow.vue'

// The one way a list is drawn as a table: the leather band pinned to the page box's top, a row
// per item, stripes, hover, and a cell per declared column that keeps to its track whatever it
// holds. The columns are the screen's (`lib/table/columns.ts`); what is in a cell is the
// screen's too, through `#cell-<key>`. Every table ends with Actions, filled by `#actions`.
const props = withDefaults(
  defineProps<{
    columns: readonly GridColumn[]
    rows: T[]
    rowKey: (row: T) => string | number
    rowHeight?: RowHeight
    virtual?: boolean
    clickable?: boolean
    offset?: ScrollOffset | null
  }>(),
  {
    rowHeight: RowHeight.Wide,
    virtual: false,
    clickable: false,
    offset: null,
  },
)
const emit = defineEmits<{
  rowClick: [row: T, event: MouseEvent]
  offsetChange: [ScrollOffset]
}>()
defineSlots<
  {
    actions(props: { row: T }): unknown
  } & {
    [cell: `cell-${string}`]: (props: { row: T; index: number }) => unknown
  } & {
    [head: `head-${string}`]: (props: Record<string, never>) => unknown
  }
>()
const { t } = useMessages()

const all = computed(() => withActions(props.columns))

const rowClass = (index: number): string =>
  cn(
    'border-b border-hairline hover:bg-row-hover',
    rowHeightClass[props.rowHeight],
    index % 2 === 1 && 'bg-row-alt',
    props.clickable && 'cursor-pointer',
  )

const click = (row: T, event: MouseEvent): void => {
  if (props.clickable) emit('rowClick', row, event)
}

const slotName = (key: string): 'actions' | `cell-${string}` =>
  key === ACTIONS_KEY ? ACTIONS_KEY : `cell-${key}`

// The find bar moves to a row by index; only the virtualizer can make that row exist.
const list = ref<{ scrollToIndex: (index: number) => void } | null>(null)
defineExpose({
  scrollToIndex: (index: number) => list.value?.scrollToIndex(index),
})
</script>

<template>
  <div class="flex flex-col">
    <div
      class="sticky top-0 z-raised-header flex border-b border-hairline bg-band text-label text-band-foreground"
    >
      <span
        v-for="column in all"
        :key="column.key"
        :class="cn(cellClass(column), 'py-1.5')"
        :style="cellStyle(column)"
      >
        <slot :name="`head-${column.key}`">
          <span v-if="column.header" class="truncate">{{
            t(column.header)
          }}</span>
        </slot>
      </span>
    </div>
    <VirtualRows
      v-if="virtual"
      ref="list"
      v-slot="{ visible }"
      :rows="rows"
      :row-px="rowHeightPx[rowHeight]"
      :offset="offset"
      @offset-change="emit('offsetChange', $event)"
    >
      <GridRow
        v-for="{ index, style, row } in visible"
        :key="rowKey(row)"
        :style="style"
        :columns="all"
        :row-class="
          cn(
            rowClass(index),
            'absolute inset-x-0 top-0 translate-y-(--row-start)',
          )
        "
        @click="click(row, $event)"
      >
        <template v-for="column in all" :key="column.key" #[column.key]>
          <slot :name="slotName(column.key)" :row="row" :index="index" />
        </template>
      </GridRow>
    </VirtualRows>
    <template v-else>
      <GridRow
        v-for="(row, index) in rows"
        :key="rowKey(row)"
        :columns="all"
        :row-class="rowClass(index)"
        @click="click(row, $event)"
      >
        <template v-for="column in all" :key="column.key" #[column.key]>
          <slot :name="slotName(column.key)" :row="row" :index="index" />
        </template>
      </GridRow>
    </template>
  </div>
</template>
