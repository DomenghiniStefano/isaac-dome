<script setup lang="ts">
import MarkCell from '@/components/marks/MarkCell.vue'
import type { MarkArt } from '@/components/marks/markVisual'
import { CellLevel, type Cell } from '@/lib/ipc/types'
import KitSection from '../../KitSection.vue'

// The cells as the IPC sends them: the mask it was read from, the level it reached, and
// whether the run was won online. Named rather than written as digits, because the page
// exists to show what each state *looks* like and a number says nothing about that.
const known = (bits: number, level: CellLevel, online = false): Cell => ({
  kind: 'known',
  bits,
  level,
  online,
})
const NEVER = known(0, CellLevel.Empty)
const NORMAL = known(1, CellLevel.Normal)
const HARD_ALONE = known(2, CellLevel.Hard)
const HARD = known(3, CellLevel.Hard)
const ONLINE_ONLY = known(4, CellLevel.Empty, true)
const NORMAL_ONLINE = known(5, CellLevel.Normal, true)
const HARD_ONLINE = known(7, CellLevel.Hard, true)
const unknown: Cell = { kind: 'unknown' }

const states: { label: string; cell: Cell }[] = [
  { label: 'mai fatto', cell: NEVER },
  { label: 'normale', cell: NORMAL },
  { label: 'hard', cell: HARD },
  { label: 'hard, vinto online', cell: HARD_ONLINE },
  { label: 'solo vinto online', cell: ONLINE_ONLY },
  { label: 'non leggibile', cell: unknown },
  { label: 'anomalo', cell: { kind: 'unexpected', value: 9 } },
]

// The game's symbols reach the app only through the icon protocol, which a browser does not
// have, so the matrix below draws on a stand-in: a pixel heart of our own, in the two inks the
// game uses, dark for normal and red for hard. It is here to show the paper, not the symbol.
const heart = (ink: string) =>
  `data:image/svg+xml,${encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" shape-rendering="crispEdges" fill="${ink}"><path d="M3 4h4v1h2V4h4v1h1v4h-1v1h-1v1h-1v1h-1v1H9v1H7v-1H6v-1H5v-1H4v-1H3V9H2V5h1z"/></svg>`,
  )}`
const STAND_IN: MarkArt = { normal: heart('#2a1208'), hard: heart('#901800') }

// The reference profile's shape: a dense top, an empty bottom, an unreadable corner.
const rows: { name: string; cells: Cell[] }[] = [
  { name: 'Isaac', cells: [HARD, HARD_ONLINE, HARD, HARD_ALONE, NORMAL] },
  { name: 'Cain', cells: [HARD, HARD, NEVER, NORMAL, NEVER] },
  {
    name: 'The Forgotten',
    cells: [HARD_ALONE, NEVER, NEVER, NEVER, unknown],
  },
  {
    name: 'Tainted Lost',
    cells: [NEVER, NORMAL_ONLINE, NEVER, NEVER, unknown],
  },
]
</script>

<template>
  <KitSection title="MarkCell" class="col-span-2">
    <div class="flex flex-wrap gap-4">
      <div class="flex flex-wrap gap-2">
        <div
          v-for="state in states"
          :key="state.label"
          class="flex w-14 flex-col items-center gap-1.5"
        >
          <MarkCell :cell="state.cell" :art="null" :label="state.label" />
          <span class="text-center text-micro text-subtle-foreground">{{
            state.label
          }}</span>
        </div>
      </div>
      <div class="flex flex-col gap-0.5">
        <div
          v-for="row in rows"
          :key="row.name"
          class="flex items-center gap-0.5"
        >
          <span class="w-24 truncate text-label text-foreground-soft">{{
            row.name
          }}</span>
          <MarkCell
            v-for="(cell, i) in row.cells"
            :key="i"
            :cell="cell"
            :art="STAND_IN"
            :label="`${row.name} ${i}`"
          />
        </div>
      </div>
    </div>
  </KitSection>
</template>
