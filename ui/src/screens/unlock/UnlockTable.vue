<script setup lang="ts">
import { VirtualRows } from '@/components/ui/virtual'
import { useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import { nodeNumber } from '@/lib/graph/achievementNode'
import type { UnlockNode } from '@/lib/ipc/types'
import { canQueue, isQueued } from '@/lib/plan/queueRows'
import { rowWidePx } from '@/lib/scale/rows'
import UnlockRow from './UnlockRow.vue'

defineProps<{
  nodes: UnlockNode[]
  queued: Set<number>
  canWrite: boolean
  busy: boolean
}>()
const emit = defineEmits<{
  add: [achievement: number]
}>()
const { t } = useMessages()
</script>

<template>
  <!-- The screen scrolls as a page (`PageScroll`): the rows virtualize against it, and the
       columns' header pins to its top while the list goes by under it. -->
  <div class="flex flex-col">
    <div
      class="sticky top-0 z-raised-header grid grid-cols-unlock items-center border-b border-hairline bg-muted text-label text-subtle-foreground @max-compact/page:grid-cols-unlock-narrow"
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
    <VirtualRows v-slot="{ visible }" :rows="nodes" :row-px="rowWidePx">
      <div
        v-for="{ index, style, row: node } in visible"
        :key="nodeNumber(node)"
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
          @add="emit('add', nodeNumber(node))"
        />
      </div>
    </VirtualRows>
  </div>
</template>
