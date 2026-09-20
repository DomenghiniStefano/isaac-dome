<script setup lang="ts">
import MarkCell from '@/components/marks/MarkCell.vue'
import type { Cell } from '@/lib/ipc/types'
import KitSection from '../../KitSection.vue'

const known = (bits: number): Cell => ({ kind: 'known', bits })
const unknown: Cell = { kind: 'unknown' }

const states: { label: string; cell: Cell }[] = [
  { label: 'mai fatto', cell: known(0) },
  { label: 'normale', cell: known(1) },
  { label: 'hard', cell: known(3) },
  { label: 'hard + bit 2', cell: known(7) },
  { label: 'solo bit 2', cell: known(4) },
  { label: 'non leggibile', cell: unknown },
  { label: 'anomalo', cell: { kind: 'unexpected', value: 9 } },
]

// The reference profile's shape: a dense top, an empty bottom, an unreadable corner.
const rows: { name: string; cells: Cell[] }[] = [
  { name: 'Isaac', cells: [known(3), known(7), known(3), known(2), known(1)] },
  { name: 'Cain', cells: [known(3), known(3), known(0), known(1), known(0)] },
  {
    name: 'The Forgotten',
    cells: [known(2), known(0), known(0), known(0), unknown],
  },
  {
    name: 'Tainted Lost',
    cells: [known(0), known(5), known(0), known(0), unknown],
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
            :art="null"
            :label="`${row.name} ${i}`"
          />
        </div>
      </div>
    </div>
  </KitSection>
</template>
