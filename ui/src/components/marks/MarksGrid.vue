<script setup lang="ts">
import type { Message } from '@/i18n/message'
import { computed, useTemplateRef } from 'vue'
import PixelSprite from '@/components/sprite/PixelSprite.vue'
import { Progress, ProgressSize, ProgressTone } from '@/components/ui/progress'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { useMessages } from '@/i18n'
import { assertNever } from '@/lib/assertNever'
import { cn } from '@/lib/cn'
import {
  CellStatus,
  MatrixGroup,
  cellReading,
  cellStatusKey,
  columnTallies,
  matrixGroups,
  tallyColumns,
} from '@/lib/completion/completionView'
import type { Tally } from '@/lib/completion/completionView'
import { SecondLevelView } from '@/lib/ipc/types'
import type { Cell, MarksMatrix } from '@/lib/ipc/types'
import MarkCell from './MarkCell.vue'
import TallyCell from './TallyCell.vue'
import { markArtOf } from './markVisual'

const props = defineProps<{ matrix: MarksMatrix }>()
const { t } = useMessages()

const art = computed(() => props.matrix.art.map((view) => markArtOf(view)))
// The column count is data: it reaches the grid template as a CSS variable.
const columns = computed(() => ({
  '--matrix-columns': props.matrix.bosses.length,
}))
// B22 item 4: two number columns, not one slot holding a pair. Each carries its own
// denominator because each sits under its own heading. Worked out once per matrix, beside the
// tally they are read from, not twice per row per render.
const groups = computed(() =>
  matrixGroups(props.matrix).map((group) => ({
    ...group,
    columns: tallyColumns(group.tally),
    rows: group.rows.map((entry) => ({
      ...entry,
      columns: tallyColumns(entry.tally),
    })),
  })),
)
const totals = computed(() => columnTallies(props.matrix).map(tallyColumns))

const groupTitle: Record<MatrixGroup, Message> = {
  [MatrixGroup.Base]: 'completion.groups.base',
  [MatrixGroup.Tainted]: 'completion.groups.tainted',
}

// What a suspicious cell holds, said in its tooltip rather than hidden among the empty ones.
const valueOf = (cell: Cell): number | null => {
  switch (cell.kind) {
    case 'known':
      return cell.bits
    case 'unexpected':
      return cell.value
    case 'unknown':
      return null
    default:
      return assertNever(cell)
  }
}

// The second level in its column's word (B66): Ultra Greedier in Greed, hard elsewhere. A
// column the payload does not name falls back to hard, which is what the eleven others are.
const cellState = (cell: Cell, column: number): string => {
  const reading = cellReading(cell)
  const second = props.matrix.secondLevels[column] ?? SecondLevelView.Hard
  const state = t(cellStatusKey(reading.status, second))
  const said =
    reading.status === CellStatus.Unexpected
      ? `${state} ${valueOf(cell) ?? ''}`
      : state
  return reading.online ? `${said} · ${t('marks.wonOnline')}` : said
}

// A row's bar is the same reading its two numbers are, drawn: hard over readable, in the
// done colour once the row is finished. It carries no figures of its own — the two columns
// at the right already print them, and a third copy on the same row is noise.
const barTone = (tally: Tally): ProgressTone =>
  tally.complete ? ProgressTone.Done : ProgressTone.Primary

// The boss header is a strip of its own, outside the box that scrolls sideways, so it follows
// that box's horizontal position instead of scrolling with it.
const head = useTemplateRef('head')
const followRows = (event: Event) => {
  if (head.value && event.target instanceof HTMLElement)
    head.value.scrollLeft = event.target.scrollLeft
}
</script>

<template>
  <!-- Schermate.dc.html, "Matrice dei marchi": the name column, one cell per boss, the row's
       levels over readable, then each boss's in the footer.
       **The page scrolls vertically, the rows scroll sideways, and the header is a strip of
       its own between the two.** The header has to pin to the top of the
       *screen*, and a `sticky top-0` holds on to the nearest scroll container. In CSS an
       axis that is not `visible` makes the other one `auto` too, so the rows' sideways box is
       also a vertical scroll container with no height, and a header inside it would have
       nothing to pin to. So the header lives outside it, pinned to the page, clipped
       sideways (`overflow-hidden`, which the page's `sticky` still sees through) and moved
       to the rows' horizontal position by `followRows`.
       Nothing here carries a horizontal padding: a sticky cell would slide under it. The
       first and last columns hold their own inset instead, and the rows paint edge to
       edge, the way a banded table wants. -->
  <div :style="columns" class="flex flex-col pb-3">
    <div
      ref="head"
      class="sticky top-0 z-raised-header overflow-hidden bg-card"
    >
      <div
        class="grid w-max min-w-full grid-cols-matrix items-end justify-items-center gap-0.5 pt-2.5 pb-1.5"
      >
        <!-- The corner: above the pinned column and above the pinned row, so neither slides
           over it. It takes the **whole** header's height and not just its label's, or the
           boss symbols scroll under the part of the name column it leaves uncovered — which
           is what they did, and it reads as the column having come unstuck. -->
        <span
          class="sticky left-0 z-raised-corner flex items-end self-stretch justify-self-stretch bg-card pl-3 text-label text-subtle-foreground"
          >{{ t('completion.grid.character') }}</span
        >
        <!-- The boss is its symbol, and its name is the tooltip: written vertically above the
             symbols, the names cost the pinned header 118px of a screen whose subject is the
             rows under it. -->
        <Tooltip v-for="(boss, b) in matrix.bosses" :key="boss">
          <TooltipTrigger as-child>
            <div tabindex="0" :aria-label="boss" class="flex">
              <PixelSprite
                :url="art[b]?.hard ?? null"
                class="size-mark-symbol shrink-0"
              />
            </div>
          </TooltipTrigger>
          <TooltipContent>{{ boss }}</TooltipContent>
        </Tooltip>
        <span
          class="justify-self-end pr-0.5 text-label text-subtle-foreground"
          >{{ t('completion.grid.normal') }}</span
        >
        <span class="justify-self-end pr-3 text-label text-subtle-foreground">{{
          t('completion.grid.hard')
        }}</span>
      </div>
    </div>

    <div class="overflow-x-auto" @scroll="followRows">
      <div class="flex w-max min-w-full flex-col">
        <section
          v-for="group in groups"
          :key="group.group"
          class="mt-1.5 flex flex-col"
        >
          <!-- The group header is the matrix's own grid, not a flex row: the container is wider
           than the tracks (min-w-full, so the rows paint the whole card), so anything pushed
           to its right edge lands past the two columns instead of under them. Everything
           that is not a total shares one cell, from the first column to the last boss. -->
          <div
            class="grid grid-cols-matrix items-center gap-0.5 border-t border-hairline pt-1 pb-1.5"
          >
            <div
              class="col-start-1 -col-end-3 flex flex-wrap items-center gap-2.5 pl-3"
            >
              <span class="text-label text-highlight">{{
                t(groupTitle[group.group])
              }}</span>
              <span class="text-label text-subtle-foreground"
                >{{ group.first }} – {{ group.last }}</span
              >
              <!-- What the group cannot read sits before the totals, where it is not read as a
               third one. -->
              <span
                v-if="group.unknown > 0"
                class="ml-auto text-label text-state-unknown-foreground tabular-nums"
                >{{
                  t('completion.grid.unreadable', { n: group.unknown })
                }}</span
              >
            </div>
            <TallyCell
              :column="group.columns.normal"
              class="justify-self-end pr-0.5 text-label"
            />
            <TallyCell
              :column="group.columns.hard"
              class="justify-self-end pr-3 text-label"
            />
          </div>

          <div
            v-for="entry in group.rows"
            :key="entry.row.character"
            :class="
              cn(
                'group grid h-row-matrix grid-cols-matrix items-center justify-items-center gap-0.5 hover:bg-row-hover',
                entry.index % 2 === 1 && 'bg-row-alt',
              )
            "
          >
            <!-- The pinned column. It repeats the row's own background because a transparent
             sticky cell shows the cells sliding under it, and it takes the hover from the
             row through `group-hover` for the same reason. -->
            <div
              :class="
                cn(
                  'sticky left-0 z-raised flex min-w-0 items-center gap-2 justify-self-stretch py-0.5 pr-2 pl-3 group-hover:bg-row-hover',
                  entry.index % 2 === 1 ? 'bg-row-alt' : 'bg-card',
                )
              "
            >
              <span
                class="grid size-matrix-head shrink-0 place-items-center border border-border tile-wash"
              >
                <PixelSprite
                  :url="entry.row.headUrl"
                  placeholder
                  class="size-8"
                />
              </span>
              <div class="flex min-w-0 flex-1 flex-col gap-1.5">
                <span class="truncate text-caption text-foreground">{{
                  entry.row.character
                }}</span>
                <Progress
                  :model-value="entry.tally.hard"
                  :max="entry.tally.readable"
                  :size="ProgressSize.Micro"
                  :tone="barTone(entry.tally)"
                  :aria-label="entry.row.character"
                />
              </div>
            </div>
            <Tooltip v-for="(cell, b) in entry.row.cells" :key="b">
              <TooltipTrigger as-child>
                <MarkCell
                  :cell="cell"
                  :art="art[b] ?? null"
                  :label="`${entry.row.character} · ${matrix.bosses[b] ?? ''}`"
                />
              </TooltipTrigger>
              <TooltipContent class="flex flex-col gap-0.5">
                <span>{{ entry.row.character }} · {{ matrix.bosses[b] }}</span>
                <span class="text-foreground-soft">{{
                  cellState(cell, b)
                }}</span>
              </TooltipContent>
            </Tooltip>
            <TallyCell
              :column="entry.columns.normal"
              class="justify-self-end pr-0.5 text-label"
            />
            <TallyCell
              :column="entry.columns.hard"
              class="justify-self-end pr-3 text-label"
            />
          </div>
        </section>

        <!-- Two rows rather than two numbers stacked in a 40px column: the name column is
         already there to say which of the two each row is, and it states the relation the
         pair exists for — every hard mark is also a normal one. -->
        <div
          class="mt-2 grid grid-cols-matrix items-center justify-items-center gap-0.5 border-t border-border pt-1.5"
        >
          <span
            class="sticky left-0 z-raised justify-self-stretch bg-card pr-1.5 pl-3 text-label text-highlight"
            >{{ t('completion.grid.columnTotals') }}</span
          >
          <TallyCell
            v-for="(total, b) in totals"
            :key="b"
            :column="total.normal"
            class="text-micro"
          />
          <span />
          <span />
        </div>
        <div
          class="grid grid-cols-matrix items-center justify-items-center gap-0.5 pt-1"
        >
          <span
            class="sticky left-0 z-raised justify-self-stretch bg-card pr-1.5 pl-3 text-label text-subtle-foreground"
            >{{ t('completion.grid.columnTotalsHard') }}</span
          >
          <TallyCell
            v-for="(total, b) in totals"
            :key="b"
            :column="total.hard"
            class="text-micro"
          />
          <span />
          <span />
        </div>
      </div>
    </div>
  </div>
</template>
