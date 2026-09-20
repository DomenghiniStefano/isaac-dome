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
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { HelpTip } from '@/components/ui/tooltip'
import { useMessages } from '@/i18n'
import { floorEntries } from '@/lib/diagnostics/floor'
import { TARGET_ORDER, cellPosition } from '@/lib/floor/cellView'
import { useFloorStore } from '@/stores/floor'
import { TargetView } from '@/lib/ipc/types'
import type { FloorSolutionView } from '@/lib/ipc/types'
import FloorClear from './floor/FloorClear.vue'
import FloorGrid from './floor/FloorGrid.vue'
import FloorMove from './floor/FloorMove.vue'
import FloorLegend from './floor/FloorLegend.vue'
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

// The rules of all three stay open below, whichever one the grid is drawing: the answer is on
// the grid and this is the reasoning behind it, which is worth reading side by side.

// The missing start room is a hint and it was drawn as an alert: a full-width box, above
// everything, saying a sentence that qualifies one of three answers. It is a mark beside the
// switch now, with the sentence a hover away — the same shape every other explanation on this
// app takes. `floorEntries` maps the kind to `null` for exactly this, so the list below cannot
// also draw it and say it twice.
const noStartRoom = computed(() =>
  (store.view?.diagnostics ?? []).some((d) => d.kind === 'noStartRoom'),
)

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

    <!-- The drawing on the left with its own controls, the rooms you can draw with on the
         right. The grid is a fixed 27.5rem wide and will never be anything else, so a page
         that stacks the two leaves that much of itself empty down the whole length of it.

         **The rooms crossed the page**, and that is the second arrangement this screen has
         had. They were a row of fourteen swatches above the grid, which is the one place with
         no width to write a name in; the right-hand column meanwhile held a switch and three
         lines of legend, and was the emptiest part of the screen. Down a column each room has
         its key, its picture and its name on one line.

         **What used to be on the right and no longer is: the rules.** Three open-ended lists
         of quotations beside a fixed-width drawing made the right-hand column the longer of
         the two and put the reasoning where the answer belongs. They are under both columns
         now, across the full width, where a quotation has a line to live on. -->
    <div class="flex flex-col items-start gap-4 lg:flex-row">
      <Card class="w-full lg:w-fit lg:shrink-0">
        <!-- The card is sized to its content and the grid is the widest thing in it, which is
             what decides this column's width. **The cap belongs on the prose, not here**: a
             width on this box is its border box, so the padding comes out of it and the grid
             spills over the card's own edge by exactly `p-3` — measured, 12px. -->
        <CardContent class="flex flex-col gap-3">
          <div class="flex items-center gap-2">
            <FloorTargets
              :solutions="solutions"
              :shown="store.shown"
              @show="store.show($event)"
            />
            <!-- Two marks and they are never both here: this one appears only while the start
                 room is missing, the legend's lives under the grid beside the arrows. Two
                 identical question marks in one row would be one question mark too many. -->
            <HelpTip v-if="noStartRoom" :label="t('floor.startRoomMissing')">{{
              t('floor.diagnostic.noStartRoom')
            }}</HelpTip>
          </div>
          <!-- The grid is 27.5rem and cannot be anything else: thirteen cells of pixel art do
               not have a smaller size that is still pixel art. The window has no minimum
               width, so below about 620px it was wider than its own card and simply drew
               over the edge of it — measured, 101px out at 620. It scrolls now, which keeps
               the whole floor reachable instead of hiding the right of it behind a border.
               At any width that fits, this box is exactly the grid and no scrollbar exists. -->
          <div class="min-w-0 overflow-x-auto">
            <FloorGrid
              :cells="store.cells"
              :icons="store.icons"
              :solutions="solutions"
              :shown="store.shown"
              @paint="store.paint($event)"
              @settle="store.settle()"
              @erase="store.erase($event)"
            />
          </div>
          <!-- Under the grid, the two things you can do to the whole of it and the one mark
               that explains what it is showing. The arrows are on the left because they move
               the drawing and the drawing starts there; the button that empties it is as far
               from them as the row allows, which is the point. -->
          <div class="flex items-center gap-2">
            <FloorMove :cells="store.cells" @move="store.move($event)" />
            <FloorLegend :shown="store.shown" />
            <!-- The wrapper carries the margin, not the component: `FloorClear`'s root is a
                 Dialog, which renders no element of its own for a class to land on. -->
            <div class="ml-auto">
              <FloorClear @clear="store.clear()" />
            </div>
          </div>
        </CardContent>
      </Card>

      <Card class="w-full min-w-0 flex-1">
        <CardHeader>
          <CardTitle>{{ t('floor.rooms') }}</CardTitle>
        </CardHeader>
        <CardContent>
          <FloorPalette
            :brush="store.brush"
            :icons="store.icons"
            @pick="store.brush = $event"
          />
        </CardContent>
      </Card>
    </div>

    <!-- Closed by default, and each one opens on its own. What the rules say is
         the tool's reasoning rather than its answer — the answer is on the grid — so it
         is there for whoever wants to check it and out of the way of whoever does not.

         No source line, on purpose: the wiki's attribution is carried once, in
         Information, where a licence belongs, and not repeated on every row of every
         screen. -->
    <div class="flex flex-col gap-4">
      <CardCollapsible
        v-for="target in TARGET_ORDER"
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
</template>
