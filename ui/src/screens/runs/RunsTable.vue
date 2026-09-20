<script setup lang="ts">
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { VirtualRows } from '@/components/ui/virtual'
import type { ScrollOffset } from '@/lib/scale/scrollOffset'
import { useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import type { RunView } from '@/lib/ipc/types'
import { runKey } from '@/lib/runs/runKey'
import { rowWidePx } from '@/lib/scale/rows'
import RunRow from './RunRow.vue'

defineProps<{
  runs: RunView[]
  selected: RunView | null
  offset: ScrollOffset | null
}>()
const emit = defineEmits<{
  select: [run: RunView]
  offsetChange: [offset: ScrollOffset]
}>()
const { t } = useMessages()
// A run is `(source, ordinal)`: that pair is its identity in the archive and therefore the key
// here, because two sources number their runs from one each. It lives in `lib/runs/runKey.ts`
// since the screen remembers which run you had open and needs the same answer twice.
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col">
    <div
      class="grid grid-cols-runs items-center border-b border-hairline bg-muted text-label text-subtle-foreground"
    >
      <span class="px-2 py-1.5">{{ t('runs.column.character') }}</span>
      <span class="px-2 py-1.5">{{ t('runs.column.outcome') }}</span>
      <span class="px-2 py-1.5 text-right">{{ t('runs.column.floors') }}</span>
      <span class="px-2 py-1.5">{{ t('runs.column.seed') }}</span>
      <span class="px-2 py-1.5">{{ t('runs.column.source') }}</span>
    </div>
    <VirtualRows
      v-slot="{ visible }"
      :rows="runs"
      :row-px="rowWidePx"
      :offset="offset"
      @offset-change="emit('offsetChange', $event)"
    >
      <Button
        v-for="{ index, style, row: run } in visible"
        :key="runKey(run)"
        :variant="ButtonVariant.Ghost"
        :size="ButtonSize.Row"
        :style="style"
        :class="
          cn(
            'absolute inset-x-0 top-0 grid h-row-wide translate-y-(--row-start) grid-cols-runs items-center border-b border-hairline text-left hover:bg-row-hover',
            index % 2 === 1 && 'bg-row-alt',
            selected !== null &&
              runKey(selected) === runKey(run) &&
              'bg-row-hover',
          )
        "
        @click="emit('select', run)"
      >
        <RunRow :run="run" />
      </Button>
    </VirtualRows>
  </div>
</template>
