<script setup lang="ts">
import { computed } from 'vue'
import { useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import { knownAchievement, nodeNumber } from '@/lib/graph/achievementNode'
import type { UnlockNode } from '@/lib/ipc/types'

// An achievement's name and its slot. "in coda" rides on the slot line: a badge beside the name
// doesn't fit a 40px row with two lines.
const props = defineProps<{ node: UnlockNode; queued: boolean }>()
const { t } = useMessages()

const text = computed(
  () => knownAchievement(props.node)?.text ?? t('graph.unknownAchievement'),
)
</script>

<template>
  <span class="flex min-w-0 flex-col">
    <span
      :class="
        cn(
          'truncate text-row',
          node.done ? 'text-subtle-foreground' : 'text-foreground',
        )
      "
      >{{ text }}</span
    >
    <span class="text-micro text-faint-foreground tabular-nums"
      >{{ t('graph.slot', { slot: nodeNumber(node) })
      }}<template v-if="queued"> · {{ t('queue.inQueue') }}</template></span
    >
  </span>
</template>
