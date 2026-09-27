<script setup lang="ts">
import { computed, ref } from 'vue'
import { VirtualRows } from '@/components/ui/virtual'
import type { WikiPageRef } from '@/lib/ipc/types'
import { rowWikiCardPx } from '@/lib/scale/rows'
import type { ScrollOffset } from '@/lib/scale/scrollOffset'
import { pageKey } from '@/lib/wiki/pageKey'
import WikiListCard from './WikiListCard.vue'
import { useCardColumns } from './useCardColumns'

// The card grid, virtualized **by row of cards**: `VirtualRows` only
// ever measures rows, the same component the list's own table and Unlock already use, so the
// pages are chunked into rows here, as many wide as `useCardColumns` says the box holds.
const props = defineProps<{
  pages: WikiPageRef[]
  offset: ScrollOffset | null
}>()
const emit = defineEmits<{
  offsetChange: [ScrollOffset]
  open: [page: WikiPageRef, event: MouseEvent]
}>()

const box = ref<HTMLElement | null>(null)
const columns = useCardColumns(box)

const rows = computed(() => {
  const perRow = columns.value
  const chunks: WikiPageRef[][] = []
  for (let at = 0; at < props.pages.length; at += perRow) {
    chunks.push(props.pages.slice(at, at + perRow))
  }
  return chunks
})

// The row's own template: as many tracks as `useCardColumns` measured, each at least a card
// wide and sharing whatever room is left. Set as a variable and read by `grid-cols-(--…)` in
// the template — never a class built from a string — the same rule `WikiFigure`'s own
// `--figure-sprite-size` follows.
const rowVars = (start: string) => ({
  '--row-start': start,
  '--wiki-grid-columns': `repeat(${columns.value}, minmax(var(--spacing-wiki-card), 1fr))`,
})
</script>

<template>
  <div ref="box" class="flex min-h-0 flex-1 flex-col">
    <VirtualRows
      v-if="rows.length > 0"
      v-slot="{ visible }"
      :rows="rows"
      :row-px="rowWikiCardPx"
      :offset="offset"
      @offset-change="emit('offsetChange', $event)"
    >
      <div
        v-for="{ index, style, row } in visible"
        :key="index"
        :style="{ ...style, ...rowVars(style['--row-start']) }"
        class="absolute inset-x-0 top-0 grid translate-y-(--row-start) grid-cols-(--wiki-grid-columns) gap-3 px-3 pb-3"
      >
        <WikiListCard
          v-for="page in row"
          :key="pageKey(page.target) ?? page.title"
          :page="page"
          @open="(event) => emit('open', page, event)"
        />
      </div>
    </VirtualRows>
  </div>
</template>
