<script setup lang="ts">
import { useVirtualizer } from '@tanstack/vue-virtual'
import { computed, ref } from 'vue'
import { useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import { nodeSlot } from '@/lib/graph/unlockFilter'
import type { UnlockNode } from '@/lib/ipc/types'
import UnlockRow from './UnlockRow.vue'
import { unlockRowHeight } from './unlockLayout'

const props = defineProps<{ nodes: UnlockNode[] }>()
const { t } = useMessages()

const scroller = ref<HTMLElement | null>(null)

// 641 rows, drawn as many as fit plus a margin: TanStack Virtual positions them, the options
// are a computed so a new filter's count reaches the virtualizer.
const virtualizer = useVirtualizer(
  computed(() => ({
    count: props.nodes.length,
    getScrollElement: () => scroller.value,
    estimateSize: () => unlockRowHeight,
    overscan: 8,
  })),
)

const visible = computed(() =>
  virtualizer.value.getVirtualItems().flatMap((item) => {
    const node = props.nodes[item.index]
    return node ? [{ item, node }] : []
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
      class="grid grid-cols-unlock items-center border-b border-hairline bg-muted text-label text-subtle-foreground"
    >
      <span />
      <span class="px-2 py-1.5">{{ t('unlock.columns.achievement') }}</span>
      <span class="px-2 py-1.5">{{ t('unlock.columns.unlocks') }}</span>
      <span class="px-2 py-1.5">{{ t('unlock.columns.condition') }}</span>
      <span class="px-2 py-1.5">{{ t('unlock.columns.state') }}</span>
      <span class="px-2 py-1.5 text-right">{{
        t('unlock.columns.fanOut')
      }}</span>
    </div>
    <div ref="scroller" class="max-h-unlock-body overflow-auto">
      <div :style="body" class="relative h-(--unlock-total)">
        <div
          v-for="{ item, node } in visible"
          :key="nodeSlot(node)"
          :style="rowStart(item.start)"
          :class="
            cn(
              'absolute inset-x-0 top-0 grid h-row-wide translate-y-(--row-start) grid-cols-unlock items-center border-b border-hairline hover:bg-row-hover',
              item.index % 2 === 1 && 'bg-row-alt',
            )
          "
        >
          <UnlockRow :node="node" />
        </div>
      </div>
    </div>
  </div>
</template>
