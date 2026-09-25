<script setup lang="ts">
import { PlusIcon } from '@lucide/vue'
import { computed, nextTick, ref, watch } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { DragGhost } from '@/components/ui/drag'
import { useTabDrag } from '@/composables/useTabDrag'
import { useMessages } from '@/i18n'
import type { Point } from '@/lib/drag/dragList'
import { toClient } from '@/lib/window/tearOff'
import TabItem from './TabItem.vue'
import type { IncomingHover, TabView } from '@/lib/shell/tabs'
import { TabRole, arrivalGap, neighbourIndex } from '@/lib/shell/tabs'

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
  // The tab left the strip, landed, or came back: the three endings of a tear-off.
  lift: [index: number]
  settle: [target: string | null, at: Point, origin: Point]
  putBack: []
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
  reorder: (from, to) => emit('move', from, to),
  lift: (index) => {
    if (!props.tabs[index]) return false
    emit('lift', index)
    return true
  },
  settle: (target, at, origin) => emit('settle', target, at, origin),
  putBack: () => emit('putBack'),
})

// The tab the ghost draws: the grabbed one, as it is — an inactive tab lifted as if it were
// the active one would read as a different tab.
const grabbed = computed(() =>
  drag.from.value === null ? null : props.tabs[drag.from.value],
)

// A tab arriving from another window. Only this window can turn the desktop point into a gap
// in its own strip — it owns the rectangles. The index is handed back so the window can dock
// there without measuring anything a second time.
const incomingGap = computed((): number | null => {
  const hover = props.incoming
  if (!hover) return null
  const boxes = tabElements().map((el) => {
    const r = el.getBoundingClientRect()
    return { left: r.left, top: r.top, width: r.width, height: r.height }
  })
  return arrivalGap(boxes, toClient(hover.at, hover.window), props.tabs.length)
})

watch(incomingGap, (gap) => emit('aim', gap))

// A tab that became active without being clicked — restored from a session, docked from another
// window, reached with the arrow keys — can be outside the scrolled strip. Active and invisible
// is the one state a tab bar must not have.
watch(
  () => props.activeId,
  async () => {
    await nextTick()
    const index = props.tabs.findIndex((tab) => tab.id === props.activeId)
    tabElements()[index]?.scrollIntoView({
      block: 'nearest',
      inline: 'nearest',
    })
  },
  { immediate: true },
)

const onKeydown = (e: KeyboardEvent) => {
  const next = neighbourIndex(
    props.tabs.map((tab) => tab.id),
    props.activeId,
    e.key,
  )
  const tab = next === null ? undefined : props.tabs[next]
  if (next === null || !tab) return
  e.preventDefault()
  emit('select', tab.id)
  void nextTick(() => tabElements()[next]?.focus())
}
</script>

<template>
  <div class="flex min-w-0 items-end gap-px pl-2">
    <div
      ref="strip"
      role="tablist"
      class="strip-scroll flex min-w-0 items-end gap-px"
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
      class="mb-px ml-1"
      @click="emit('add')"
    >
      <PlusIcon />
    </Button>
  </div>
</template>
