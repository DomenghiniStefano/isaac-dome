<script setup lang="ts">
import { computed, ref } from 'vue'
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import { Card, CardHeader, CardTitle } from '@/components/ui/card'
import { useMessages } from '@/i18n'
import { EventKey } from '@/lib/constants/eventKeys'
import type {
  QueueDiagnostic,
  QueueRow as QueueRowView,
  UnlockNode,
} from '@/lib/ipc/types'
import type { Anchor } from '@/lib/plan/queueDrop'
import {
  DropEdge,
  StepDirection,
  dropAnchor,
  dropEdge,
  stepAnchor,
} from '@/lib/plan/queueDrop'
import { knownText, rowId, stoppedUnder } from '@/lib/plan/queueRows'
import type { QueueMove } from '@/stores/queue'
import QueueFootnotes from './QueueFootnotes.vue'
import QueueRow from './QueueRow.vue'

const props = defineProps<{
  rows: QueueRowView[]
  diagnostics: QueueDiagnostic[]
  nodes: UnlockNode[]
  busy: boolean
  lastMove: QueueMove | null
}>()
const emit = defineEmits<{
  move: [achievement: number, after: number | null]
  remove: [achievement: number]
}>()
const { t } = useMessages()

interface Drag {
  from: number
  startY: number
  moving: boolean
}
interface Drop {
  index: number
  edge: DropEdge
}

// Pixels the pointer travels before a press on the grip becomes a drag: below it, a click.
const dragThreshold = 4
const rowSelector = '[data-queue-row]'

const list = ref<HTMLElement | null>(null)
const drag = ref<Drag | null>(null)
const drop = ref<Drop | null>(null)
// Where the rows are, read once when a press becomes a drag: nothing moves until the drop.
let rowRects: DOMRect[] = []

const ids = computed(() => props.rows.map(rowId))

const rowElements = (): HTMLElement[] =>
  list.value ? [...list.value.querySelectorAll<HTMLElement>(rowSelector)] : []

const dropAt = (y: number): Drop | null => {
  for (const [index, r] of rowRects.entries()) {
    if (y >= r.top && y < r.bottom)
      return { index, edge: dropEdge(y, r.top, r.height) }
  }
  return null
}

const anchorOf = (d: Drag, target: Drop | null): Anchor | null =>
  target ? dropAnchor(ids.value, d.from, target.index, target.edge) : null

// The gap the line is drawn in, only for a drop that would move something.
const gap = computed((): number | null => {
  const d = drag.value
  const target = drop.value
  if (!d?.moving || !target || !anchorOf(d, target)) return null
  return target.edge === DropEdge.Above ? target.index : target.index + 1
})

// A press becomes a drag only past the threshold, and only then is the pointer captured, as
// on the tab strip.
const onGrab = (index: number, e: PointerEvent) => {
  if (e.button !== 0 || props.busy) return
  drag.value = { from: index, startY: e.clientY, moving: false }
}

const onPointerMove = (e: PointerEvent) => {
  const d = drag.value
  if (!d) return
  if (!d.moving) {
    if (Math.abs(e.clientY - d.startY) < dragThreshold) return
    d.moving = true
    rowRects = rowElements().map((el) => el.getBoundingClientRect())
    list.value?.setPointerCapture(e.pointerId)
  }
  drop.value = dropAt(e.clientY)
}

const onPointerUp = (e: PointerEvent) => {
  const d = drag.value
  const target = drop.value
  drag.value = null
  drop.value = null
  rowRects = []
  if (list.value?.hasPointerCapture(e.pointerId))
    list.value.releasePointerCapture(e.pointerId)
  if (!d?.moving) return
  const anchor = anchorOf(d, target)
  const moved = ids.value[d.from]
  if (anchor && moved !== undefined) emit('move', moved, anchor.after)
}

const directionOf = (key: string): StepDirection | null => {
  if (key === EventKey.ArrowUp) return StepDirection.Up
  if (key === EventKey.ArrowDown) return StepDirection.Down
  return null
}

const onStep = (index: number, e: KeyboardEvent) => {
  const direction = e.altKey ? directionOf(e.key) : null
  if (!direction || props.busy) return
  e.preventDefault()
  const anchor = stepAnchor(ids.value, index, direction)
  const moved = ids.value[index]
  if (anchor && moved !== undefined) emit('move', moved, anchor.after)
}

// What the band says: how to drag, what a drop will do, or where the last move stopped and why.
const hint = computed((): string => {
  if (drag.value?.moving) return t('plan.hint.dragging')
  const last = props.lastMove
  const wall = last
    ? stoppedUnder(props.rows, last.achievement, last.after)
    : null
  if (!wall) return t('plan.hint.idle')
  const text = knownText(wall.node) ?? `${t('plan.achievement')} ${rowId(wall)}`
  return `${t('plan.hint.stoppedUnder')} «${text}»: ${t('plan.hint.prerequisite')}`
})
</script>

<template>
  <Card>
    <CardHeader class="flex-wrap">
      <CardTitle>{{ t('plan.queueTitle') }}</CardTitle>
      <span class="text-caption text-foreground-soft">{{ hint }}</span>
    </CardHeader>
    <div
      v-if="rows.length > 0"
      ref="list"
      class="flex flex-col py-1"
      @pointermove="onPointerMove"
      @pointerup="onPointerUp"
      @pointercancel="onPointerUp"
    >
      <div
        v-for="(row, index) in rows"
        :key="rowId(row)"
        data-queue-row
        class="relative border-b border-hairline last:border-b-0"
      >
        <span
          v-if="gap === index"
          class="absolute inset-x-0 -top-px h-0.5 bg-primary"
        />
        <QueueRow
          :row="row"
          :rows="rows"
          :position="index + 1"
          :dragging="drag?.moving === true && drag.from === index"
          :busy="busy"
          @grab="onGrab(index, $event)"
          @step="onStep(index, $event)"
          @remove="emit('remove', rowId(row))"
        />
        <span
          v-if="index === rows.length - 1 && gap === rows.length"
          class="absolute inset-x-0 -bottom-px h-0.5 bg-primary"
        />
      </div>
    </div>
    <div v-else class="flex flex-col items-start gap-1 p-4">
      <EmptyCategory>{{ t('plan.empty') }}</EmptyCategory>
      <span class="text-caption text-subtle-foreground">{{
        t('plan.emptyHint')
      }}</span>
    </div>
    <QueueFootnotes
      :diagnostics="diagnostics"
      :nodes="nodes"
      :busy="busy"
      @remove="emit('remove', $event)"
    />
  </Card>
</template>
