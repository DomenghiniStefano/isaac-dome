<script setup lang="ts">
import { ExternalLinkIcon } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { GridTable } from '@/components/ui/grid-table'
import { useMessages } from '@/i18n'
import type { RunView } from '@/lib/ipc/types'
import { runKey } from '@/lib/runs/runKey'
import { runsColumns } from '@/lib/table/columns'
import RunCharacterCell from './RunCharacterCell.vue'
import RunDate from './RunDate.vue'
import RunOutcomeCell from './RunOutcomeCell.vue'
import RunSourceCell from './RunSourceCell.vue'

// A run is `(source, ordinal)`: that pair is its identity in the archive and so the key here,
// because two sources number their runs from one each. A click anywhere on the row opens it, and
// so does the button in Actions — the one the keyboard reaches.
defineProps<{ runs: RunView[] }>()
const emit = defineEmits<{ open: [run: RunView, event: MouseEvent] }>()
const { t } = useMessages()
</script>

<template>
  <!-- The screen scrolls as a page (`PageScroll`): the rows virtualize against it, and the
       columns' header pins to its top while the list goes by under it. -->
  <GridTable
    :columns="runsColumns"
    :rows="runs"
    :row-key="runKey"
    virtual
    clickable
    @row-click="(run, event) => emit('open', run, event)"
  >
    <template #cell-date="{ row }"><RunDate :run="row" /></template>
    <template #cell-character="{ row }">
      <RunCharacterCell :run="row" />
    </template>
    <template #cell-outcome="{ row }"><RunOutcomeCell :run="row" /></template>
    <template #cell-floors="{ row }">
      <span class="text-row tabular-nums">{{ row.floors }}</span>
    </template>
    <template #cell-seed="{ row }">
      <span class="truncate text-label text-subtle-foreground">{{
        row.seedWords
      }}</span>
    </template>
    <template #cell-source="{ row }"><RunSourceCell :run="row" /></template>
    <template #actions="{ row }">
      <Button
        :variant="ButtonVariant.Ghost"
        :size="ButtonSize.IconCompact"
        :aria-label="t('runs.open')"
        @click="emit('open', row, $event)"
      >
        <ExternalLinkIcon />
      </Button>
    </template>
  </GridTable>
</template>
