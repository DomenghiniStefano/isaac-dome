import { useElementSize } from '@vueuse/core'
import { computed } from 'vue'
import type { ComputedRef, Ref } from 'vue'
import { remToPx, wikiCardPx } from '@/lib/scale/rows'
import { useSettingsStore } from '@/stores/settings'

// The 12px gap the grid draws between cards (`gap-3`), read in device pixels the same way a
// card's own width is: the 4px grid's third step, at the interface's own scale.
const CardGapRem = 0.75

// As many cards as fit a box this wide, gap included, never fewer than one: a card grid a
// window too narrow for even one card still shows the first one, the same way a table never
// hides its only column.
export const columnsForWidth = (
  widthPx: number,
  cardPx: number,
  gapPx: number,
): number => {
  if (cardPx <= 0) return 1
  const columns = Math.floor((widthPx + gapPx) / (cardPx + gapPx))
  return Math.max(1, columns)
}

// How many cards a row of the grid holds, from the box the grid is drawn in and the
// interface's own scale — the same `remToPx` reasoning `useScaledRows` already applies to a
// table row's height, applied here to a card's width instead. Not pure — it measures a live
// element — so the arithmetic above is what a test can hold.
export const useCardColumns = (
  box: Ref<HTMLElement | null>,
): ComputedRef<number> => {
  const settings = useSettingsStore()
  const { width } = useElementSize(box)
  return computed(() => {
    const cardPx = wikiCardPx(settings.scale)
    const gapPx = remToPx(CardGapRem, settings.scale)
    return columnsForWidth(width.value, cardPx, gapPx)
  })
}
