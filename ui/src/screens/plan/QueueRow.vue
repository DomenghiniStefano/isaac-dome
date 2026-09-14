<script setup lang="ts">
import { GripVerticalIcon, XIcon } from '@lucide/vue'
import { computed } from 'vue'
import AchievementArt from '@/components/graph/AchievementArt.vue'
import NodeStateBadge from '@/components/graph/NodeStateBadge.vue'
import { ArtSize } from '@/components/graph/artSize'
import { unlockKindText } from '@/components/graph/unlockKindText'
import { Badge, BadgeVariant } from '@/components/ui/badge'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import { targetName } from '@/lib/graph/characterName'
import { targetKind } from '@/lib/graph/unlockFacets'
import type { QueueRow } from '@/lib/ipc/types'
import { knownText, originRows, rowId } from '@/lib/plan/queueRows'

const props = defineProps<{
  row: QueueRow
  rows: QueueRow[]
  position: number
  dragging: boolean
  busy: boolean
}>()
const emit = defineEmits<{
  grab: [e: PointerEvent]
  step: [e: KeyboardEvent]
  remove: []
}>()
const { t } = useMessages()

const known = computed(() =>
  props.row.node.achievement.kind === 'known'
    ? props.row.node.achievement
    : null,
)
const byId = (id: number): string => `${t('plan.achievement')} ${id}`
const text = computed(() => knownText(props.row.node) ?? byId(rowId(props.row)))
const condition = computed(() =>
  known.value?.condition
    ? `${t('plan.row.condition')} ${known.value.condition}`
    : null,
)
// The wishes this row serves, by their text when the queue shows them.
const originText = (id: number, row: QueueRow | null): string =>
  (row ? knownText(row.node) : null) ?? byId(id)
const serves = computed(() =>
  originRows(props.row, props.rows).map((origin) => ({
    id: origin.id,
    label: `${t('plan.row.serves')} «${originText(origin.id, origin.row)}»`,
  })),
)
const firstUnlock = computed(() => {
  const target = props.row.node.unlocks[0]
  return target
    ? `${t('plan.row.unlocks')} ${targetName(t, target)} · ${t(unlockKindText[targetKind(target)])}`
    : null
})
const fanOut = computed(() => props.row.node.graph.fanOut)
</script>

<template>
  <!-- Schermate.dc.html, "La coda": the grip, the position, the drawing, the text and why the
       row is here; on the right its state and what is still outside the queue. -->
  <div
    :class="
      cn('flex items-start gap-3 px-3 py-2.5', dragging && 'opacity-disabled')
    "
  >
    <Button
      :variant="ButtonVariant.Ghost"
      :size="ButtonSize.Icon"
      :aria-label="t('plan.row.move')"
      :disabled="busy"
      class="cursor-grab touch-none"
      @pointerdown="emit('grab', $event)"
      @keydown="emit('step', $event)"
    >
      <GripVerticalIcon />
    </Button>
    <span
      class="mt-2 w-4 shrink-0 text-right text-label text-subtle-foreground tabular-nums"
      >{{ position }}</span
    >
    <AchievementArt :url="known?.iconUrl ?? null" :size="ArtSize.Thumb" />
    <div class="flex min-w-0 flex-1 flex-col items-start gap-1.5">
      <span class="text-row text-foreground">{{ text }}</span>
      <span v-if="condition" class="text-caption text-subtle-foreground">{{
        condition
      }}</span>
      <div class="flex flex-wrap items-center gap-1.5">
        <Badge v-if="row.wanted" :variant="BadgeVariant.Wanted">{{
          t('plan.row.wanted')
        }}</Badge>
        <Badge v-for="s in serves" :key="s.id" :variant="BadgeVariant.Tag">{{
          s.label
        }}</Badge>
        <Badge v-if="firstUnlock" :variant="BadgeVariant.Tag">{{
          firstUnlock
        }}</Badge>
      </div>
    </div>
    <div class="flex shrink-0 flex-col items-end gap-1">
      <NodeStateBadge :node="row.node" />
      <span
        v-if="fanOut > 0"
        class="text-label text-subtle-foreground tabular-nums"
        >{{ t('plan.row.fanOut') }}: {{ fanOut }}</span
      >
      <span
        v-if="row.stepsNotQueued > 0"
        class="text-label text-foreground-soft tabular-nums"
        >{{ t('plan.row.outsideQueue') }}: {{ row.stepsNotQueued }}</span
      >
    </div>
    <Button
      v-if="row.wanted"
      :variant="ButtonVariant.Ghost"
      :size="ButtonSize.Icon"
      :aria-label="t('queue.remove')"
      :disabled="busy"
      @click="emit('remove')"
    >
      <XIcon />
    </Button>
    <span v-else class="size-control shrink-0" />
  </div>
</template>
