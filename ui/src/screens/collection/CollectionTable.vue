<script setup lang="ts">
import { ref } from 'vue'
import { VirtualRows } from '@/components/ui/virtual'
import type { ScrollOffset } from '@/lib/scale/scrollOffset'
import { useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import type { CollectionItem } from '@/lib/ipc/types'
import { rowWidePx } from '@/lib/scale/rows'
import CollectionRow from './CollectionRow.vue'

defineProps<{
  items: CollectionItem[]
  offset: ScrollOffset | null
  /** What the find bar is looking for, so a row can paint it (B67). */
  findQuery: string
  /** The id of the match the bar is standing on. */
  findCurrent: string | null
}>()
const emit = defineEmits<{ offsetChange: [offset: ScrollOffset] }>()
const { t } = useMessages()

// The find bar hands back an index; moving there is the virtualizer's job and the screen
// cannot reach it, so the table passes the call through.
// Structural and not `InstanceType`: `VirtualRows` is generic, so it has no instance type to
// take — and the one thing wanted from it is the one call named here.
const rows = ref<{ scrollToIndex: (index: number) => void } | null>(null)
defineExpose({
  scrollToIndex: (index: number) => rows.value?.scrollToIndex(index),
})
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col">
    <div
      class="grid grid-cols-collection items-center border-b border-hairline bg-muted text-label text-subtle-foreground @max-compact/page:grid-cols-collection-narrow"
    >
      <span />
      <span class="px-2 py-1.5">{{ t('collection.columns.item') }}</span>
      <span class="px-2 py-1.5 @max-compact/page:hidden">{{
        t('collection.columns.quality')
      }}</span>
      <span class="px-2 py-1.5 @max-compact/page:hidden">{{
        t('collection.columns.pools')
      }}</span>
      <span class="px-2 py-1.5 @max-compact/page:hidden">{{
        t('collection.columns.origin')
      }}</span>
      <span class="px-2 py-1.5">{{ t('collection.columns.state') }}</span>
    </div>
    <VirtualRows
      ref="rows"
      v-slot="{ visible }"
      :rows="items"
      :row-px="rowWidePx"
      :offset="offset"
      @offset-change="emit('offsetChange', $event)"
    >
      <div
        v-for="{ index, style, row: item } in visible"
        :key="item.id"
        :style="style"
        :class="
          cn(
            'absolute inset-x-0 top-0 grid h-row-wide translate-y-(--row-start) grid-cols-collection items-center border-b border-hairline hover:bg-row-hover @max-compact/page:grid-cols-collection-narrow',
            index % 2 === 1 && 'bg-row-alt',
          )
        "
      >
        <CollectionRow
          :item="item"
          :find-query="findQuery"
          :find-current="String(item.id) === findCurrent"
        />
      </div>
    </VirtualRows>
  </div>
</template>
