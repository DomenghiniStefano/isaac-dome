<script setup lang="ts">
import { useVirtualizer } from '@tanstack/vue-virtual'
import { computed, ref } from 'vue'
import { useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import type { CollectionItem } from '@/lib/ipc/types'
import CollectionRow from './CollectionRow.vue'
import { collectionRowHeight } from './collectionLayout'

const props = defineProps<{ items: CollectionItem[] }>()
const { t } = useMessages()

const scroller = ref<HTMLElement | null>(null)

// Some 720 collectibles, drawn as many as fit plus a margin, the same rule as Unlock: the options
// are a computed so a new filter's count reaches the virtualizer.
const virtualizer = useVirtualizer(
  computed(() => ({
    count: props.items.length,
    getScrollElement: () => scroller.value,
    estimateSize: () => collectionRowHeight,
    overscan: 8,
  })),
)

const visible = computed(() =>
  virtualizer.value.getVirtualItems().flatMap((entry) => {
    const item = props.items[entry.index]
    return item ? [{ entry, item }] : []
  }),
)

// Geometry measured at runtime travels as CSS variables, read by utilities.
const body = computed(() => ({
  '--unlock-total': `${virtualizer.value.getTotalSize()}px`,
}))
const rowStart = (start: number) => ({ '--row-start': `${start}px` })
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
    <div ref="scroller" class="max-h-unlock-body overflow-auto">
      <div :style="body" class="relative h-(--unlock-total)">
        <div
          v-for="{ entry, item } in visible"
          :key="item.id"
          :style="rowStart(entry.start)"
          :class="
            cn(
              'absolute inset-x-0 top-0 grid h-row-wide translate-y-(--row-start) grid-cols-collection items-center border-b border-hairline hover:bg-row-hover',
              entry.index % 2 === 1 && 'bg-row-alt',
            )
          "
        >
          <CollectionRow :item="item" />
        </div>
      </div>
    </div>
  </div>
</template>
