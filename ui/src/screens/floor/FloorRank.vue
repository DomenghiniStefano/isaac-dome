<script setup lang="ts">
import type { RankStep } from '@/lib/floor/cellView'
import { levelHeight, targetAura, targetFill } from '@/lib/floor/targets'
import type { TargetView } from '@/lib/ipc/types'

// What a candidate cell wears, drawn once and used in three places: on the grid, in the legend
// and beside each row of the reasoning. It was written out twice before, the grid's and the
// legend's, and the reasoning would have made it three — three copies of one picture is how a
// legend ends up explaining a square that no longer looks like that.
//
// It covers whatever box it lands in, so the caller decides the square; the veil, the level and
// the number are the same everywhere.

defineProps<{ target: TargetView; step: RankStep; rank: number }>()
</script>

<template>
  <span aria-hidden="true" class="absolute inset-0" :class="targetAura[target]">
    <span
      class="absolute inset-x-0 bottom-0"
      :class="targetFill[target]"
      :style="{ height: levelHeight[step] }"
    />
    <span
      class="absolute inset-0 grid place-items-center text-floor-rank-foreground tabular-nums text-shadow-floor-rank"
      >{{ rank }}</span
    >
  </span>
</template>
