import { useVirtualizer } from '@tanstack/vue-virtual'
import { computed, watch } from 'vue'
import type { ComputedRef, Ref } from 'vue'
import { useSettingsStore } from '@/stores/settings'

// A virtualizer keeps the sizes it measured. When the interface changes size with the table
// already on screen, `estimateSize` answers the new height but the rows keep the old offsets:
// they overlap, or leave gaps, and nothing fails — the same shape of defect cycle 3.5c found
// in the fixed 40. Watching the scale is the other half of that fix: the row's height is a
// token, so the scale is what says when to measure again.
//
// Not pure — it installs a `watch` — but separate from the virtualizer, so it can be stated in a
// test without a DOM.
export const remeasureOnScale = (
  scale: Ref<number> | ComputedRef<number>,
  measure: () => void,
): void => {
  watch(scale, () => measure())
}

export interface ScaledRowsOptions {
  count: () => number
  scroller: Ref<HTMLElement | null>
  /** The row's height in device pixels at a given scale, from its own token. */
  rowPx: (percent: number) => number
  overscan?: number
}

// What the four virtualized tables share: as many rows as fit plus a margin, positioned with
// the same number the row's token is drawn from, and measured again when the scale moves.
export const useScaledRows = ({
  count,
  scroller,
  rowPx,
  overscan = 8,
}: ScaledRowsOptions) => {
  const settings = useSettingsStore()
  const virtualizer = useVirtualizer(
    computed(() => ({
      count: count(),
      getScrollElement: () => scroller.value,
      estimateSize: () => rowPx(settings.scale),
      overscan,
    })),
  )
  remeasureOnScale(
    computed(() => settings.scale),
    () => virtualizer.value.measure(),
  )
  return virtualizer
}
