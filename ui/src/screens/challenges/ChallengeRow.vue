<script setup lang="ts">
import { ListPlusIcon } from '@lucide/vue'
import { computed } from 'vue'
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import WikiInline from '@/components/wiki/WikiInline.vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import type { ChallengeRow, Target } from '@/lib/ipc/types'
import ChallengeStateBadge from './ChallengeStateBadge.vue'

const props = defineProps<{
  row: ChallengeRow
  queued: boolean
  canAdd: boolean
  busy: boolean
}>()
const emit = defineEmits<{
  add: []
  navigate: [target: Target, newTab: boolean]
}>()
const { t } = useMessages()

// The first reward is the one the button queues; the rest are counted, the way Unlock counts
// what a node unlocks beyond the first.
const reward = computed(() => props.row.rewards[0] ?? null)
const more = computed(() => Math.max(0, props.row.rewards.length - 1))
const done = computed(() => props.row.state.kind === 'done')
const openName = (page: Target, ctrlKey: boolean) =>
  emit('navigate', page, ctrlKey)
</script>

<template>
  <!-- One row of the Challenges grid: the six cells are the grid's children, so this component
       has no root of its own. -->
  <span
    class="px-2 text-right text-label text-subtle-foreground tabular-nums"
    >{{ row.number }}</span
  >
  <span class="flex min-w-0 flex-col px-2">
    <Button
      v-if="row.page"
      :variant="ButtonVariant.Link"
      :size="ButtonSize.Compact"
      :class="cn('justify-start truncate', done && 'text-subtle-foreground')"
      @click="openName(row.page, $event.ctrlKey)"
      >{{ row.name }}</Button
    >
    <span
      v-else
      :class="
        cn(
          'truncate text-row',
          done ? 'text-subtle-foreground' : 'text-foreground',
        )
      "
      >{{ row.name }}</span
    >
    <span class="min-w-0 truncate text-micro text-faint-foreground">
      <template v-if="reward"
        >{{
          reward.text ??
          t('challenges.unnamedReward', { id: reward.achievement })
        }}<template v-if="more > 0"> +{{ more }}</template></template
      >
      <template v-else>{{ t('challenges.unlocksNothing') }}</template>
      <template v-if="queued"> · {{ t('queue.inQueue') }}</template>
    </span>
  </span>
  <span
    class="min-w-0 truncate px-2 text-caption text-foreground @max-compact/page:hidden"
  >
    <!-- A challenge the wiki has no page for says nothing here: it must not read as "any
         character", which is a fact about the game nobody read. -->
    <template v-if="row.characterName">{{ row.characterName }}</template>
    <EmptyValue v-else>{{ t('challenges.noCondition') }}</EmptyValue>
  </span>
  <span class="flex min-w-0 items-center gap-1.5 px-2 @max-compact/page:hidden">
    <WikiInline
      v-if="row.goal"
      :inline="row.goal"
      class="min-w-0 truncate text-caption"
      @navigate="(target, newTab) => emit('navigate', target, newTab)"
    />
    <EmptyValue v-else>{{ t('challenges.noCondition') }}</EmptyValue>
    <span
      v-if="row.blindfolded"
      class="shrink-0 text-label text-subtle-foreground"
      >{{ t('challenges.blindfolded') }}</span
    >
  </span>
  <span class="px-2">
    <ChallengeStateBadge :state="row.state" />
  </span>
  <span class="flex justify-center">
    <Button
      v-if="canAdd"
      :variant="ButtonVariant.Ghost"
      :size="ButtonSize.IconCompact"
      :aria-label="t('queue.add')"
      :disabled="busy"
      @click="emit('add')"
    >
      <ListPlusIcon />
    </Button>
  </span>
</template>
