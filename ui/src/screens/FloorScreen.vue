<script setup lang="ts">
import { Grid3x3Icon } from '@lucide/vue'
import { computed } from 'vue'
import DiagnosticsList from '@/components/diagnostics/DiagnosticsList.vue'
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import {
  Card,
  CardCollapsible,
  CardCollapsibleContent,
  CardCollapsibleTrigger,
  CardContent,
} from '@/components/ui/card'
import { useMessages } from '@/i18n'
import { floorEntries } from '@/lib/diagnostics/floor'
import { cellPosition } from '@/lib/floor/cellView'
import { useFloorStore } from '@/stores/floor'
import { TargetView } from '@/lib/ipc/types'
import type { FloorSolutionView } from '@/lib/ipc/types'
import FloorClear from './floor/FloorClear.vue'
import FloorGrid from './floor/FloorGrid.vue'
import FloorPalette from './floor/FloorPalette.vue'
import FloorTargets from './floor/FloorTargets.vue'
import ScreenHeader from './ScreenHeader.vue'

const store = useFloorStore()
const { t } = useMessages()

// The grid answers from the moment it opens: an empty floor is a diagnostic, not a blank.
void store.solve()
// The pictures are asked once, beside the first answer: they do not change with the drawing.
void store.loadIcons()

const solutions = computed(() => store.view?.solutions ?? [])

const solutionFor = (target: TargetView): FloorSolutionView | null =>
  solutions.value.find((s) => s.target === target) ?? null

// The order the three targets are read in: the one a player looks for on every floor, then
// the one that needs the whole map painted, then the one this grid can only partly judge.
const order = [
  TargetView.Secret,
  TargetView.SuperSecret,
  TargetView.UltraSecret,
]

// A candidate names a cell, and a cell index is not a place. "Cell 97" sends you counting
// along the grid; row 8, column 7 is where you were already looking.
const placeOf = (cell: number): string => {
  const { row, column } = cellPosition(cell)
  return t('floor.at', { row, column })
}
</script>

<template>
  <div class="flex max-w-250 flex-col gap-4">
    <ScreenHeader :icon="Grid3x3Icon" :title="t('routes.floor')">{{
      t('floor.intro')
    }}</ScreenHeader>

    <DiagnosticsList
      v-if="store.view"
      :entries="floorEntries(store.view.diagnostics)"
    />

    <!-- The failure is said, and the grid stays: what you painted is yours, and losing it
         because a command did not answer would be the app throwing away your work. -->
    <EmptyCategory v-if="store.failed">{{ t('floor.failed') }}</EmptyCategory>

    <!-- The drawing on the left, the answers about it on the right. The grid is a fixed
         27.5rem wide and will never be anything else, so a page that stacks the two leaves
         that much of itself empty down the whole length of the floor. -->
    <div class="flex flex-col items-start gap-4 lg:flex-row">
      <Card class="w-full lg:w-fit lg:shrink-0">
        <CardContent class="flex flex-col gap-3">
          <FloorPalette
            :brush="store.brush"
            :icons="store.icons"
            @pick="store.brush = $event"
          />
          <FloorGrid
            :cells="store.cells"
            :icons="store.icons"
            :solutions="solutions"
            :shown="store.shown"
            @stroke="store.stroke($event)"
            @erase="store.erase($event)"
          />
          <!-- Under the grid, because that is what it empties. Beside the filters it read as
               one more thing you could do to the answer. -->
          <div class="flex justify-end">
            <FloorClear @clear="store.clear()" />
          </div>
        </CardContent>
      </Card>

      <div class="flex w-full min-w-0 flex-1 flex-col gap-4">
        <FloorTargets
          :solutions="solutions"
          :shown="store.shown"
          @toggle="store.toggle($event)"
        />

        <!-- Closed by default, and each one opens on its own. What the rules say is
             the tool's reasoning rather than its answer — the answer is on the grid — so it
             is there for whoever wants to check it and out of the way of whoever does not.

             No source line, on purpose: the wiki's attribution is carried once, in
             Information, where a licence belongs, and not repeated on every row of every
             screen. -->
        <CardCollapsible
          v-for="target in order"
          :key="target"
          :default-open="false"
        >
          <CardCollapsibleTrigger>
            {{ t(`floor.target.${target}`) }}
            <template #summary>
              <span class="text-caption text-subtle-foreground tabular-nums">{{
                solutionFor(target)?.candidates.length ?? 0
              }}</span>
            </template>
          </CardCollapsibleTrigger>
          <CardCollapsibleContent class="flex flex-col gap-3">
            <template v-if="(solutionFor(target)?.candidates.length ?? 0) > 0">
              <div
                v-for="candidate in solutionFor(target)?.candidates ?? []"
                :key="candidate.cell"
                class="flex flex-col gap-1 border-b border-hairline pb-2"
              >
                <div class="flex items-center gap-3">
                  <span class="text-label tabular-nums">{{
                    placeOf(candidate.cell)
                  }}</span>
                  <span class="text-caption text-subtle-foreground"
                    >{{ candidate.neighbours }}
                    {{ t('floor.neighbours') }}</span
                  >
                </div>
                <span
                  v-for="rule in candidate.applied"
                  :key="rule.id"
                  class="text-caption text-foreground-soft"
                  >{{ rule.quote }}</span
                >
              </div>
            </template>
            <EmptyCategory v-else>{{ t('floor.none') }}</EmptyCategory>

            <!-- What the grid cannot judge is shown under what it can, never instead of it: a
                 rule we cannot evaluate is not a rule that allows everything. -->
            <template v-if="(solutionFor(target)?.unresolved.length ?? 0) > 0">
              <span class="text-label text-subtle-foreground">{{
                t('floor.unresolved')
              }}</span>
              <div
                v-for="item in solutionFor(target)?.unresolved ?? []"
                :key="item.rule"
                class="flex flex-col"
              >
                <span class="text-caption text-foreground-soft">{{
                  item.quote
                }}</span>
                <span class="text-caption text-faint-foreground">{{
                  item.note
                }}</span>
              </div>
            </template>
          </CardCollapsibleContent>
        </CardCollapsible>
      </div>
    </div>
  </div>
</template>
