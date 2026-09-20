<script setup lang="ts">
import { Grid3x3Icon } from '@lucide/vue'
import { computed } from 'vue'
import DiagnosticsList from '@/components/diagnostics/DiagnosticsList.vue'
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useMessages } from '@/i18n'
import { floorEntries } from '@/lib/diagnostics/floor'
import { cellPosition } from '@/lib/floor/cellView'
import { targetPage } from '@/lib/floor/targets'
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
          <FloorPalette :brush="store.brush" @pick="store.brush = $event" />
          <FloorGrid
            :cells="store.cells"
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

        <Card v-for="target in order" :key="target">
          <CardHeader class="flex-col items-start gap-0.5">
            <CardTitle>{{ t(`floor.target.${target}`) }}</CardTitle>
            <!-- The page that explains the room itself. The rules below were all read from
                 the Secret Room's page, so without this the other two rooms are judged here
                 and explained nowhere. -->
            <span class="text-caption text-faint-foreground"
              >{{ t('floor.wiki') }}: {{ targetPage[target] }}</span
            >
          </CardHeader>
          <CardContent class="flex flex-col gap-3">
            <template v-if="(solutionFor(target)?.candidates.length ?? 0) > 0">
              <!-- Every row carries the sentence that lit it and the page it came from. That
                   is the CC BY-SA attribution reaching the person reading the screen, not a
                   layout detail: the rules are quotations, and a quotation without its source
                   is not one. -->
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
                <div
                  v-for="rule in candidate.applied"
                  :key="rule.id"
                  class="flex flex-col"
                >
                  <span class="text-caption text-foreground-soft">{{
                    rule.quote
                  }}</span>
                  <span class="text-caption text-faint-foreground"
                    >{{ t('floor.source') }}: {{ rule.url }}</span
                  >
                </div>
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
                <span class="text-caption text-faint-foreground"
                  >{{ t('floor.source') }}: {{ item.url }}</span
                >
              </div>
            </template>
          </CardContent>
        </Card>
      </div>
    </div>
  </div>
</template>
