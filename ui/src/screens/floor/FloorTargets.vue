<script setup lang="ts">
import { computed } from 'vue'
import {
  ToggleGroup,
  ToggleGroupItem,
  ToggleGroupType,
} from '@/components/ui/toggle-group'
import { useMessages } from '@/i18n'
import { TARGET_ORDER } from '@/lib/floor/cellView'
import { targetFill } from '@/lib/floor/targets'
import { TargetView } from '@/lib/ipc/types'
import type { FloorSolutionView } from '@/lib/ipc/types'

// Which of the three the grid is answering, and nothing else about it.
//
// **One at a time, and never none.** Three answers over one 2rem square was the first design
// and it failed in a window twice — as pips in the corners, then as bands side by side — so
// this is a choice of one rather than three switches. It cannot be emptied: a candidate only
// ever sits on a cell nobody painted, so showing the answer hides nothing of the drawing, and
// "none" would only be the tool declining to speak.
//
// Each carries its count, which is what the other two keep saying while you are looking at the
// third — and it is how "you are not looking at it" stays different from "there is nothing
// there", which used to be the same picture.

const props = defineProps<{
  solutions: FloorSolutionView[]
  shown: TargetView
}>()
const emit = defineEmits<{ show: [target: TargetView] }>()
const { t } = useMessages()

const countOf = computed(
  () =>
    new Map<TargetView, number>(
      props.solutions.map((solution) => [
        solution.target,
        solution.candidates.length,
      ]),
    ),
)

// Reka empties a single toggle group when you press the item that is already on. Here that
// would be a screen with no answer on it, which is a state this control has no reason to have,
// so the press lands on what is already chosen and nothing moves.
const choose = (value: unknown): void => {
  if (typeof value === 'string' && value !== '')
    emit('show', value as TargetView)
}
</script>

<template>
  <ToggleGroup
    class="w-full"
    :type="ToggleGroupType.Single"
    :model-value="shown"
    @update:model-value="choose"
  >
    <ToggleGroupItem
      v-for="target in TARGET_ORDER"
      :key="target"
      class="min-w-0 flex-1 gap-2"
      :value="target"
    >
      <span
        aria-hidden="true"
        class="size-3 shrink-0 rounded-cell"
        :class="targetFill[target]"
      />
      <!-- The short name, because three of these share the grid's 27.5rem and "Stanza segreta"
           three times over does not fit in it. The full name is on the rules below, where
           there is a line to write it on. -->
      <span class="truncate">{{ t(`floor.targetShort.${target}`) }}</span>
      <span class="tabular-nums">{{ countOf.get(target) ?? 0 }}</span>
    </ToggleGroupItem>
  </ToggleGroup>
</template>
