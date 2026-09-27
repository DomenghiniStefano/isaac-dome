<script setup lang="ts" generic="T">
import {
  useDebounceFn,
  useEventListener,
  useResizeObserver,
} from '@vueuse/core'
import { computed, inject, nextTick, onMounted, ref, watch } from 'vue'
import { useScaledRows } from '@/composables/useScaledRows'
import { Timing } from '@/lib/constants/timing'
import { offsetToApply } from '@/lib/scale/scrollOffset'
import type { ScrollOffset } from '@/lib/scale/scrollOffset'
import {
  listScrollTop,
  pageScrollTop,
  totalHeightPx,
  visibleRows,
  type VisibleRow,
} from '@/lib/scale/virtualRows'
import { PageScroller } from './pageScroller'

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

// The box this list scrolls in: its own, or the screen's when the screen scrolls as a whole
// (`PageScroller`). With the page, the rows are offset by what sits above them in it — the
// band, the filters — measured, not assumed, since a filter panel that opens moves them.
const page = inject(PageScroller, null)
const own = ref<HTMLElement | null>(null)
const bodyEl = ref<HTMLElement | null>(null)
const scroller = computed(() => page?.value ?? own.value)
const margin = ref(0)
const measureMargin = (): void => {
  const outer = page?.value
  if (!outer || !bodyEl.value) {
    margin.value = 0
    return
  }
  margin.value =
    bodyEl.value.getBoundingClientRect().top -
    outer.getBoundingClientRect().top +
    outer.scrollTop
}
onMounted(measureMargin)
const pageContent = computed(
  () => (page?.value?.firstElementChild as HTMLElement | null) ?? null,
)
useResizeObserver(pageContent, measureMargin)

// 641 rows on Unlock, drawn as many as fit plus a margin: TanStack Virtual positions them,
// and the count is a getter rather than a number so a new filter's count reaches the
// virtualizer instead of the one it was built with.
const virtualizer = useScaledRows({
  count: () => props.rows.length,
  scroller,
  rowPx: props.rowPx,
  overscan: props.overscan,
  scrollMargin: () => margin.value,
})

const visible = computed(() =>
  visibleRows(virtualizer.value.getVirtualItems(), props.rows, margin.value),
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
    measureMargin()
    if (scroller.value)
      scroller.value.scrollTop = page ? pageScrollTop(top, margin.value) : top
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
// With the page as the scroller the position kept is still the list's own, not the page's.
const onScroll = useDebounceFn(() => {
  const box = scroller.value
  if (box)
    emit('offsetChange', {
      top: page ? listScrollTop(box.scrollTop, margin.value) : box.scrollTop,
      rows: props.rows.length,
    })
}, Timing.ViewWrite)
useEventListener(() => page?.value ?? null, 'scroll', onScroll, {
  passive: true,
})
</script>

<template>
  <!-- The height is what is left, not a number (spec 3.13a §4): this used to cap at a 35rem token,
       which made a tall window show the same short list as a short one. Every ancestor up to the
       page box needs `min-h-0`, or `flex-1` grows to fit the rows instead of fitting the space and
       the list pushes the screen — which fails nothing and reads as a bug in the virtualizer.
       With a page scroller the box is only the rows' height: the page is what scrolls. -->
  <div
    ref="own"
    :class="page ? 'relative' : 'min-h-0 flex-1 overflow-auto'"
    @scroll="onScroll"
  >
    <div ref="bodyEl" :style="body" class="relative h-(--virtual-rows-total)">
      <slot :visible="visible" />
    </div>
  </div>
</template>
