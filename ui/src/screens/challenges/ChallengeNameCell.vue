<script setup lang="ts">
import { computed } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import type { ChallengeRow, Target } from '@/lib/ipc/types'

// A challenge's name and what it rewards. The first reward is the one the Actions button
// queues; the rest are counted, the way Unlock counts what a node unlocks beyond the first.
const props = defineProps<{ row: ChallengeRow; queued: boolean }>()
const emit = defineEmits<{ navigate: [target: Target, newTab: boolean] }>()
const { t } = useMessages()

const reward = computed(() => props.row.rewards[0] ?? null)
const more = computed(() => Math.max(0, props.row.rewards.length - 1))
const done = computed(() => props.row.state.kind === 'done')

const open = (event: MouseEvent): void => {
  if (props.row.page) emit('navigate', props.row.page, event.ctrlKey)
}
</script>

<template>
  <span class="flex min-w-0 flex-col">
    <Button
      v-if="row.page"
      :variant="ButtonVariant.Link"
      :size="ButtonSize.Compact"
      :class="cn('max-w-full justify-start', done && 'text-subtle-foreground')"
      @click="open"
      ><span class="min-w-0 truncate">{{ row.name }}</span></Button
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
</template>
