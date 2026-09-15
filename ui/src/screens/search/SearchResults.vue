<script setup lang="ts">
import { VirtualRows } from '@/components/ui/virtual'
import type { ScrollOffset } from '@/lib/scale/scrollOffset'
import SearchRow from '@/components/search/SearchRow.vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { cn } from '@/lib/cn'
import { rowResultPx } from '@/lib/scale/rows'
import type { SearchRow as Row } from '@/lib/search/rows'

defineProps<{ rows: Row[]; offset: ScrollOffset | null }>()
const emit = defineEmits<{
  open: [row: Row, event: MouseEvent]
  offsetChange: [offset: ScrollOffset]
}>()

// `text-left` is not decoration: the row is a pressable primitive, which the user agent
// centres, and a row of text has to read from its left edge like every other table's rows.
const rowClass =
  'absolute inset-x-0 top-0 h-row-result translate-y-(--row-start) border-x-0 border-t-0 border-b border-hairline px-3 text-left hover:bg-row-hover'
</script>

<template>
  <VirtualRows
    v-slot="{ visible }"
    :rows="rows"
    :row-px="rowResultPx"
    :offset="offset"
    @offset-change="emit('offsetChange', $event)"
  >
    <Button
      v-for="{ index, style, row } in visible"
      :key="row.key"
      :variant="ButtonVariant.Ghost"
      :size="ButtonSize.Row"
      :style="style"
      :class="cn(rowClass, index % 2 === 1 && 'bg-row-alt')"
      @click="emit('open', row, $event)"
    >
      <SearchRow :row="row" />
    </Button>
  </VirtualRows>
</template>
