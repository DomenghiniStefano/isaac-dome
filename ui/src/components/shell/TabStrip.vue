<script setup lang="ts">
import { PlusIcon } from '@lucide/vue'
import { nextTick, ref } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { useMessages } from '@/i18n'
import { EventKey } from '@/lib/constants/eventKeys'
import TabItem from './TabItem.vue'
import type { DropSide, TabView } from './tabs'
import { TabDrag, TabRole, dropSide, moveIndex } from './tabs'

const props = defineProps<{ tabs: TabView[]; activeId: string | null }>()
const emit = defineEmits<{
  select: [id: string]
  close: [id: string]
  move: [from: number, to: number]
  add: []
}>()
const { t } = useMessages()

interface Drag {
  from: number
  startX: number
  moving: boolean
}
interface Drop {
  index: number
  side: DropSide
}

const tabSelector = `[role="${TabRole}"]`

const strip = ref<HTMLElement | null>(null)
const drag = ref<Drag | null>(null)
const drop = ref<Drop | null>(null)
// Where the tabs are, read once when a press becomes a drag: nothing moves until the drag
// ends, and a layout read for every tab on every pointermove would be waste.
let tabRects: DOMRect[] = []

const tabElements = (): HTMLElement[] =>
  strip.value ? [...strip.value.querySelectorAll<HTMLElement>(tabSelector)] : []

const dropAt = (x: number, from: number): Drop | null => {
  for (const [index, r] of tabRects.entries()) {
    if (x >= r.left && x < r.right)
      return index === from
        ? null
        : { index, side: dropSide(x, r.left, r.width) }
  }
  return null
}

// A press becomes a drag only past the threshold, and only then is the pointer captured:
// a capture from the first pixel would send the click to the strip instead of the tab.
const onPointerDown = (index: number, e: PointerEvent) => {
  if (e.button !== 0) return
  drag.value = { from: index, startX: e.clientX, moving: false }
}

const onPointerMove = (e: PointerEvent) => {
  const d = drag.value
  if (!d) return
  if (!d.moving) {
    if (Math.abs(e.clientX - d.startX) < TabDrag.Threshold) return
    d.moving = true
    tabRects = tabElements().map((el) => el.getBoundingClientRect())
    strip.value?.setPointerCapture(e.pointerId)
  }
  drop.value = dropAt(e.clientX, d.from)
}

const onPointerUp = (e: PointerEvent) => {
  const d = drag.value
  const target = drop.value
  drag.value = null
  drop.value = null
  tabRects = []
  if (strip.value?.hasPointerCapture(e.pointerId))
    strip.value.releasePointerCapture(e.pointerId)
  if (!d?.moving || !target) return
  const to = moveIndex(d.from, target.index, target.side)
  if (to !== d.from) emit('move', d.from, to)
}

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
      @pointermove="onPointerMove"
      @pointerup="onPointerUp"
      @pointercancel="onPointerUp"
      @keydown="onKeydown"
    >
      <TabItem
        v-for="(tab, index) in tabs"
        :key="tab.id"
        :tab="tab"
        :active="tab.id === activeId"
        :dragging="drag?.moving === true && drag.from === index"
        :drop="drop?.index === index ? drop.side : null"
        @pointerdown="onPointerDown(index, $event)"
        @select="emit('select', tab.id)"
        @close="emit('close', tab.id)"
      />
    </div>
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
