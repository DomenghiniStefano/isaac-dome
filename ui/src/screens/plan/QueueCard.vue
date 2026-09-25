<script setup lang="ts">
import { computed, ref } from 'vue'
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import GoalRow from '@/components/plan/GoalRow.vue'
import { Card, CardHeader, CardTitle } from '@/components/ui/card'
import { DragGhost } from '@/components/ui/drag'
import { useDragList } from '@/composables/useDragList'
import { useMessages } from '@/i18n'

import { Axis, boxAt } from '@/lib/drag/dragList'
import type { Box, Point } from '@/lib/drag/dragList'
import type {
  QueueDiagnostic,
  QueueRow as QueueRowView,
  UnlockNode,
} from '@/lib/ipc/types'
import {
  DropEdge,
  dropAnchor,
  dropEdge,
  stepAnchor,
  stepDirection,
} from '@/lib/plan/queueDrop'
import { queueExtras } from '@/lib/plan/queueExtras'
import { knownText } from '@/lib/graph/achievementNode'
import { rowId, stoppedUnder } from '@/lib/plan/queueRows'
import { rowModel } from '@/lib/plan/rowModel'

import type { QueueMove } from '@/stores/queue'
import QueueFootnotes from './QueueFootnotes.vue'

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

interface Drop {
  index: number
  edge: DropEdge
}

const rowSelector = '[data-queue-row]'

const list = ref<HTMLElement | null>(null)

const ids = computed(() => props.rows.map(rowId))

const rowElements = (): HTMLElement[] =>
  list.value ? [...list.value.querySelectorAll<HTMLElement>(rowSelector)] : []

// The half of the row under the pointer decides the edge; `dropAnchor` then says whether that
// edge means anything — the gaps either side of the dragged row are where it already is.
const resolve = (p: Point, boxes: Box[]): Drop | null => {
  const index = boxAt(boxes, p, Axis.Y)
  const box = index === null ? undefined : boxes[index]
  if (index === null || !box) return null
  return { index, edge: dropEdge(p.y, box.top, box.height) }
}

const drag = useDragList<Drop>({
  axis: Axis.Y,
  container: list,
  items: rowElements,
  enabled: () => !props.busy,
  resolve,
  commit: (from, landing) => {
    const anchor = landing
      ? dropAnchor(ids.value, from, landing.index, landing.edge)
      : null
    const moved = ids.value[from]
    if (anchor && moved !== undefined) emit('move', moved, anchor.after)
  },
})

// The gap the line is drawn in, only for a drop that would move something.
const gap = computed((): number | null => {
  const landing = drag.drop.value
  const from = drag.from.value
  if (!drag.moving.value || !landing || from === null) return null
  if (!dropAnchor(ids.value, from, landing.index, landing.edge)) return null
  return landing.edge === DropEdge.Above ? landing.index : landing.index + 1
})

// What each row draws, worked out once per answer from the queue. In the template these were
// two calls per row per render, and `queueExtras` reads the whole queue for each row: O(n²) on
// every render, and a drag renders on every pointer move.
const drawn = computed(() =>
  props.rows.map((row) => ({
    row,
    model: rowModel(row.node, t),
    extras: queueExtras(row, props.rows),
  })),
)

const grabbed = computed(() =>
  drag.from.value === null ? null : drawn.value[drag.from.value],
)

const onStep = (index: number, e: KeyboardEvent) => {
  const direction = stepDirection(e.key, e.altKey)
  if (!direction || props.busy) return
  e.preventDefault()
  const anchor = stepAnchor(ids.value, index, direction)
  const moved = ids.value[index]
  if (anchor && moved !== undefined) emit('move', moved, anchor.after)
}

// What the band says: how to drag, what a drop will do, or where the last move stopped and why.
const hint = computed((): string => {
  if (drag.moving.value) return t('plan.hint.dragging')
  const last = props.lastMove
  const wall = last
    ? stoppedUnder(props.rows, last.achievement, last.after)
    : null
  if (!wall) return t('plan.hint.idle')
  const name =
    knownText(wall.node) ?? t('plan.achievementNumbered', { id: rowId(wall) })
  return t('plan.hint.stoppedUnder', { name })
})
</script>

<template>
  <Card>
    <CardHeader class="flex-wrap">
      <!-- The count the page used to carry at the top, on the thing it counts (spec §5). -->
      <CardTitle
        >{{ t('plan.queueCount') }}
        <span class="text-subtle-foreground tabular-nums">{{
          rows.length
        }}</span></CardTitle
      >
      <span class="text-caption text-foreground-soft">{{ hint }}</span>
    </CardHeader>
    <div
      v-if="rows.length > 0"
      ref="list"
      class="flex flex-col py-1"
      @pointermove="drag.move"
      @pointerup="drag.end"
      @pointercancel="drag.end"
    >
      <div
        v-for="({ row, model, extras }, index) in drawn"
        :key="rowId(row)"
        data-queue-row
        class="relative border-b border-hairline last:border-b-0"
      >
        <span
          v-if="gap === index"
          class="absolute inset-x-0 -top-px h-0.5 bg-primary"
        />
        <GoalRow
          :model="model"
          :node="row.node"
          :extras="extras"
          :position="index + 1"
          :dragging="drag.moving.value && drag.from.value === index"
          :busy="busy"
          @grab="drag.start(index, $event)"
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
    <!-- The lifted copy, drawn `busy` on purpose: its grip and its remove button are a picture
         of the row's, and must not answer a pointer that is in the middle of a drag. -->
    <DragGhost v-if="drag.ghost.value && grabbed" :box="drag.ghost.value">
      <GoalRow
        :model="grabbed.model"
        :node="grabbed.row.node"
        :extras="grabbed.extras"
        :position="(drag.from.value ?? 0) + 1"
        :dragging="false"
        :busy="true"
      />
    </DragGhost>
    <QueueFootnotes
      :diagnostics="diagnostics"
      :nodes="nodes"
      :busy="busy"
      @remove="emit('remove', $event)"
    />
  </Card>
</template>
