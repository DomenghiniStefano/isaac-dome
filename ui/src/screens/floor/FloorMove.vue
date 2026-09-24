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
import { HelpTip } from '@/components/ui/tooltip'
import { useMessages } from '@/i18n'
import { Direction, shift } from '@/lib/floor/painting'
import type { PaintedCells } from '@/lib/floor/painting'

// Four arrows under the grid, laid out as a pad: the whole drawing, one cell over.
//
// It is there because a floor is drawn from memory and the middle is a guess. You paint the
// start room where you think it goes, the map grows, and half of it turns out to want more
// room on one side — without this the only way across is rubbing out the whole thing and
// drawing it again.
//
// **An arrow that would lose a room is disabled**, never a move that quietly drops it. There
// is no undo here, so a press too many would delete work with nothing left to say so. The grey
// arrow is also the explanation: it means one thing only, that something is against that edge.
//
// **A cross, not a row.** Four equal buttons in a line read as four unrelated commands, and the
// left-up-down-right order had to be learned; placed where they point, each arrow says its
// direction before its icon is read. They are the compact size because the pad is a control
// beside the canvas and not the canvas's subject — four full buttons were the heaviest thing
// under the grid. What they do is said by the mark beside them, since an arrow alone reads as
// "scroll", which is exactly what it is not.

const props = defineProps<{ cells: PaintedCells }>()
const emit = defineEmits<{ move: [to: PaintedCells] }>()
const { t } = useMessages()

// Each arrow's place on the pad, where it points: the up arrow on top, the down one at the
// bottom, left and right either side of the empty middle.
const arrows: { direction: Direction; icon: Component; place: string }[] = [
  {
    direction: Direction.Up,
    icon: ArrowUpIcon,
    place: 'col-start-2 row-start-1',
  },
  {
    direction: Direction.Left,
    icon: ArrowLeftIcon,
    place: 'col-start-1 row-start-2',
  },
  {
    direction: Direction.Right,
    icon: ArrowRightIcon,
    place: 'col-start-3 row-start-2',
  },
  {
    direction: Direction.Down,
    icon: ArrowDownIcon,
    place: 'col-start-2 row-start-3',
  },
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
  <div class="flex items-center gap-2">
    <div
      class="grid grid-cols-3 grid-rows-3 gap-floor-gap"
      role="group"
      :aria-label="t('floor.move.title')"
    >
      <Button
        v-for="move in moves"
        :key="move.direction"
        :class="move.place"
        :variant="ButtonVariant.Outline"
        :size="ButtonSize.IconCompact"
        :disabled="move.to === null"
        :aria-label="t(`floor.move.${move.direction}`)"
        @click="move.to && emit('move', move.to)"
      >
        <component :is="move.icon" />
      </Button>
    </div>
    <HelpTip :label="t('floor.move.title')">{{ t('floor.move.note') }}</HelpTip>
  </div>
</template>
