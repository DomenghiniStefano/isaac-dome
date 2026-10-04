<script setup lang="ts">
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { VirtualRows } from '@/components/ui/virtual'
import { useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import type { RunView } from '@/lib/ipc/types'
import { runKey } from '@/lib/runs/runKey'
import { rowWidePx } from '@/lib/scale/rows'
import RunRow from './RunRow.vue'

defineProps<{ runs: RunView[] }>()
const emit = defineEmits<{ open: [run: RunView, event: MouseEvent] }>()
const { t } = useMessages()
// A run is `(source, ordinal)`: that pair is its identity in the archive and therefore the key
// here, because two sources number their runs from one each. It lives in `lib/runs/runKey.ts`
// since a run's page is addressed by the same key.
</script>

<template>
  <!-- The screen scrolls as a page (`PageScroll`): the rows virtualize against it, and the
       columns' header pins to its top while the list goes by under it. -->
  <div class="flex flex-col">
    <div
      class="sticky top-0 z-raised-header grid grid-cols-runs items-center border-b border-hairline bg-muted text-label text-subtle-foreground @max-compact/page:grid-cols-runs-narrow"
    >
      <span class="px-2 py-1.5">{{ t('runs.column.date') }}</span>
      <span class="px-2 py-1.5">{{ t('runs.column.character') }}</span>
      <span class="px-2 py-1.5">{{ t('runs.column.outcome') }}</span>
      <span class="px-2 py-1.5 text-right">{{ t('runs.column.floors') }}</span>
      <span class="px-2 py-1.5 @max-compact/page:hidden">{{
        t('runs.column.seed')
      }}</span>
      <span class="px-2 py-1.5 @max-compact/page:hidden">{{
        t('runs.column.source')
      }}</span>
    </div>
    <VirtualRows v-slot="{ visible }" :rows="runs" :row-px="rowWidePx">
      <Button
        v-for="{ index, style, row: run } in visible"
        :key="runKey(run)"
        :variant="ButtonVariant.Ghost"
        :size="ButtonSize.Row"
        :style="style"
        :class="
          cn(
            'absolute inset-x-0 top-0 grid h-row-wide translate-y-(--row-start) grid-cols-runs items-center border-b border-hairline text-left hover:bg-row-hover @max-compact/page:grid-cols-runs-narrow',
            index % 2 === 1 && 'bg-row-alt',
          )
        "
        @click="emit('open', run, $event)"
      >
        <RunRow :run="run" />
      </Button>
    </VirtualRows>
  </div>
</template>
