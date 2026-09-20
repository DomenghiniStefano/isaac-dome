<script setup lang="ts">
import { computed } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { useMessages } from '@/i18n'
import { RankStep } from '@/lib/floor/cellView'
import { bandFill } from '@/lib/floor/bands'
import { TargetView } from '@/lib/ipc/types'
import type { FloorSolutionView } from '@/lib/ipc/types'

// The three targets, switched on and off — and since the answer moved onto the grid, these are
// the grid's own controls: switching one off does not hide a list, it takes a colour out of
// every cell and widens what is left.
//
// Each carries its count, because a target switched off and a target with nothing to say look
// identical on the grid — a filter that hides the difference between "you are not looking" and
// "there is nothing there" answers the wrong question.
//
// **The legend lives here too**, under the switches, because the two halves of a band's colour
// are read together: the switch says which hue is which target, the legend says which step is
// which rank. Nothing said the second half before (B64, defect 3) — neither that 1 is the most
// likely nor that the scale stops at the third, which is precisely the thing the rules refuse
// to claim.

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

const steps = [RankStep.First, RankStep.Second, RankStep.Third]

const countOf = computed(() => {
  const counts = new Map<TargetView, number>()
  for (const solution of props.solutions)
    counts.set(solution.target, solution.candidates.length)
  return counts
})
</script>

<template>
  <div class="flex flex-col gap-3">
    <div class="flex flex-col gap-1.5">
      <Button
        v-for="target in targets"
        :key="target"
        :variant="ButtonVariant.Secondary"
        :size="ButtonSize.Compact"
        class="w-full justify-start"
        :aria-pressed="shown.includes(target)"
        :class="shown.includes(target) ? '' : 'text-faint-foreground'"
        @click="emit('toggle', target)"
      >
        <span
          aria-hidden="true"
          class="size-3 shrink-0 rounded-cell"
          :class="[
            bandFill[target][RankStep.First],
            shown.includes(target) ? '' : 'opacity-disabled',
          ]"
        />
        <span>{{ t(`floor.target.${target}`) }}</span>
        <span class="ml-auto tabular-nums">{{ countOf.get(target) ?? 0 }}</span>
      </Button>
    </div>

    <div class="flex flex-col gap-1.5 border-t border-hairline pt-3">
      <span class="text-label text-subtle-foreground">{{
        t('floor.rank.title')
      }}</span>
      <!-- Three swatches per row and not one: the colour of a band is a hue *and* a step, and
           a legend that showed one hue would explain half of it. -->
      <div v-for="step in steps" :key="step" class="flex items-center gap-2">
        <span aria-hidden="true" class="flex shrink-0 gap-floor-gap">
          <span
            v-for="target in targets"
            :key="target"
            class="size-3 rounded-cell"
            :class="bandFill[target][step]"
          />
        </span>
        <span class="text-caption text-subtle-foreground">{{
          t(`floor.rank.${step}`)
        }}</span>
      </div>
    </div>
  </div>
</template>
