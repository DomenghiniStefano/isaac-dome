<script setup lang="ts">
import { useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import type { ChallengeRow as Row, Target } from '@/lib/ipc/types'
import ChallengeRow from './ChallengeRow.vue'

// Forty-five rows: no virtual list. `VirtualRows` exists for 733 items and 642 achievements,
// and a list this short pays its machinery for nothing.
defineProps<{
  rows: Row[]
  queued: number[]
  canWrite: boolean
  busy: boolean
}>()
const emit = defineEmits<{
  add: [achievement: number]
  navigate: [target: Target, newTab: boolean]
}>()
const { t } = useMessages()
</script>

<template>
  <div class="flex flex-col">
    <div
      class="grid grid-cols-challenges items-center border-b border-hairline bg-muted text-label text-subtle-foreground"
    >
      <span class="px-2 py-1.5 text-right">{{
        t('challenges.columns.number')
      }}</span>
      <span class="px-2 py-1.5">{{ t('challenges.columns.challenge') }}</span>
      <span class="px-2 py-1.5">{{ t('challenges.columns.character') }}</span>
      <span class="px-2 py-1.5">{{ t('challenges.columns.goal') }}</span>
      <span class="px-2 py-1.5">{{ t('challenges.columns.state') }}</span>
      <span />
    </div>
    <div
      v-for="(row, index) in rows"
      :key="row.number"
      :class="
        cn(
          'grid min-h-row-wide grid-cols-challenges items-center border-b border-hairline hover:bg-row-hover',
          index % 2 === 1 && 'bg-row-alt',
        )
      "
    >
      <ChallengeRow
        :row="row"
        :queued="row.rewards.some((r) => queued.includes(r.achievement))"
        :can-add="canWrite"
        :busy="busy"
        @add="emit('add', row.rewards[0]?.achievement ?? 0)"
        @navigate="(target, newTab) => emit('navigate', target, newTab)"
      />
    </div>
  </div>
</template>
