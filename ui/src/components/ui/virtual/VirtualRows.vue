<script setup lang="ts" generic="T">
import { useDebounceFn } from '@vueuse/core'
import { computed, nextTick, ref, watch } from 'vue'
import { useScaledRows } from '@/composables/useScaledRows'
import { Timing } from '@/lib/constants/timing'
import { offsetToApply } from '@/lib/scale/scrollOffset'
import type { ScrollOffset } from '@/lib/scale/scrollOffset'
import {
  totalHeightPx,
  visibleRows,
  type VisibleRow,
} from '@/lib/scale/virtualRows'

// What Unlock, the Collection, Search and the wiki's category list all repeated
// (`docs/BACKLOG.md` B43): the scroll box, the height read from the virtualizer, and the
// window of rows it currently wants drawn. The columns are not this component's job — N3
// keeps `UnlockTable` and `CollectionTable` two files on purpose, and this doesn't reopen
// that — a screen brings its own row markup through the default slot.
const props = defineProps<{
  rows: T[]
  /** The row's height in device pixels at a given scale, from its own token. */
  rowPx: (percent: number) => number
  overscan?: number
  /** Where this list was left, and the list that number was measured against (3.7a). */
  offset?: ScrollOffset | null
}>()

const emit = defineEmits<{ offsetChange: [ScrollOffset] }>()
defineSlots<{
  default(props: { visible: VisibleRow<T>[] }): unknown
}>()

const scroller = ref<HTMLElement | null>(null)

// 641 rows on Unlock, drawn as many as fit plus a margin: TanStack Virtual positions them,
// and the count is a getter rather than a number so a new filter's count reaches the
// virtualizer instead of the one it was built with.
const virtualizer = useScaledRows({
  count: () => props.rows.length,
  scroller,
  rowPx: props.rowPx,
  overscan: props.overscan,
})

const visible = computed(() =>
  visibleRows(virtualizer.value.getVirtualItems(), props.rows),
)

// Geometry measured at runtime travels as a CSS variable, read by the utility below.
const body = computed(() => ({
  '--virtual-rows-total': totalHeightPx(virtualizer.value.getTotalSize()),
}))

// Restored **after** the rows are there: an offset into an empty list scrolls nothing, and the
// data arrives a tick after the component. Once only — a later change of the stored offset is
// this component's own echo coming back, not somebody moving the list.
let restored = false
watch(
  () => props.rows.length,
  async (rows) => {
    if (restored || rows === 0) return
    restored = true
    const top = offsetToApply(props.offset ?? null, rows)
    if (top === null) return
    await nextTick()
    if (scroller.value) scroller.value.scrollTop = top
  },
  { immediate: true },
)

// Moving to a row somebody else picked — today the find bar (B67). It is `scrollToIndex` and
// never `scrollIntoView`: of 733 rows only the twenty in view are nodes at all, so the node to
// scroll into does not exist until the virtualizer has been told to make it.
defineExpose({
  scrollToIndex: (index: number) =>
    virtualizer.value.scrollToIndex(index, { align: 'center' }),
})

// The length travels with the position, because that is what makes the position mean anything.
const onScroll = useDebounceFn(() => {
  if (scroller.value)
    emit('offsetChange', {
      top: scroller.value.scrollTop,
      rows: props.rows.length,
    })
}, Timing.ViewWrite)
</script>

<template>
  <!-- The height is what is left, not a number (spec 3.13a §4): this used to cap at a 35rem token,
       which made a tall window show the same short list as a short one. Every ancestor up to the
       page box needs `min-h-0`, or `flex-1` grows to fit the rows instead of fitting the space and
       the list pushes the screen — which fails nothing and reads as a bug in the virtualizer. -->
  <div ref="scroller" class="min-h-0 flex-1 overflow-auto" @scroll="onScroll">
    <div :style="body" class="relative h-(--virtual-rows-total)">
      <slot :visible="visible" />
    </div>
  </div>
</template>
