<script setup lang="ts">
import {
  ArrowDownIcon,
  ArrowLeftIcon,
  ArrowRightIcon,
  ArrowUpIcon,
} from '@lucide/vue'
import type { Component } from 'vue'
import { computed } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { useMessages } from '@/i18n'
import { Direction, shift } from '@/lib/floor/painting'
import type { PaintedCells } from '@/lib/floor/painting'

// Four arrows under the grid: the whole drawing, one cell over.
//
// It is there because a floor is drawn from memory and the middle is a guess. You paint the
// start room where you think it goes, the map grows, and half of it turns out to want more
// room on one side — without this the only way across is rubbing out the whole thing and
// drawing it again.
//
// **An arrow that would lose a room is disabled**, never a move that quietly drops it. There
// is no undo here, so a press too many would delete work with nothing left to say so. The grey
// arrow is also the explanation: it means one thing only, that something is against that edge.

const props = defineProps<{ cells: PaintedCells }>()
const emit = defineEmits<{ move: [to: PaintedCells] }>()
const { t } = useMessages()

const arrows: { direction: Direction; icon: Component }[] = [
  { direction: Direction.Left, icon: ArrowLeftIcon },
  { direction: Direction.Up, icon: ArrowUpIcon },
  { direction: Direction.Down, icon: ArrowDownIcon },
  { direction: Direction.Right, icon: ArrowRightIcon },
]

// The four answers, worked out once per drawing rather than twice per arrow: the button needs
// to know whether the move is possible and, if it is, what it lands on, and asking the same
// question two ways is how the two end up disagreeing.
const moves = computed(() =>
  arrows.map((arrow) => ({
    ...arrow,
    to: shift(props.cells, arrow.direction),
  })),
)
</script>

<template>
  <div class="flex gap-1">
    <Button
      v-for="move in moves"
      :key="move.direction"
      :variant="ButtonVariant.Outline"
      :size="ButtonSize.Icon"
      :disabled="move.to === null"
      :aria-label="t(`floor.move.${move.direction}`)"
      @click="move.to && emit('move', move.to)"
    >
      <component :is="move.icon" />
    </Button>
  </div>
</template>
