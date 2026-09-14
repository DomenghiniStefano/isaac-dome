<script setup lang="ts">
import { VirtualRows } from '@/components/ui/virtual'
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
}>()
const emit = defineEmits<{ add: [achievement: number] }>()
const { t } = useMessages()
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
      <span />
    </div>
    <VirtualRows v-slot="{ visible }" :rows="nodes" :row-px="rowWidePx">
      <div
        v-for="{ index, style, row: node } in visible"
        :key="nodeSlot(node)"
        :style="style"
        :class="
          cn(
            'absolute inset-x-0 top-0 grid h-row-wide translate-y-(--row-start) grid-cols-unlock items-center border-b border-hairline hover:bg-row-hover',
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
