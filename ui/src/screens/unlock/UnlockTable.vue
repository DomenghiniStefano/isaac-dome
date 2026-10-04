<script setup lang="ts">
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import AchievementArt from '@/components/graph/AchievementArt.vue'
import NodeStateBadge from '@/components/graph/NodeStateBadge.vue'
import { ArtSize } from '@/components/graph/artSize'
import QueueActionButton from '@/components/plan/QueueActionButton.vue'
import { GridTable } from '@/components/ui/grid-table'
import { useQueueOffer } from '@/composables/useQueueOffer'
import { useMessages } from '@/i18n'
import { knownAchievement, nodeNumber } from '@/lib/graph/achievementNode'
import type { UnlockNode } from '@/lib/ipc/types'
import { isQueued, nodeQueueTarget } from '@/lib/plan/queueRows'
import { unlockColumns } from '@/lib/table/columns'
import UnlockNameCell from './UnlockNameCell.vue'
import UnlockUnlocksCell from './UnlockUnlocksCell.vue'

// The screen scrolls as a page (`PageScroll`): the rows virtualize against it, and the
// columns' header pins to its top while the list goes by under it.
defineProps<{ nodes: UnlockNode[] }>()
const { t } = useMessages()
const { queued } = useQueueOffer()
</script>

<template>
  <GridTable
    :columns="unlockColumns"
    :rows="nodes"
    :row-key="nodeNumber"
    virtual
  >
    <template #cell-art="{ row }">
      <AchievementArt
        :url="knownAchievement(row)?.iconUrl ?? null"
        :size="ArtSize.Thumb"
      />
    </template>
    <template #cell-achievement="{ row }">
      <UnlockNameCell :node="row" :queued="isQueued(row, queued)" />
    </template>
    <template #cell-unlocks="{ row }">
      <UnlockUnlocksCell :node="row" />
    </template>
    <template #cell-condition="{ row }">
      <span
        v-if="knownAchievement(row)?.condition"
        class="truncate text-caption text-foreground-soft"
        >{{ knownAchievement(row)?.condition }}</span
      >
      <EmptyValue v-else>{{ t('unlock.noCondition') }}</EmptyValue>
    </template>
    <template #cell-state="{ row }">
      <NodeStateBadge :node="row" />
    </template>
    <template #cell-fanOut="{ row }">
      <span class="text-row text-foreground tabular-nums">{{
        row.done ? '—' : row.graph.fanOut
      }}</span>
    </template>
    <template #actions="{ row }">
      <QueueActionButton :target="nodeQueueTarget(row)" />
    </template>
  </GridTable>
</template>
