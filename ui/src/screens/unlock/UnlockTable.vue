<script setup lang="ts">
import { VirtualRows } from '@/components/ui/virtual'
import type { ScrollOffset } from '@/lib/scale/scrollOffset'
import { useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import { nodeSlot } from '@/lib/graph/unlockFacets'
import type { UnlockNode } from '@/lib/ipc/types'
import { canQueue, isQueued } from '@/lib/plan/queueRows'
import { rowWidePx } from '@/lib/scale/rows'
import UnlockRow from './UnlockRow.vue'

defineProps<{
  nodes: UnlockNode[]
  queued: Set<number>
  canWrite: boolean
  busy: boolean
  offset: ScrollOffset | null
}>()
const emit = defineEmits<{
  add: [achievement: number]
  offsetChange: [offset: ScrollOffset]
}>()
const { t } = useMessages()
</script>

<template>
  <!-- A link in the filling chain (spec 3.13a §4): the header keeps its own height, the rows take
       the rest. `min-h-0` here as on every link, or the scroll box below never shrinks. -->
  <div class="flex min-h-0 flex-1 flex-col">
    <div
      class="grid grid-cols-unlock items-center border-b border-hairline bg-muted text-label text-subtle-foreground @max-compact/page:grid-cols-unlock-narrow"
    >
      <span />
      <span class="px-2 py-1.5">{{ t('unlock.columns.achievement') }}</span>
      <span class="px-2 py-1.5 @max-compact/page:hidden">{{
        t('unlock.columns.unlocks')
      }}</span>
      <span class="px-2 py-1.5 @max-compact/page:hidden">{{
        t('unlock.columns.condition')
      }}</span>
      <span class="px-2 py-1.5">{{ t('unlock.columns.state') }}</span>
      <span class="px-2 py-1.5 text-right @max-compact/page:hidden">{{
        t('unlock.columns.fanOut')
      }}</span>
      <span />
    </div>
    <VirtualRows
      v-slot="{ visible }"
      :rows="nodes"
      :row-px="rowWidePx"
      :offset="offset"
      @offset-change="emit('offsetChange', $event)"
    >
      <div
        v-for="{ index, style, row: node } in visible"
        :key="nodeSlot(node)"
        :style="style"
        :class="
          cn(
            'absolute inset-x-0 top-0 grid h-row-wide translate-y-(--row-start) grid-cols-unlock items-center border-b border-hairline hover:bg-row-hover @max-compact/page:grid-cols-unlock-narrow',
            index % 2 === 1 && 'bg-row-alt',
          )
        "
      >
        <UnlockRow
          :node="node"
          :queued="isQueued(node, queued)"
          :can-add="canWrite && canQueue(node, queued)"
          :busy="busy"
          @add="emit('add', nodeSlot(node))"
        />
      </div>
    </VirtualRows>
  </div>
</template>
