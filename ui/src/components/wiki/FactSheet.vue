<script setup lang="ts">
import { computed } from 'vue'
import { useMessages } from '@/i18n'
import type { WikiPageRef } from '@/lib/ipc/types'
import { factColumns, shownValue } from '@/lib/wiki/factChips'
import type { WikiCategory } from '@/router/routeTable'

// A card's facts as a spec sheet — a label, then its value — so they read as data and not as
// states or tags. The rows are the table view's own columns (`factColumns`), the same facts in
// the same order; one without a value is left out.
const props = defineProps<{ page: WikiPageRef; category: WikiCategory }>()
const { t } = useMessages()

const rows = computed(() =>
  factColumns(props.category).flatMap((column) => {
    const shown = shownValue(column, props.page)
    if (shown === null) return []
    const value = shown.kind === 'message' ? t(shown.key) : shown.text
    return [{ key: column.key, label: column.label, value }]
  }),
)
</script>

<template>
  <dl
    v-if="rows.length > 0"
    class="grid w-full min-w-0 grid-cols-fact-sheet gap-x-3 gap-y-1 text-left"
  >
    <template v-for="row in rows" :key="row.key">
      <dt class="text-micro text-faint-foreground">{{ t(row.label) }}</dt>
      <dd
        class="truncate text-caption text-foreground tabular-nums"
        :title="row.value"
      >
        {{ row.value }}
      </dd>
    </template>
  </dl>
</template>
