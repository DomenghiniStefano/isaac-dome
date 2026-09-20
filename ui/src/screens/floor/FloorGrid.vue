<script setup lang="ts">
import { computed } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { useMessages } from '@/i18n'
import { cellPosition, pipsFor } from '@/lib/floor/cellView'
import { CELLS, START, WIDTH } from '@/lib/floor/painting'
import type { PaintedCells } from '@/lib/floor/painting'
import { cornerAt, pipFill } from '@/lib/floor/pips'
import { roomFill } from '@/lib/floor/rooms'
import type {
  FloorSolutionView,
  RoomKindView,
  TargetView,
} from '@/lib/ipc/types'
import RoomSymbol from './RoomSymbol.vue'

// The grid says two things at once, and keeping them apart is the whole design: the **fill and
// the drawing** are what you painted, the **pips in the corners** are what the rules make of
// it. Before this it said one — a rank, or "painted", or "empty" — so fourteen room kinds came
// out one grey square and only the Secret Room's answer ever reached the map.

const props = defineProps<{
  cells: PaintedCells
  solutions: FloorSolutionView[]
  shown: TargetView[]
  icons: Map<RoomKindView, string>
}>()
const emit = defineEmits<{ stroke: [path: number[]]; erase: [cell: number] }>()
const { t } = useMessages()

const indexes = Array.from({ length: CELLS }, (_, i) => i)

// Every cell's pips, built once per answer rather than searched per cell: 169 cells against
// three candidate lists is the one place on this screen where that would show.
const pips = computed(() =>
  indexes.map((cell) => pipsFor(cell, props.solutions, props.shown)),
)

const fillOf = (cell: number): string => {
  const kind = props.cells[cell]
  return kind === null || kind === undefined ? 'bg-floor-empty' : roomFill[kind]
}

// What the cell is called out loud. A cell index is not a position — "cell 97" connects to
// nothing on a drawing — so it is read the way it is looked at, by row and column.
const nameOf = (cell: number): string => {
  const kind = props.cells[cell]
  const { row, column } = cellPosition(cell)
  const room =
    kind === null || kind === undefined
      ? t('floor.empty')
      : t(`floor.room.${kind}`)
  return t('floor.cell', { row, column, room })
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
      <span
        v-for="pip in pips[i]"
        :key="pip.target"
        class="absolute flex size-floor-pip items-center justify-center rounded-cell text-micro text-floor-pip-foreground tabular-nums"
        :class="[cornerAt[pip.corner], pipFill[pip.target][pip.step]]"
        >{{ pip.rank }}</span
      >
    </Button>
  </div>
</template>
