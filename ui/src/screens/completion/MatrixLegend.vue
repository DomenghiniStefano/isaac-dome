<script setup lang="ts">
import type { Message } from '@/i18n/message'
import MarkCell from '@/components/marks/MarkCell.vue'
import type { MarkArt } from '@/components/marks/markVisual'
import { useMessages } from '@/i18n'
import { CellLevel, type Cell } from '@/lib/ipc/types'

defineProps<{ art: MarkArt | null }>()
const { t } = useMessages()

// The states a player has to learn. A suspicious value is left out on purpose: it explains
// itself in its own tooltip, it isn't a state to learn (the export's choice).
//
// Each entry states its reading rather than a mask to be decoded: these are examples of
// what the IPC sends, and `bits` is the value a real cell in that state would carry.
const entries: { cell: Cell; label: Message }[] = [
  {
    cell: { kind: 'known', bits: 0, level: CellLevel.Empty, online: false },
    label: 'completion.legend.empty',
  },
  {
    cell: { kind: 'known', bits: 1, level: CellLevel.Normal, online: false },
    label: 'completion.legend.normal',
  },
  {
    cell: { kind: 'known', bits: 3, level: CellLevel.Hard, online: false },
    label: 'completion.legend.hard',
  },
  {
    cell: { kind: 'known', bits: 7, level: CellLevel.Hard, online: true },
    label: 'completion.legend.online',
  },
  { cell: { kind: 'unknown' }, label: 'completion.legend.unknown' },
]
</script>

<template>
  <div class="flex flex-wrap items-center gap-3">
    <div
      v-for="entry in entries"
      :key="entry.label"
      class="flex items-center gap-1.5"
    >
      <MarkCell :cell="entry.cell" :art="art" />
      <span class="text-label text-foreground-soft">{{ t(entry.label) }}</span>
    </div>
  </div>
</template>
