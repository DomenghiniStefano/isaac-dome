<script setup lang="ts">
import { computed } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { CELLS, START, WIDTH } from '@/lib/floor/painting'
import type { PaintedCells } from '@/lib/floor/painting'
import type { FloorCandidate } from '@/lib/ipc/types'

const props = defineProps<{
  cells: PaintedCells
  candidates: FloorCandidate[]
}>()
const emit = defineEmits<{ stroke: [path: number[]] }>()

// A cell's rank, or nothing. Built once per answer rather than searched per cell: 169 cells
// against a candidate list is the one place on this screen where that shows.
const rankOf = computed(() => {
  const m = new Map<number, number>()
  for (const c of props.candidates) m.set(c.cell, c.rank)
  return m
})

const indexes = Array.from({ length: CELLS }, (_, i) => i)

// The fill is the whole answer: an unpainted cell, a room, or a cell the rules allow, in the
// rank's own step. Ranks past the third share the last step — the scale says "less likely",
// not "exactly how much less", which is a claim no rule here makes.
const fillOf = (cell: number): string => {
  const rank = rankOf.value.get(cell)
  if (rank === 0) return 'bg-floor-candidate-first'
  if (rank === 1) return 'bg-floor-candidate-second'
  if (rank !== undefined) return 'bg-floor-candidate-third'
  return props.cells[cell] === null ? 'bg-floor-empty' : 'bg-floor-room'
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
      :class="fillOf(i)"
      :aria-current="i === START ? 'location' : undefined"
      @pointerdown="start(i)"
      @pointerenter="over(i)"
    >
      <span v-if="rankOf.get(i) !== undefined">{{ rankOf.get(i)! + 1 }}</span>
    </Button>
  </div>
</template>
