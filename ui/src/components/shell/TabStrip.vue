<script setup lang="ts">
import { PlusIcon } from '@lucide/vue'
import { computed, nextTick, ref, watch } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { DragGhost } from '@/components/ui/drag'
import { useDragList } from '@/composables/useDragList'
import { useMessages } from '@/i18n'
import { EventKey } from '@/lib/constants/eventKeys'
import { Axis, boxAt } from '@/lib/drag/dragList'
import type { Box, Point } from '@/lib/drag/dragList'
import { toClient } from '@/lib/window/tearOff'
import TabItem from './TabItem.vue'
import type { IncomingHover, TabView } from './tabs'
import { DropSide, TabDrag, TabRole, dropSide, moveIndex } from './tabs'

const props = withDefaults(
  defineProps<{
    tabs: TabView[]
    activeId: string | null
    // Optional because most of the app has no second window in sight — the Kit draws a strip
    // with no drag in flight, and nothing should have to say "nothing is arriving".
    incoming?: IncomingHover | null
  }>(),
  { incoming: null },
)
const emit = defineEmits<{
  select: [id: string]
  close: [id: string]
  move: [from: number, to: number]
  add: []
  aim: [index: number | null]
}>()
const { t } = useMessages()

interface Drop {
  index: number
  side: DropSide
}

const tabSelector = `[role="${TabRole}"]`

const strip = ref<HTMLElement | null>(null)

const tabElements = (): HTMLElement[] =>
  strip.value ? [...strip.value.querySelectorAll<HTMLElement>(tabSelector)] : []

// Which side of which tab the drop lands on. A pointer over the dragged tab itself means
// nothing to do; `dropSide` and `moveIndex` stay the strip's own semantics.
const resolve = (p: Point, boxes: Box[], from: number): Drop | null => {
  const index = boxAt(boxes, p, Axis.X)
  const box = index === null ? undefined : boxes[index]
  if (index === null || index === from || !box) return null
  return { index, side: dropSide(p.x, box.left, box.width) }
}

const drag = useDragList<Drop>({
  axis: Axis.X,
  container: strip,
  items: tabElements,
  threshold: TabDrag.Threshold,
  resolve,
  commit: (from, landing) => {
    if (!landing) return
    const to = moveIndex(from, landing.index, landing.side)
    if (to !== from) emit('move', from, to)
  },
})

// The tab the ghost draws: the grabbed one, as it is — an inactive tab lifted as if it were
// the active one would read as a different tab.
const grabbed = computed(() =>
  drag.from.value === null ? null : props.tabs[drag.from.value],
)

// A tab arriving from another window. Only this window can turn the desktop point into a gap
// in its own strip — it owns the rectangles — and the gap the marker is drawn in **is** where
// the drop lands: one computation, so what you saw is what you get. The index is handed back
// so the window can dock there without measuring anything a second time.
const incomingGap = computed((): number | null => {
  const hover = props.incoming
  if (!hover) return null
  const p = toClient(hover.at, hover.window)
  const boxes = tabElements().map((el) => {
    const r = el.getBoundingClientRect()
    return { left: r.left, top: r.top, width: r.width, height: r.height }
  })
  const index = boxAt(boxes, p, Axis.X)
  const box = index === null ? undefined : boxes[index]
  if (index === null || !box) return props.tabs.length
  return dropSide(p.x, box.left, box.width) === DropSide.Before
    ? index
    : index + 1
})

watch(incomingGap, (gap) => emit('aim', gap))

const neighbour = (key: string): number | null => {
  const index = props.tabs.findIndex((tab) => tab.id === props.activeId)
  if (index < 0) return null
  if (key === EventKey.ArrowRight)
    return Math.min(index + 1, props.tabs.length - 1)
  if (key === EventKey.ArrowLeft) return Math.max(index - 1, 0)
  return null
}

const onKeydown = (e: KeyboardEvent) => {
  const next = neighbour(e.key)
  const tab = next === null ? undefined : props.tabs[next]
  if (next === null || !tab) return
  e.preventDefault()
  emit('select', tab.id)
  void nextTick(() => tabElements()[next]?.focus())
}
</script>

<template>
  <div class="flex min-w-0 items-end gap-px pl-1.75">
    <div
      ref="strip"
      role="tablist"
      class="flex min-w-0 items-end gap-px"
      @pointermove="drag.move"
      @pointerup="drag.end"
      @pointercancel="drag.end"
      @keydown="onKeydown"
    >
      <template v-for="(tab, index) in tabs" :key="tab.id">
        <!-- Where a tab from another window would land: the same 2px primary line the queue
             draws, in the gap the marker is aimed at. -->
        <span
          v-if="incomingGap === index"
          class="h-tab w-0.5 shrink-0 self-end bg-primary"
        />
        <TabItem
          :tab="tab"
          :active="tab.id === activeId"
          :dragging="drag.moving.value && drag.from.value === index"
          :drop="drag.drop.value?.index === index ? drag.drop.value.side : null"
          @pointerdown="drag.start(index, $event)"
          @select="emit('select', tab.id)"
          @close="emit('close', tab.id)"
        />
      </template>
      <span
        v-if="incomingGap === tabs.length"
        class="h-tab w-0.5 shrink-0 self-end bg-primary"
      />
    </div>
    <DragGhost v-if="drag.ghost.value && grabbed" :box="drag.ghost.value">
      <TabItem
        :tab="grabbed"
        :active="grabbed.id === activeId"
        :dragging="false"
        :drop="null"
      />
    </DragGhost>
    <Button
      :variant="ButtonVariant.Chrome"
      :size="ButtonSize.IconCompact"
      :aria-label="t('shell.newTab')"
      class="mb-px ml-0.75"
      @click="emit('add')"
    >
      <PlusIcon />
    </Button>
  </div>
</template>
