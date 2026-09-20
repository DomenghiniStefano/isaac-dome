<script setup lang="ts">
import { computed } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { useMessages } from '@/i18n'
import { candidateFor, cellPosition } from '@/lib/floor/cellView'
import { CELLS, START, WIDTH } from '@/lib/floor/painting'
import type { PaintedCells } from '@/lib/floor/painting'
import { levelHeight, targetAura, targetFill } from '@/lib/floor/targets'
import { roomFill } from '@/lib/floor/rooms'
import type {
  FloorSolutionView,
  RoomKindView,
  TargetView,
} from '@/lib/ipc/types'
import RoomSymbol from './RoomSymbol.vue'

// The grid says two things at once, and keeping them apart is the whole design: the **fill and
// the drawing** are what you painted, the **level** is what the rules make of it. Before this
// it said one — a rank, or "painted", or "empty" — so fourteen room kinds came out one grey
// square and only the Secret Room's answer ever reached the map.
//
// The rules' half went through two shapes in front of a real window and both failed the same
// way. Four pips in the corners, one per target with its rank printed inside: four numbers
// that small are four numbers nobody reads. Then the cell split into a band per target, the
// rank still printed: better, still a number in a 2rem square, and still three answers laid
// over one cell.
//
// **One target at a time, and the rank is not written down.** A cell the rules allow fills
// with that target's colour, to the brim for the best place and less for each step after it.
// There is nothing to read: a fuller square is a better place, which is the sentence the
// screen exists to say.

const props = defineProps<{
  cells: PaintedCells
  solutions: FloorSolutionView[]
  shown: TargetView
  icons: Map<RoomKindView, string>
}>()
const emit = defineEmits<{ stroke: [path: number[]]; erase: [cell: number] }>()
const { t } = useMessages()

const indexes = Array.from({ length: CELLS }, (_, i) => i)

// Every cell's answer, built once per answer rather than searched per cell: 169 cells against
// a candidate list is the one place on this screen where that would show.
const candidates = computed(() =>
  indexes.map((cell) => candidateFor(cell, props.solutions, props.shown)),
)

const fillOf = (cell: number): string => {
  const kind = props.cells[cell]
  return kind === null || kind === undefined ? 'bg-floor-empty' : roomFill[kind]
}

// What the cell is called out loud. A cell index is not a position — "cell 97" connects to
// nothing on a drawing — so it is read the way it is looked at, by row and column.
//
// The answer is said here as well, because it is now drawn and never written: a height and a
// hue reach the eye and nothing else. The place in the order is the one thing a label can
// carry that the square cannot.
const nameOf = (cell: number): string => {
  const kind = props.cells[cell]
  const { row, column } = cellPosition(cell)
  const room =
    kind === null || kind === undefined
      ? t('floor.empty')
      : t(`floor.room.${kind}`)
  const name = t('floor.cell', { row, column, room })
  const candidate = candidates.value[cell]
  if (candidate === null) return name
  return t('floor.cellCandidate', {
    cell: name,
    target: t(`floor.target.${props.shown}`),
    rank: candidate.rank,
  })
}

// A stroke is one press and everything the pointer crossed before release, so dragging paints
// a corridor. The path is collected here and handed over whole: one answer per stroke, not one
// per cell.
let path: number[] = []
let painting = false

const start = (cell: number): void => {
  painting = true
  path = [cell]
}
const over = (cell: number): void => {
  if (painting) path.push(cell)
}
const end = (): void => {
  if (!painting) return
  painting = false
  if (path.length > 0) emit('stroke', path)
  path = []
}

// The right button rubs out, whatever the brush is. Erasing by picking "nothing" in the
// palette still works and is the keyboard's way in; this is the one the hand reaches for, and
// it costs a trip to the palette and back for every correction if it is not there.
const rub = (cell: number): void => {
  painting = false
  path = []
  emit('erase', cell)
}
</script>

<template>
  <div
    class="grid w-fit gap-floor-gap"
    :style="{
      gridTemplateColumns: `repeat(${WIDTH}, var(--spacing-floor-cell))`,
    }"
    @pointerup="end"
    @pointerleave="end"
  >
    <Button
      v-for="i in indexes"
      :key="i"
      :variant="ButtonVariant.Cell"
      :size="ButtonSize.Cell"
      class="relative"
      :class="fillOf(i)"
      :aria-label="nameOf(i)"
      :aria-current="i === START ? 'location' : undefined"
      @pointerdown="start(i)"
      @pointerenter="over(i)"
      @contextmenu.prevent="rub(i)"
    >
      <RoomSymbol
        v-if="cells[i]"
        :kind="cells[i]!"
        :url="icons.get(cells[i]!) ?? null"
      />
      <!-- Two layers, and they answer two different questions. The glow covers the cell and
           says it is in play at all; the level is anchored to the cell's floor and says how
           good a place it is. Both rise from the bottom, because a level hanging from the top
           would be read as something draining. -->
      <span
        v-if="candidates[i]"
        aria-hidden="true"
        class="absolute inset-0"
        :class="targetAura[shown]"
      >
        <span
          class="absolute inset-x-0 bottom-0"
          :class="targetFill[shown]"
          :style="{ height: levelHeight[candidates[i]!.step] }"
        />
      </span>
    </Button>
  </div>
</template>
