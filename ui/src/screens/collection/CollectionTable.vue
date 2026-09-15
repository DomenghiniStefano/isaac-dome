<script setup lang="ts">
import { VirtualRows } from '@/components/ui/virtual'
import type { ScrollOffset } from '@/lib/scale/scrollOffset'
import { useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import type { CollectionItem } from '@/lib/ipc/types'
import { rowWidePx } from '@/lib/scale/rows'
import CollectionRow from './CollectionRow.vue'

defineProps<{ items: CollectionItem[]; offset: ScrollOffset | null }>()
const emit = defineEmits<{ offsetChange: [offset: ScrollOffset] }>()
const { t } = useMessages()
</script>

<template>
  <div class="flex flex-col">
    <div
      class="grid grid-cols-collection items-center border-b border-hairline bg-muted text-label text-subtle-foreground"
    >
      <span />
      <span class="px-2 py-1.5">{{ t('collection.columns.item') }}</span>
      <span class="px-2 py-1.5">{{ t('collection.columns.quality') }}</span>
      <span class="px-2 py-1.5">{{ t('collection.columns.pools') }}</span>
      <span class="px-2 py-1.5">{{ t('collection.columns.origin') }}</span>
      <span class="px-2 py-1.5">{{ t('collection.columns.state') }}</span>
    </div>
    <VirtualRows
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
            'absolute inset-x-0 top-0 grid h-row-wide translate-y-(--row-start) grid-cols-collection items-center border-b border-hairline hover:bg-row-hover',
            index % 2 === 1 && 'bg-row-alt',
          )
        "
      >
        <CollectionRow :item="item" />
      </div>
    </VirtualRows>
  </div>
</template>
