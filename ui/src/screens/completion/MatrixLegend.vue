<script setup lang="ts">
import MarkCell from '@/components/marks/MarkCell.vue'
import type { MarkArt } from '@/components/marks/markVisual'
import { useMessages } from '@/i18n'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import type { Cell } from '@/lib/ipc/types'

defineProps<{ art: MarkArt | null }>()
const { t } = useMessages()

// The states a player has to learn. A suspicious value is left out on purpose: it explains
// itself in its own tooltip, it isn't a state to learn (the export's choice).
const entries: { cell: Cell; label: MessageKey<MessageSchema> }[] = [
  { cell: { kind: 'known', bits: 0 }, label: 'completion.legend.empty' },
  { cell: { kind: 'known', bits: 1 }, label: 'completion.legend.normal' },
  { cell: { kind: 'known', bits: 3 }, label: 'completion.legend.hard' },
  { cell: { kind: 'known', bits: 7 }, label: 'completion.legend.online' },
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
