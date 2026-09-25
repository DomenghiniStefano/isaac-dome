import { computed, ref, shallowRef, watch } from 'vue'
import { useTabView } from '@/composables/useTabView'
import { floorCandidates } from '@/lib/ipc/floor'
import { paintStroke, emptyCells } from '@/lib/floor/painting'
import type { PaintedCells } from '@/lib/floor/painting'
import type { FloorView, RoomKindView, TargetView } from '@/lib/ipc/types'
import { floorView } from './tabView'

// The drawing of the tab being shown, and the rules' answer about it.
//
// The drawing is the tab's reading (`tabView.ts`), so it travels with the tab; the answer is
// not, because it is worked out from the drawing and asked again whenever the drawing arrives —
// on mount, and when the tab under a mounted screen changes. Storing it would be storing an
// answer from a version of the rules that may no longer be the one running.
export const useFloorDrawing = () => {
  const { reading } = useTabView(floorView)
  const view = shallowRef<FloorView | null>(null)
  const failed = ref(false)

  // An answer is kept only if the drawing it answers is still the one on screen. Paint replaces
  // the cells rather than editing them, so "the same array" is "the same drawing": a slow answer
  // for a stroke already drawn over, or for the tab you just left, is thrown away.
  const solve = async (): Promise<void> => {
    const asked = reading.value.cells
    try {
      const answer = await floorCandidates(asked)
      if (reading.value.cells !== asked) return
      view.value = answer
      failed.value = false
    } catch {
      if (reading.value.cells !== asked) return
      view.value = null
      failed.value = true
    }
  }

  // A new reading object is a new tab (or a new entry in this one): `useTabView` replaces it
  // whole, where painting only ever replaces its cells.
  watch(
    () => reading.value,
    () => void solve(),
  )
  // The grid answers from the moment it opens: an empty floor is a diagnostic, not a blank.
  void solve()

  const redraw = async (cells: PaintedCells): Promise<void> => {
    reading.value.cells = cells
    await solve()
  }

  // Paint lands at once; the rules are asked afterwards, once, in `settle`. A drag is many cells
  // and one question: a `solve` per cell the pointer brushes past is a round trip to Rust for an
  // answer about a corridor that is still being drawn.
  const paint = (path: number[]): void => {
    reading.value.cells = paintStroke(
      reading.value.cells,
      path,
      reading.value.brush,
    )
  }

  return {
    cells: computed(() => reading.value.cells),
    shown: computed(() => reading.value.shown),
    brush: computed(() => reading.value.brush),
    view,
    failed,
    paint,
    settle: solve,
    // Already worked out by `shift`, which is also what decided the arrow could be pressed.
    move: redraw,
    // Rubbing out is painting with no brush: one path through the grid, not a second one.
    erase: (cell: number) =>
      redraw(paintStroke(reading.value.cells, [cell], null)),
    clear: () => redraw(emptyCells()),
    show: (target: TargetView): void => {
      reading.value.shown = target
    },
    pick: (brush: RoomKindView): void => {
      reading.value.brush = brush
    },
  }
}
