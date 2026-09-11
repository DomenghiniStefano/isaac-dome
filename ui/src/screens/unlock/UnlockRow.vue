<script setup lang="ts">
import { computed } from 'vue'
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import AchievementArt from '@/components/graph/AchievementArt.vue'
import NodeStateBadge from '@/components/graph/NodeStateBadge.vue'
import { ArtSize } from '@/components/graph/artSize'
import PixelSprite from '@/components/sprite/PixelSprite.vue'
import { useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import { nodeSlot } from '@/lib/graph/unlockFilter'
import type { UnlockNode } from '@/lib/ipc/types'

const props = defineProps<{ node: UnlockNode }>()
const { t } = useMessages()

const known = computed(() =>
  props.node.achievement.kind === 'known' ? props.node.achievement : null,
)
const text = computed(() => known.value?.text ?? t('graph.unknownAchievement'))
const first = computed(() => props.node.unlocks[0] ?? null)
const more = computed(() => Math.max(0, props.node.unlocks.length - 1))
</script>

<template>
  <!-- One row of Unlock's grid: the six cells are the grid's children, so this component has
       no root of its own. -->
  <span class="px-2">
    <AchievementArt :url="known?.iconUrl ?? null" :size="ArtSize.Thumb" />
  </span>
  <span class="flex min-w-0 flex-col px-2">
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
      >{{ t('graph.slot') }} {{ nodeSlot(node) }}</span
    >
  </span>
  <span class="flex min-w-0 items-center gap-2 px-2">
    <template v-if="first">
      <PixelSprite
        v-if="first.kind === 'item'"
        :url="first.iconUrl"
        placeholder
        class="size-8 shrink-0"
      />
      <span class="truncate text-caption text-foreground">{{
        first.name
      }}</span>
      <span
        v-if="more > 0"
        class="shrink-0 text-label text-subtle-foreground tabular-nums"
        >+{{ more }}</span
      >
    </template>
    <EmptyValue v-else>{{ t('unlock.unlocksNothing') }}</EmptyValue>
  </span>
  <span class="min-w-0 truncate px-2 text-caption text-foreground-soft">
    <template v-if="known?.hint">{{ known.hint }}</template>
    <EmptyValue v-else>{{ t('unlock.noCondition') }}</EmptyValue>
  </span>
  <span class="px-2">
    <NodeStateBadge :node="node" />
  </span>
  <span class="px-2 text-right text-row text-foreground tabular-nums">{{
    node.done ? '—' : node.graph.fanOut
  }}</span>
</template>
