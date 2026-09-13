<script setup lang="ts">
import { PlusIcon } from '@lucide/vue'
import { computed, nextTick, ref } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { DragGhost } from '@/components/ui/drag'
import { useDragList } from '@/composables/useDragList'
import { useMessages } from '@/i18n'
import { EventKey } from '@/lib/constants/eventKeys'
import { Axis, boxAt } from '@/lib/drag/dragList'
import type { Box, Point } from '@/lib/drag/dragList'
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
      <TabItem
        v-for="(tab, index) in tabs"
        :key="tab.id"
        :tab="tab"
        :active="tab.id === activeId"
        :dragging="drag.moving.value && drag.from.value === index"
        :drop="drag.drop.value?.index === index ? drag.drop.value.side : null"
        @pointerdown="drag.start(index, $event)"
        @select="emit('select', tab.id)"
        @close="emit('close', tab.id)"
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
