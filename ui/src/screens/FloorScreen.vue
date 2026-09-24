<script setup lang="ts">
import { vScrollMemory } from '@/directives/scrollMemory'
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
import FloorPalette from './floor/FloorPalette.vue'
import FloorReasoning from './floor/FloorReasoning.vue'
import FloorTargets from './floor/FloorTargets.vue'
import { useFloorDrawing } from './floor/useFloorDrawing'
import ScreenHeader from './ScreenHeader.vue'

const store = useFloorStore()
const { t } = useMessages()

// The drawing is this tab's and the pictures are this window's: a torn-off tab takes its floor
// with it, and every tab draws with the same game.
const {
  cells,
  shown,
  brush,
  view,
  failed,
  paint,
  settle,
  move,
  erase,
  clear,
  show,
  pick,
} = useFloorDrawing()
// The pictures are asked once, beside the first answer: they do not change with the drawing.
void store.loadIcons()

const solutions = computed(() => view.value?.solutions ?? [])

// The reasoning follows the switch: it explains the picture on the grid, and only that one.
const shownSolution = computed(
  () => solutions.value.find((s) => s.target === shown.value) ?? null,
)

// The missing start room is a hint and it was drawn as an alert: a full-width box, above
// everything, saying a sentence that qualifies one of three answers. It is a mark beside the
// switch now, with the sentence a hover away — the same shape every other explanation on this
// app takes. `floorEntries` maps the kind to `null` for exactly this, so the list below cannot
// also draw it and say it twice.
const noStartRoom = computed(() =>
  (view.value?.diagnostics ?? []).some((d) => d.kind === 'noStartRoom'),
)
</script>

<template>
  <div
    v-scroll-memory="'page'"
    class="flex h-full flex-col gap-4 overflow-y-auto px-5.5 pt-5 pb-15"
  >
    <ScreenHeader :icon="Grid3x3Icon" :title="t('routes.floor')">{{
      t('floor.intro')
    }}</ScreenHeader>

    <DiagnosticsList v-if="view" :entries="floorEntries(view.diagnostics)" />

    <!-- The failure is said, and the grid stays: what you painted is yours, and losing it
         because a command did not answer would be the app throwing away your work. -->
    <EmptyCategory v-if="failed">{{ t('floor.failed') }}</EmptyCategory>

    <!-- **One workbench, the way a drawing program is laid out**: the tools against the canvas,
         the canvas, and what the canvas means beside it. It was two cards before — the grid in
         one, the rooms in another across the screen — and at most widths the second wrapped
         under the first, so choosing a room meant scrolling past the drawing to reach the
         brush. The rules were three closed cards under all of it, one per target.

         **The order never changes: the grid, then the rooms, then the reasoning.** What gives
         way to a narrower window is the line breaks, in the reverse of that order — the
         reasoning leaves the first line before the rooms do, and the rooms leave it only to sit
         under the grid. Both breaks are the workbench's own width (`@container/floor`, the sums
         are in `floor.css`), not the page's: the question is whether these three fit in this
         card. -->
    <Card>
      <CardContent class="@container/floor">
        <div
          class="flex flex-col gap-4 @floor-all/floor:flex-row @floor-all/floor:items-stretch"
        >
          <div class="flex w-fit max-w-full min-w-0 flex-col gap-3">
            <div class="flex items-center gap-2">
              <FloorTargets
                :solutions="solutions"
                :shown="shown"
                @show="show($event)"
              />
              <!-- Two marks and they are never both here: this one appears only while the start
                 room is missing, the legend's lives under the grid beside the arrows. Two
                 identical question marks in one row would be one question mark too many. -->
              <HelpTip
                v-if="noStartRoom"
                :label="t('floor.startRoomMissing')"
                >{{ t('floor.diagnostic.noStartRoom') }}</HelpTip
              >
            </div>
            <!-- Where the rail and the grid do not fit side by side, the rail goes under the
               drawing in two columns rather than away from it: a tool a scroll away from its
               canvas is the arrangement this one replaced. -->
            <div
              class="flex flex-col gap-3 @floor-rail/floor:flex-row @floor-rail/floor:items-start"
            >
              <FloorPalette
                class="order-last grid-cols-2 @floor-rail/floor:order-first @floor-rail/floor:w-floor-palette @floor-rail/floor:shrink-0 @floor-rail/floor:grid-cols-1"
                :brush="brush"
                :icons="store.icons"
                @pick="pick($event)"
              />
              <div class="flex max-w-full min-w-0 flex-col gap-3">
                <!-- The grid is 27.5rem and cannot be anything else: thirteen cells of pixel art
                   do not have a smaller size that is still pixel art. Where the window is
                   narrower than that it scrolls, which keeps the whole floor reachable instead
                   of hiding the right of it behind the card's edge. -->
                <div class="min-w-0 overflow-x-auto">
                  <FloorGrid
                    :cells="cells"
                    :icons="store.icons"
                    :solutions="solutions"
                    :shown="shown"
                    @paint="paint($event)"
                    @settle="settle()"
                    @erase="erase($event)"
                  />
                </div>
                <!-- The arrows move the drawing and sit where it starts; the button that empties
                   it is as far from them as the row allows, which is the point. What the colours
                   mean is not here: it sits beside the list of ranked places it explains. -->
                <div class="flex items-center gap-2">
                  <FloorMove :cells="cells" @move="move($event)" />
                  <!-- The wrapper carries the margin, not the component: `FloorClear`'s root is
                     a Dialog, which renders no element of its own for a class to land on. -->
                  <div class="ml-auto">
                    <FloorClear @clear="clear()" />
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- As tall as the workbench beside it and never taller: the pane is laid over a box
             that only has a minimum height, so a long list scrolls inside it instead of
             stretching the row and pushing the grid's controls down. Under the workbench it
             keeps the grid's height, for the same reason.

             No source line, on purpose: the wiki's attribution is carried once, in
             Information, where a licence belongs, and not repeated on every row. -->
          <div
            class="relative min-h-floor-grid @floor-all/floor:min-w-0 @floor-all/floor:flex-1"
          >
            <FloorReasoning
              class="absolute inset-0"
              :target="shown"
              :solution="shownSolution"
            />
          </div>
        </div>
      </CardContent>
    </Card>
  </div>
</template>
