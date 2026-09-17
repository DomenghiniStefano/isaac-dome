<script setup lang="ts">
import { computed } from 'vue'
import PixelSprite from '@/components/sprite/PixelSprite.vue'
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
  cellReading,
  columnTallies,
  matrixGroups,
} from '@/lib/completion/completionView'
import type { Tally } from '@/lib/completion/completionView'
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
  return reading.third ? `${said} · ${t('marks.thirdLevel')}` : said
}

// Complete in the done colour; nothing readable fainter than a count that has a denominator.
// Gold is not used for "almost there": gold means unlockable now, and nothing else.
const tallyClass = (tally: Tally): string => {
  if (tally.complete) return 'text-state-done-foreground'
  if (tally.readable === 0) return 'text-faint-foreground'
  return 'text-subtle-foreground'
}

// Two numbers over one denominator (B22): how many bosses have a level at all, and how many
// have the second. `hard` is a subset of the first, so stating the denominator twice would
// say nothing the pair does not.
//
// **This is not B22's layout, which is item 4 of that entry**: the design file gives the
// grid a second number column on the right of every row, in the group header and per boss
// in the footer. Until it does, the pair lives in the one slot the grid already has, which
// keeps it truthful without inventing the columns.
const tallyLabel = (tally: Tally): string =>
  `${tally.normal}/${tally.readable} · ${tally.hard}`
</script>

<template>
  <!-- Schermate.dc.html, "Matrice dei marchi": the name column, one cell per boss, the row's
       levels over readable, then each boss's in the footer. -->
  <div class="overflow-x-auto">
    <div :style="columns" class="flex w-max min-w-full flex-col">
      <div
        class="grid grid-cols-matrix items-end justify-items-center gap-0.5 pb-1.5"
      >
        <span class="justify-self-start text-label text-subtle-foreground">{{
          t('completion.grid.character')
        }}</span>
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
        <span
          class="justify-self-end pr-0.5 text-label text-subtle-foreground"
          >{{ t('completion.grid.levels') }}</span
        >
      </div>

      <section
        v-for="group in groups"
        :key="group.group"
        class="mt-1.5 flex flex-col"
      >
        <div
          class="flex flex-wrap items-center gap-2.5 border-t border-hairline pt-1 pb-1.5"
        >
          <span class="text-label text-highlight">{{
            t(groupTitle[group.group])
          }}</span>
          <span class="text-label text-subtle-foreground"
            >{{ group.first }} – {{ group.last }}</span
          >
          <span class="ml-auto text-label text-subtle-foreground tabular-nums"
            >{{ group.normal }}/{{ group.readable }} · {{ group.hard }}
            {{ t('completion.grid.levels') }}</span
          >
          <span
            v-if="group.unknown > 0"
            class="text-label text-state-unknown-foreground tabular-nums"
            >{{ group.unknown }} {{ t('completion.grid.unreadable') }}</span
          >
        </div>

        <div
          v-for="entry in group.rows"
          :key="entry.row.character"
          :class="
            cn(
              'grid grid-cols-matrix items-center justify-items-center gap-0.5 hover:bg-row-hover',
              entry.index % 2 === 1 && 'bg-row-alt',
            )
          "
        >
          <div
            class="flex min-w-0 items-center gap-2 justify-self-stretch py-0.5 pr-2"
          >
            <PixelSprite
              :url="entry.row.headUrl"
              placeholder
              class="size-8 shrink-0"
            />
            <span class="truncate text-caption text-foreground">{{
              entry.row.character
            }}</span>
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
                tallyClass(entry.tally),
              )
            "
            >{{ tallyLabel(entry.tally) }}</span
          >
        </div>
      </section>

      <div
        class="mt-2 grid grid-cols-matrix items-center justify-items-center gap-0.5 border-t border-border pt-1.5"
      >
        <span class="justify-self-start pr-1.5 text-label text-highlight">{{
          t('completion.grid.columnTotals')
        }}</span>
        <span
          v-for="(tally, b) in totals"
          :key="b"
          :class="cn('text-micro tabular-nums', tallyClass(tally))"
          >{{ tallyLabel(tally) }}</span
        >
        <span />
      </div>
    </div>
  </div>
</template>
