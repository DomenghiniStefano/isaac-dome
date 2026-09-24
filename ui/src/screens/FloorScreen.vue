<script setup lang="ts">
import { Grid3x3Icon } from '@lucide/vue'
import { computed } from 'vue'
import DiagnosticsList from '@/components/diagnostics/DiagnosticsList.vue'
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import { Card, CardContent } from '@/components/ui/card'
import { HelpTip } from '@/components/ui/tooltip'
import { useMessages } from '@/i18n'
import { floorEntries } from '@/lib/diagnostics/floor'
import { useFloorStore } from '@/stores/floor'
import FloorClear from './floor/FloorClear.vue'
import FloorGrid from './floor/FloorGrid.vue'
import FloorMove from './floor/FloorMove.vue'
import FloorLegend from './floor/FloorLegend.vue'
import FloorPalette from './floor/FloorPalette.vue'
import FloorReasoning from './floor/FloorReasoning.vue'
import FloorTargets from './floor/FloorTargets.vue'
import ScreenHeader from './ScreenHeader.vue'

const store = useFloorStore()
const { t } = useMessages()

// The grid answers from the moment it opens: an empty floor is a diagnostic, not a blank.
void store.solve()
// The pictures are asked once, beside the first answer: they do not change with the drawing.
void store.loadIcons()

const solutions = computed(() => store.view?.solutions ?? [])

// The reasoning follows the switch: it explains the picture on the grid, and only that one.
const shownSolution = computed(
  () => solutions.value.find((s) => s.target === store.shown) ?? null,
)

// The missing start room is a hint and it was drawn as an alert: a full-width box, above
// everything, saying a sentence that qualifies one of three answers. It is a mark beside the
// switch now, with the sentence a hover away — the same shape every other explanation on this
// app takes. `floorEntries` maps the kind to `null` for exactly this, so the list below cannot
// also draw it and say it twice.
const noStartRoom = computed(() =>
  (store.view?.diagnostics ?? []).some((d) => d.kind === 'noStartRoom'),
)
</script>

<template>
  <div class="flex h-full flex-col gap-4 overflow-y-auto px-5.5 pt-5 pb-15">
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

    <!-- **One workbench, the way a drawing program is laid out**: the tools against the canvas,
         the canvas, and what the canvas means beside it. It was two cards before — the grid in
         one, the rooms in another across the screen — and at most widths the second wrapped
         under the first, so choosing a room meant scrolling past the drawing to reach the
         brush. The rules were three closed cards under all of it, one per target.

         The workbench wraps rather than breaks: the drawing and its rail are one piece that is
         never split, and the reasoning takes whatever width is left beside it or, where there is
         none, the line underneath. -->
    <Card>
      <CardContent class="flex flex-wrap items-stretch gap-4">
        <div class="flex max-w-full min-w-0 flex-col gap-3">
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
          <!-- Below the compact width the rail and the grid do not fit side by side, and the
               rail goes under the drawing in two columns rather than leaving it: a tool a scroll
               away from its canvas is the arrangement this one replaced. -->
          <div class="flex items-start gap-3 @max-compact/page:flex-col">
            <FloorPalette
              class="@max-compact/page:order-last @max-compact/page:w-full @max-compact/page:grid-cols-2"
              :brush="store.brush"
              :icons="store.icons"
              @pick="store.brush = $event"
            />
            <div class="flex max-w-full min-w-0 flex-col gap-3">
              <!-- The grid is 27.5rem and cannot be anything else: thirteen cells of pixel art
                   do not have a smaller size that is still pixel art. Where the window is
                   narrower than that it scrolls, which keeps the whole floor reachable instead
                   of hiding the right of it behind the card's edge. -->
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
              <!-- The arrows move the drawing and sit where it starts; the button that empties
                   it is as far from them as the row allows, which is the point. -->
              <div class="flex items-center gap-2">
                <FloorMove :cells="store.cells" @move="store.move($event)" />
                <FloorLegend :shown="store.shown" />
                <!-- The wrapper carries the margin, not the component: `FloorClear`'s root is
                     a Dialog, which renders no element of its own for a class to land on. -->
                <div class="ml-auto">
                  <FloorClear @clear="store.clear()" />
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- As tall as the workbench beside it and never taller: the pane is laid over a box
             that only has a minimum height, so a long list scrolls inside it instead of
             stretching the row and pushing the grid's controls down. On a line of its own it
             keeps the grid's height, for the same reason.

             No source line, on purpose: the wiki's attribution is carried once, in
             Information, where a licence belongs, and not repeated on every row. -->
        <div
          class="relative min-h-floor-grid min-w-floor-reasoning flex-1 basis-floor-reasoning"
        >
          <FloorReasoning
            class="absolute inset-0"
            :target="store.shown"
            :solution="shownSolution"
          />
        </div>
      </CardContent>
    </Card>
  </div>
</template>
