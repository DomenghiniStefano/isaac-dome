<script setup lang="ts">
import { PlusIcon } from '@lucide/vue'
import { computed, nextTick, ref, watch } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { DragGhost } from '@/components/ui/drag'
import { useTabDrag } from '@/composables/useTabDrag'
import { useMessages } from '@/i18n'
import { EventKey } from '@/lib/constants/eventKeys'
import { Axis, boxAt } from '@/lib/drag/dragList'
import type { Point } from '@/lib/drag/dragList'
import { toClient } from '@/lib/window/tearOff'
import TabItem from './TabItem.vue'
import type { IncomingHover, TabView } from './tabs'
import { DropSide, TabRole, dropSide } from './tabs'

const props = withDefaults(
  defineProps<{
    tabs: TabView[]
    activeId: string | null
    // Optional because most of the app has no second window in sight — the Kit draws a strip
    // with no drag in flight, and nothing should have to say "nothing is arriving".
    incoming?: IncomingHover | null
    // A window holding one tab is that tab: it has nothing to tear off.
    canTear?: boolean
  }>(),
  { incoming: null, canTear: false },
)
const emit = defineEmits<{
  select: [id: string]
  close: [id: string]
  move: [from: number, to: number]
  add: []
  aim: [index: number | null]
  giveTo: [index: number, label: string, at: Point]
  openWith: [index: number, at: Point]
}>()
const { t } = useMessages()

const tabSelector = `[role="${TabRole}"]`

const strip = ref<HTMLElement | null>(null)

const tabElements = (): HTMLElement[] =>
  strip.value ? [...strip.value.querySelectorAll<HTMLElement>(tabSelector)] : []

// Inside the strip this is the reorder; past the tear band the same gesture becomes a window.
// Which of the two is `useTabDrag`'s business — the strip only says what its tabs are and what
// each ending means.
const { drag, detached } = useTabDrag({
  strip,
  items: tabElements,
  labelOf: (index) => props.tabs[index]?.label ?? '',
  canTear: () => props.canTear,
  reorder: (from, to) => emit('move', from, to),
  giveTo: (index, label, at) => emit('giveTo', index, label, at),
  openWith: (index, at) => emit('openWith', index, at),
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
    <!-- Outside the window the preview has taken over: two things following one cursor would
         be two answers to "where is the tab". -->
    <DragGhost
      v-if="drag.ghost.value && grabbed && !detached"
      :box="drag.ghost.value"
    >
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
