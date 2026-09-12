<script setup lang="ts">
import { computed, ref } from 'vue'
import SearchRow from '@/components/search/SearchRow.vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { cn } from '@/lib/cn'
import { useScaledRows } from '@/composables/useScaledRows'
import { rowResultPx } from '@/lib/scale/rows'
import type { SearchRow as Row } from '@/lib/search/rows'

const props = defineProps<{ rows: Row[] }>()
const emit = defineEmits<{ open: [row: Row, event: MouseEvent] }>()

const scroller = ref<HTMLElement | null>(null)

// Up to 300 rows: drawn as many as fit plus a margin, positioned with the same number the
// row's own token is measured from, so no scale leaves them overlapping.
const virtualizer = useScaledRows({
  count: () => props.rows.length,
  scroller,
  rowPx: rowResultPx,
})

const visible = computed(() =>
  virtualizer.value.getVirtualItems().flatMap((item) => {
    const row = props.rows[item.index]
    return row ? [{ item, row }] : []
  }),
)

// `text-left` is not decoration: the row is a pressable primitive, which the user agent
// centres, and a row of text has to read from its left edge like every other table's rows.
const rowClass =
  'absolute inset-x-0 top-0 h-row-result translate-y-(--row-start) border-x-0 border-t-0 border-b border-hairline px-3 text-left hover:bg-row-hover'

// Geometry measured at runtime travels as CSS variables, read by utilities.
const body = computed(() => ({
  '--search-total': `${virtualizer.value.getTotalSize()}px`,
}))
const rowStart = (start: number) => ({ '--row-start': `${start}px` })
</script>

<template>
  <div ref="scroller" class="max-h-unlock-body overflow-auto">
    <div :style="body" class="relative h-(--search-total)">
      <Button
        v-for="{ item, row } in visible"
        :key="row.key"
        :variant="ButtonVariant.Ghost"
        :size="ButtonSize.Row"
        :style="rowStart(item.start)"
        :class="cn(rowClass, item.index % 2 === 1 && 'bg-row-alt')"
        @click="emit('open', row, $event)"
      >
        <SearchRow :row="row" />
      </Button>
    </div>
  </div>
</template>
