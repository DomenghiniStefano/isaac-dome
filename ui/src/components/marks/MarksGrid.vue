<script setup lang="ts">
import { computed } from 'vue'
import PixelSprite from '@/components/sprite/PixelSprite.vue'
import { Progress, ProgressSize, ProgressTone } from '@/components/ui/progress'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { useMessages } from '@/i18n'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import { assertNever } from '@/lib/assertNever'
import { cn } from '@/lib/cn'
import {
  CellStatus,
  MatrixGroup,
  TallyTone,
  cellReading,
  columnTallies,
  matrixGroups,
  tallyColumns,
} from '@/lib/completion/completionView'
import type { Tally, TallyColumn } from '@/lib/completion/completionView'
import type { Cell, MarksMatrix } from '@/lib/ipc/types'
import MarkCell from './MarkCell.vue'
import { markArtOf } from './markVisual'

const props = defineProps<{ matrix: MarksMatrix }>()
const { t } = useMessages()

const art = computed(() => props.matrix.art.map((view) => markArtOf(view)))
// The column count is data: it reaches the grid template as a CSS variable.
const columns = computed(() => ({
  '--matrix-columns': props.matrix.bosses.length,
}))
const groups = computed(() => matrixGroups(props.matrix))
const totals = computed(() => columnTallies(props.matrix))

const groupTitle: Record<MatrixGroup, MessageKey<MessageSchema>> = {
  [MatrixGroup.Base]: 'completion.groups.base',
  [MatrixGroup.Tainted]: 'completion.groups.tainted',
}

const statusText: Record<CellStatus, MessageKey<MessageSchema>> = {
  [CellStatus.Empty]: 'completion.cell.empty',
  [CellStatus.Normal]: 'completion.cell.normal',
  [CellStatus.Hard]: 'completion.cell.hard',
  [CellStatus.Unknown]: 'completion.cell.unknown',
  [CellStatus.Unexpected]: 'completion.cell.unexpected',
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

const cellState = (cell: Cell): string => {
  const reading = cellReading(cell)
  const state = t(statusText[reading.status])
  const said =
    reading.status === CellStatus.Unexpected
      ? `${state} ${valueOf(cell) ?? ''}`
      : state
  return reading.online ? `${said} · ${t('marks.wonOnline')}` : said
}

// A number that fills its denominator in the done colour, one with no denominator at all
// fainter than one that has progress to show. Gold is not used for "almost there": gold
// means unlockable now, and nothing else. A record over the whole set, so a tone with no
// colour fails to compile.
const toneClass: Record<TallyTone, string> = {
  [TallyTone.Full]: 'text-state-done-foreground',
  [TallyTone.Partial]: 'text-subtle-foreground',
  [TallyTone.Unreadable]: 'text-faint-foreground',
}

// B22 item 4: two number columns, not one slot holding a pair. Each carries its own
// denominator because each sits under its own heading.
const columnsOf = (tally: Tally) => tallyColumns(tally)
const label = (column: TallyColumn): string =>
  `${column.value}/${column.readable}`

// A row's bar is the same reading its two numbers are, drawn: hard over readable, in the
// done colour once the row is finished. It carries no figures of its own — the two columns
// at the right already print them, and a third copy on the same row is noise.
const barTone = (tally: Tally): ProgressTone =>
  tally.complete ? ProgressTone.Done : ProgressTone.Primary
</script>

<template>
  <!-- Schermate.dc.html, "Matrice dei marchi": the name column, one cell per boss, the row's
       levels over readable, then each boss's in the footer.
       **The scrolling box is the card's, not this element's** (card #58). It used to be an
       `overflow-x-auto` here, and in CSS an axis that is not `visible` makes the other one
       `auto` too — so that box already was a vertical scroll container, but with no height
       to constrain it, it never scrolled and a `sticky top-0` inside had nothing to hold on
       to. With the card taking the height that is left and scrolling on both axes, the boss
       header can stay at the top and the name column at the left, which on twelve columns
       is the difference between reading a row and counting cells with a finger.
       Nothing here carries a horizontal padding: a sticky cell would slide under it. The
       first and last columns hold their own inset instead, and the rows paint edge to
       edge, the way a banded table wants. -->
  <div :style="columns" class="flex w-max min-w-full flex-col pb-3">
    <div
      class="sticky top-0 z-20 grid grid-cols-matrix items-end justify-items-center gap-0.5 bg-card pt-2.5 pb-1.5"
    >
      <!-- The corner: above the pinned column and above the pinned row, so neither slides
           over it. It takes the **whole** header's height and not just its label's, or the
           boss names scroll under the part of the name column it leaves uncovered — which
           is what they did, and it reads as the column having come unstuck. -->
      <span
        class="sticky left-0 z-30 flex h-matrix-header items-end justify-self-stretch bg-card pl-3 text-label text-subtle-foreground"
        >{{ t('completion.grid.character') }}</span
      >
      <div
        v-for="(boss, b) in matrix.bosses"
        :key="boss"
        class="flex h-matrix-header flex-col items-center justify-end gap-2"
      >
        <span
          class="writing-vertical text-label whitespace-nowrap text-foreground"
          >{{ boss }}</span
        >
        <PixelSprite
          :url="art[b]?.hard ?? null"
          class="size-mark-symbol shrink-0"
        />
      </div>
      <span class="justify-self-end pr-0.5 text-label text-subtle-foreground">{{
        t('completion.grid.normal')
      }}</span>
      <span class="justify-self-end pr-3 text-label text-subtle-foreground">{{
        t('completion.grid.hard')
      }}</span>
    </div>

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
            >{{ group.unknown }} {{ t('completion.grid.unreadable') }}</span
          >
        </div>
        <span
          :class="
            cn(
              'justify-self-end pr-0.5 text-label tabular-nums',
              toneClass[columnsOf(group.tally).normal.tone],
            )
          "
          >{{ label(columnsOf(group.tally).normal) }}</span
        >
        <span
          :class="
            cn(
              'justify-self-end pr-3 text-label tabular-nums',
              toneClass[columnsOf(group.tally).hard.tone],
            )
          "
          >{{ label(columnsOf(group.tally).hard) }}</span
        >
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
              'sticky left-0 z-10 flex min-w-0 items-center gap-2 justify-self-stretch py-0.5 pr-2 pl-3 group-hover:bg-row-hover',
              entry.index % 2 === 1 ? 'bg-row-alt' : 'bg-card',
            )
          "
        >
          <span
            class="grid size-matrix-head shrink-0 place-items-center border border-border tile-wash"
          >
            <PixelSprite :url="entry.row.headUrl" placeholder class="size-8" />
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
            <span class="text-foreground-soft">{{ cellState(cell) }}</span>
          </TooltipContent>
        </Tooltip>
        <span
          :class="
            cn(
              'justify-self-end pr-0.5 text-label tabular-nums',
              toneClass[columnsOf(entry.tally).normal.tone],
            )
          "
          >{{ label(columnsOf(entry.tally).normal) }}</span
        >
        <span
          :class="
            cn(
              'justify-self-end pr-3 text-label tabular-nums',
              toneClass[columnsOf(entry.tally).hard.tone],
            )
          "
          >{{ label(columnsOf(entry.tally).hard) }}</span
        >
      </div>
    </section>

    <!-- Two rows rather than two numbers stacked in a 40px column: the name column is
         already there to say which of the two each row is, and it states the relation the
         pair exists for — every hard mark is also a normal one. -->
    <div
      class="mt-2 grid grid-cols-matrix items-center justify-items-center gap-0.5 border-t border-border pt-1.5"
    >
      <span
        class="sticky left-0 z-10 justify-self-stretch bg-card pr-1.5 pl-3 text-label text-highlight"
        >{{ t('completion.grid.columnTotals') }}</span
      >
      <span
        v-for="(tally, b) in totals"
        :key="b"
        :class="
          cn('text-micro tabular-nums', toneClass[columnsOf(tally).normal.tone])
        "
        >{{ label(columnsOf(tally).normal) }}</span
      >
      <span />
      <span />
    </div>
    <div
      class="grid grid-cols-matrix items-center justify-items-center gap-0.5 pt-1"
    >
      <span
        class="sticky left-0 z-10 justify-self-stretch bg-card pr-1.5 pl-3 text-label text-subtle-foreground"
        >{{ t('completion.grid.columnTotalsHard') }}</span
      >
      <span
        v-for="(tally, b) in totals"
        :key="b"
        :class="
          cn('text-micro tabular-nums', toneClass[columnsOf(tally).hard.tone])
        "
        >{{ label(columnsOf(tally).hard) }}</span
      >
      <span />
      <span />
    </div>
  </div>
</template>
