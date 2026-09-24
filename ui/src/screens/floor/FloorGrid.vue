<script setup lang="ts">
import { computed } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { useMessages } from '@/i18n'
import { candidateFor, cellPosition } from '@/lib/floor/cellView'
import { CELLS, START, WIDTH } from '@/lib/floor/painting'
import type { PaintedCells } from '@/lib/floor/painting'
import { roomFill } from '@/lib/floor/rooms'
import type {
  FloorSolutionView,
  RoomKindView,
  TargetView,
} from '@/lib/ipc/types'
import FloorRank from './FloorRank.vue'
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
const emit = defineEmits<{
  paint: [cells: number[]]
  settle: []
  erase: [cell: number]
}>()
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
// a corridor.
//
// **The paint lands as the pointer passes, not when it lifts.** It used to collect the path
// and hand it over whole, so a drag across ten cells was ten cells appearing at once at the
// end — the corridor was drawn blind, and a hand that had gone one cell too far only found out
// after letting go.
//
// What is still handed over once is the *question*: the rules are asked on release, in a
// single `settle`. Asking them per cell would be a round trip to Rust for every cell the
// pointer brushes past, and the answer for a corridor half-drawn is not an answer anyone is
// reading — the hand is still moving.
let painting = false

const start = (cell: number): void => {
  painting = true
  emit('paint', [cell])
}
const over = (cell: number): void => {
  if (painting) emit('paint', [cell])
}
const end = (): void => {
  if (!painting) return
  painting = false
  emit('settle')
}

// The right button rubs out, whatever the brush is. Erasing by picking "nothing" in the
// palette still works and is the keyboard's way in; this is the one the hand reaches for, and
// it costs a trip to the palette and back for every correction if it is not there.
const rub = (cell: number): void => {
  painting = false
  emit('erase', cell)
}
</script>

<template>
  <div
    class="grid w-fit grid-cols-floor gap-floor-gap"
    :style="{ '--floor-width': WIDTH }"
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
      <!-- Three layers, and they answer three different questions. The veil and its lit edge
           cover the cell and say it is in play at all; the level is anchored to the cell's
           floor and says how good a place it is — rising, because one hanging from the top
           would be read as something draining; the number says exactly which place, for
           whoever wants the answer rather than the impression of it.

           The digit carries its own halo instead of an ink chosen for what is under it: the
           middle of the cell is the bright fill at the first step and the dark veil at the
           third, and no single colour reads on both. -->
      <FloorRank
        v-if="candidates[i]"
        :target="shown"
        :step="candidates[i]!.step"
        :rank="candidates[i]!.rank"
      />
    </Button>
  </div>
</template>
