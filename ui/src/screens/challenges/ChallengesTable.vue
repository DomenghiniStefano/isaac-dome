<script setup lang="ts">
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import QueueActionButton from '@/components/plan/QueueActionButton.vue'
import { GridTable } from '@/components/ui/grid-table'
import { useQueueOffer } from '@/composables/useQueueOffer'
import { useMessages } from '@/i18n'
import { challengeQueueTarget } from '@/lib/challenges/challengeQueue'
import type { ChallengeRow, Target } from '@/lib/ipc/types'
import { challengeColumns } from '@/lib/table/columns'
import ChallengeGoalCell from './ChallengeGoalCell.vue'
import ChallengeNameCell from './ChallengeNameCell.vue'
import ChallengeStateBadge from './ChallengeStateBadge.vue'

// Forty-five rows: no virtual list. `VirtualRows` exists for 733 items and 642 achievements,
// and a list this short pays its machinery for nothing.
defineProps<{ rows: ChallengeRow[] }>()
const emit = defineEmits<{ navigate: [target: Target, newTab: boolean] }>()
const { t } = useMessages()
const { queued } = useQueueOffer()

const isQueued = (row: ChallengeRow): boolean =>
  row.rewards.some((r) => queued.value.has(r.achievement))
</script>

<template>
  <GridTable
    :columns="challengeColumns"
    :rows="rows"
    :row-key="(row) => row.number"
  >
    <template #cell-number="{ row }">
      <span class="text-label text-subtle-foreground tabular-nums">{{
        row.number
      }}</span>
    </template>
    <template #cell-name="{ row }">
      <ChallengeNameCell
        :row="row"
        :queued="isQueued(row)"
        @navigate="(target, newTab) => emit('navigate', target, newTab)"
      />
    </template>
    <template #cell-character="{ row }">
      <!-- A challenge the wiki has no page for says nothing here: it must not read as "any
           character", which is a fact about the game nobody read. -->
      <span
        v-if="row.characterName"
        class="truncate text-caption text-foreground"
        >{{ row.characterName }}</span
      >
      <EmptyValue v-else>{{ t('challenges.noCondition') }}</EmptyValue>
    </template>
    <template #cell-goal="{ row }">
      <ChallengeGoalCell
        :row="row"
        @navigate="(target, newTab) => emit('navigate', target, newTab)"
      />
    </template>
    <template #cell-state="{ row }">
      <ChallengeStateBadge :state="row.state" />
    </template>
    <template #actions="{ row }">
      <QueueActionButton :target="challengeQueueTarget(row)" />
    </template>
  </GridTable>
</template>
