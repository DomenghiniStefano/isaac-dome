<script setup lang="ts" generic="Facet extends string, Sort extends string">
import { XIcon } from '@lucide/vue'
import { computed } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import {
  ToggleGroup,
  ToggleGroupItem,
  ToggleGroupType,
} from '@/components/ui/toggle-group'
import { useMessages } from '@/i18n'
import type { Label, ToolbarLabels } from './labels'

// How many rows are shown, the search, the sort, and a chip per picked value.
//
// Generic over the screen's facet and sort so a call site stays typed end to end: with `string`
// props each screen would narrow the emitted sort back to its own union, which is the small
// duplication this component exists to remove.
const props = defineProps<{
  shown: number
  total: number
  query: string
  sort: Sort
  sorts: Sort[]
  sortText: Record<Sort, Label>
  order: Facet[]
  picks: Record<Facet, string[]>
  // A picked value in words: the Character facet stores ids (`docs/BACKLOG.md` B28), so no
  // component can label one on its own.
  valueLabel: (facet: Facet, value: string) => string
  labels: ToolbarLabels
}>()
const emit = defineEmits<{
  'update:query': [query: string]
  'update:sort': [sort: Sort]
  toggle: [facet: Facet, value: string]
}>()
const { t } = useMessages()

// A single-choice group empties when its chosen item is clicked again; a sort always has one.
const onSort = (value: unknown) => {
  const next = props.sorts.find((s) => s === value)
  if (next) emit('update:sort', next)
}

const chips = computed(() =>
  props.order.flatMap((facet) =>
    props.picks[facet].map((value) => ({
      facet,
      value,
      label: props.valueLabel(facet, value),
    })),
  ),
)
</script>

<template>
  <CardHeader class="flex-wrap">
    <CardTitle class="tabular-nums"
      >{{ shown }} / {{ total }} {{ t(labels.rows) }}</CardTitle
    >
    <div class="flex flex-wrap items-center gap-2">
      <Input
        :model-value="query"
        :placeholder="t(labels.search)"
        class="w-search"
        @update:model-value="emit('update:query', String($event))"
      />
      <span class="text-label">{{ t(labels.sortBy) }}</span>
      <ToggleGroup
        :type="ToggleGroupType.Single"
        :model-value="sort"
        @update:model-value="onSort"
      >
        <ToggleGroupItem v-for="s in sorts" :key="s" :value="s">{{
          t(sortText[s])
        }}</ToggleGroupItem>
      </ToggleGroup>
    </div>
  </CardHeader>
  <div
    v-if="chips.length > 0"
    class="flex flex-wrap items-center gap-1.5 border-b border-hairline bg-muted px-3 py-2"
  >
    <span class="text-label text-subtle-foreground">{{
      t(labels.activeFilters)
    }}</span>
    <Button
      v-for="chip in chips"
      :key="`${chip.facet}-${chip.value}`"
      :variant="ButtonVariant.Outline"
      :size="ButtonSize.Compact"
      @click="emit('toggle', chip.facet, chip.value)"
      >{{ chip.label }}<XIcon
    /></Button>
  </div>
</template>
