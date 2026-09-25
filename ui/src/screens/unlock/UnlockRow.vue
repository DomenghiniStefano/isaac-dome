<script setup lang="ts">
import { ListPlusIcon } from '@lucide/vue'
import { computed } from 'vue'
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import AchievementArt from '@/components/graph/AchievementArt.vue'
import NodeStateBadge from '@/components/graph/NodeStateBadge.vue'
import { ArtSize } from '@/components/graph/artSize'
import PixelSprite from '@/components/sprite/PixelSprite.vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import { knownAchievement } from '@/lib/graph/achievementNode'
import { targetName } from '@/lib/graph/characterName'
import { nodeSlot } from '@/lib/graph/unlockFacets'
import type { UnlockNode } from '@/lib/ipc/types'

const props = defineProps<{
  node: UnlockNode
  queued: boolean
  canAdd: boolean
  busy: boolean
}>()
const emit = defineEmits<{ add: [] }>()
const { t } = useMessages()

const known = computed(() => knownAchievement(props.node))
const text = computed(() => known.value?.text ?? t('graph.unknownAchievement'))
const first = computed(() => props.node.unlocks[0] ?? null)
const more = computed(() => Math.max(0, props.node.unlocks.length - 1))
</script>

<template>
  <!-- One row of Unlock's grid: the seven cells are the grid's children, so this component has
       no root of its own. "in coda" rides on the slot line: a badge beside the name doesn't
       fit a 40px row with two lines. -->
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
      >{{ t('graph.slot') }} {{ nodeSlot(node)
      }}<template v-if="queued"> · {{ t('queue.inQueue') }}</template></span
    >
  </span>
  <!-- The three cells that fall at compact, each paired with a track dropped from
       `grid-cols-unlock-narrow` (spec 3.13a §7): what the row unlocks, its condition, its fan-out.
       Hidden and not collapsed to a zero-width track, because a zero-width cell stays in the
       accessibility tree and a screen reader would read columns the eye was told it could do
       without. -->
  <span class="flex min-w-0 items-center gap-2 px-2 @max-compact/page:hidden">
    <template v-if="first">
      <PixelSprite
        v-if="first.kind === 'item'"
        :url="first.iconUrl"
        placeholder
        class="size-8 shrink-0"
      />
      <span class="truncate text-caption text-foreground">{{
        targetName(t, first)
      }}</span>
      <span
        v-if="more > 0"
        class="shrink-0 text-label text-subtle-foreground tabular-nums"
        >+{{ more }}</span
      >
    </template>
    <EmptyValue v-else>{{ t('unlock.unlocksNothing') }}</EmptyValue>
  </span>
  <span
    class="min-w-0 truncate px-2 text-caption text-foreground-soft @max-compact/page:hidden"
  >
    <template v-if="known?.condition">{{ known.condition }}</template>
    <EmptyValue v-else>{{ t('unlock.noCondition') }}</EmptyValue>
  </span>
  <span class="px-2">
    <NodeStateBadge :node="node" />
  </span>
  <span
    class="px-2 text-right text-row text-foreground tabular-nums @max-compact/page:hidden"
    >{{ node.done ? '—' : node.graph.fanOut }}</span
  >
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
