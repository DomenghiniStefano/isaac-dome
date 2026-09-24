<script setup lang="ts">
import { sortBy } from 'lodash-es'
import { computed } from 'vue'
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import { useMessages } from '@/i18n'
import { cellPosition, rankStep } from '@/lib/floor/cellView'
import type { FloorSolutionView, TargetView } from '@/lib/ipc/types'
import FloorRank from './FloorRank.vue'

// The reasoning behind the answer on the grid, for the one target the grid is drawing.
//
// **It sits beside the grid and follows the switch**, and that is the third arrangement. It was
// three collapsible cards under the whole screen, one per target, closed: the answer and the
// reason for it were a scroll and a click apart, and two of the three explained a picture that
// was not on the screen. One pane, the shown target's, next to the drawing it explains.
//
// Each row opens on the same square the cell wears, so a row and its cell are recognisably one
// thing — the order is the square's order, best place first.

const props = defineProps<{
  target: TargetView
  solution: FloorSolutionView | null
}>()
const { t } = useMessages()

const candidates = computed(() =>
  sortBy(props.solution?.candidates ?? [], 'rank'),
)
const unresolved = computed(() => props.solution?.unresolved ?? [])

// A candidate names a cell, and a cell index is not a place. "Cell 97" sends you counting
// along the grid; row 8, column 7 is where you were already looking.
const placeOf = (cell: number): string => {
  const { row, column } = cellPosition(cell)
  return t('floor.at', { row, column })
}
</script>

<template>
  <section class="flex min-h-0 flex-col border border-hairline bg-data">
    <header
      class="flex h-control shrink-0 items-center justify-between gap-2 border-b border-hairline px-3"
    >
      <span class="text-control">{{ t(`floor.target.${target}`) }}</span>
      <span class="text-caption text-subtle-foreground tabular-nums">{{
        candidates.length
      }}</span>
    </header>
    <div class="flex min-h-0 flex-1 flex-col overflow-y-auto">
      <div
        v-for="candidate in candidates"
        :key="candidate.cell"
        class="flex gap-3 border-b border-hairline px-3 py-2"
      >
        <span
          class="relative size-floor-cell shrink-0 bg-floor-empty text-caption"
        >
          <FloorRank
            :target="target"
            :step="rankStep(candidate.rank)"
            :rank="candidate.rank + 1"
          />
        </span>
        <div class="flex min-w-0 flex-col gap-1">
          <div class="flex flex-wrap items-baseline gap-x-3">
            <span class="text-label tabular-nums">{{
              placeOf(candidate.cell)
            }}</span>
            <span class="text-caption text-subtle-foreground"
              >{{ candidate.neighbours }} {{ t('floor.neighbours') }}</span
            >
          </div>
          <span
            v-for="rule in candidate.applied"
            :key="rule.id"
            class="text-caption text-foreground-soft"
            >{{ rule.quote }}</span
          >
        </div>
      </div>
      <div v-if="candidates.length === 0" class="p-3">
        <EmptyCategory>{{ t('floor.none') }}</EmptyCategory>
      </div>

      <!-- What the grid cannot judge is shown under what it can, never instead of it: a rule
           we cannot evaluate is not a rule that allows everything. -->
      <div v-if="unresolved.length > 0" class="flex flex-col gap-2 px-3 py-3">
        <span class="text-label text-subtle-foreground">{{
          t('floor.unresolved')
        }}</span>
        <div v-for="item in unresolved" :key="item.rule" class="flex flex-col">
          <span class="text-caption text-foreground-soft">{{
            item.quote
          }}</span>
          <span class="text-caption text-faint-foreground">{{
            item.note
          }}</span>
        </div>
      </div>
    </div>
  </section>
</template>
