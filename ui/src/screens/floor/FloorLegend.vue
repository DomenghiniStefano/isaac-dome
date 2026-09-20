<script setup lang="ts">
import { HelpTip } from '@/components/ui/tooltip'
import { useMessages } from '@/i18n'
import { RankStep } from '@/lib/floor/cellView'
import { levelHeight, targetAura, targetFill } from '@/lib/floor/targets'
import type { TargetView } from '@/lib/ipc/types'

// What the fill means, a hover away.
//
// There was a `FloorLegend.vue` before and it was not a legend: it was the brush selector,
// which says what you are *about* to paint and never what the colours mean (B64, defect 3).
// Nothing explained that the fullest square is the likeliest place, nor that the scale stops
// at the third — which is the one thing the rules refuse to claim and therefore the one thing
// that has to be said out loud.
//
// It was a line under the grid and it is a mark now, for the reason every other explanation on
// this screen became one: a sentence read once, kept on screen forever, is a sentence taking
// width from the drawing beside it. **The squares came into the tooltip with it**, because
// three fills and a sentence about fills belong in one place — the sentence alone would ask
// you to remember which square it was talking about.
//
// They are the cell itself, at the cell's own size and in the hue of the target being shown:
// a legend drawn smaller, or in some other colour, explains a screen nobody is looking at.

defineProps<{ shown: TargetView }>()
const { t } = useMessages()

// Rising left to right, because that is the direction "more" is read in.
const steps = [RankStep.Third, RankStep.Second, RankStep.First]
</script>

<template>
  <HelpTip :label="t('floor.rank.title')">
    <span class="flex flex-col gap-2">
      <span class="flex gap-floor-gap">
        <span
          v-for="(step, place) in steps"
          :key="step"
          class="relative size-floor-cell bg-floor-empty text-caption"
        >
          <span class="absolute inset-0" :class="targetAura[shown]">
            <span
              class="absolute inset-x-0 bottom-0"
              :class="targetFill[shown]"
              :style="{ height: levelHeight[step] }"
            />
            <span
              class="absolute inset-0 grid place-items-center text-floor-rank-foreground tabular-nums text-shadow-floor-rank"
              >{{ steps.length - place }}</span
            >
          </span>
        </span>
      </span>
      <span>{{ t('floor.rank.note') }}</span>
    </span>
  </HelpTip>
</template>
