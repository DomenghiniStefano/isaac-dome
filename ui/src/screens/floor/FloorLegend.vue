<script setup lang="ts">
import { useMessages } from '@/i18n'
import { RankStep } from '@/lib/floor/cellView'
import { levelHeight, targetAura, targetFill } from '@/lib/floor/targets'
import type { TargetView } from '@/lib/ipc/types'

// What the fill means, in one line under the grid.
//
// There was a `FloorLegend.vue` before and it was not a legend: it was the brush selector,
// which says what you are *about* to paint and never what the colours mean (B64, defect 3).
// Nothing explained that the fullest square is the likeliest place, nor that the scale stops
// at the third — which is the one thing the rules refuse to claim and therefore the one thing
// that has to be said out loud.
//
// Three squares and a sentence, rising left to right, because that is the direction "more"
// is read in. They are the cell itself, at the cell's own size and in the hue of the target
// actually being shown, number included — a legend drawn smaller, or in some other colour, is
// a legend for a screen nobody is looking at.

defineProps<{ shown: TargetView }>()
const { t } = useMessages()

const steps = [RankStep.Third, RankStep.Second, RankStep.First]
</script>

<template>
  <!-- Capped at the grid's width, which is the one thing on this card that has to be told to
       stop. The card is sized to its content, so a sentence on one line makes the column as
       wide as the sentence and takes that width from the one beside it; the grid cannot say
       "no wider than me" on its own. -->
  <div class="flex max-w-floor-grid items-center gap-2">
    <span aria-hidden="true" class="flex shrink-0 gap-floor-gap">
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
    <span class="text-caption text-subtle-foreground">{{
      t('floor.rank.note')
    }}</span>
  </div>
</template>
