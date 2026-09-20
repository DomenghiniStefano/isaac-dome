<script setup lang="ts">
import { computed } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { useMessages } from '@/i18n'
import { RankStep } from '@/lib/floor/cellView'
import { pipFill } from '@/lib/floor/pips'
import { TargetView } from '@/lib/ipc/types'
import type { FloorSolutionView } from '@/lib/ipc/types'

// The three targets, switched on and off. Showing all three at once is the point of the fixed
// corners; being able to stop showing them is what keeps that from becoming a crowd.
//
// Each carries its count, because a target switched off and a target with nothing to say look
// identical on the grid — a filter that hides the difference between "you are not looking" and
// "there is nothing there" answers the wrong question.

const props = defineProps<{
  solutions: FloorSolutionView[]
  shown: TargetView[]
}>()
const emit = defineEmits<{ toggle: [target: TargetView] }>()
const { t } = useMessages()

const targets = [
  TargetView.Secret,
  TargetView.SuperSecret,
  TargetView.UltraSecret,
]

const countOf = computed(() => {
  const counts = new Map<TargetView, number>()
  for (const solution of props.solutions)
    counts.set(solution.target, solution.candidates.length)
  return counts
})
</script>

<template>
  <div class="flex flex-wrap gap-2">
    <Button
      v-for="target in targets"
      :key="target"
      :variant="ButtonVariant.Secondary"
      :size="ButtonSize.Compact"
      :aria-pressed="shown.includes(target)"
      :class="shown.includes(target) ? '' : 'text-faint-foreground'"
      @click="emit('toggle', target)"
    >
      <span
        aria-hidden="true"
        class="size-3 shrink-0 rounded-cell"
        :class="[
          pipFill[target][RankStep.First],
          shown.includes(target) ? '' : 'opacity-disabled',
        ]"
      />
      <span>{{ t(`floor.target.${target}`) }}</span>
      <span class="tabular-nums">{{ countOf.get(target) ?? 0 }}</span>
    </Button>
  </div>
</template>
