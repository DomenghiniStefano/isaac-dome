<script setup lang="ts">
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
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import {
  CollectionFacet,
  CollectionSort,
  collectionFacetOrder,
} from '@/lib/collection/collectionFacets'
import type { CollectionFilter } from '@/lib/collection/collectionFacets'
import { collectionFacetValueLabel } from './collectionLabels'

const props = defineProps<{
  shown: number
  total: number
  filter: CollectionFilter
  query: string
  sort: CollectionSort
}>()
const emit = defineEmits<{
  'update:query': [query: string]
  'update:sort': [sort: CollectionSort]
  toggle: [facet: CollectionFacet, value: string]
}>()
const { t } = useMessages()

const sorts: CollectionSort[] = [
  CollectionSort.Quality,
  CollectionSort.Id,
  CollectionSort.Name,
]
const sortText: Record<CollectionSort, MessageKey<MessageSchema>> = {
  [CollectionSort.Quality]: 'collection.sort.quality',
  [CollectionSort.Id]: 'collection.sort.id',
  [CollectionSort.Name]: 'collection.sort.name',
}

// A single-choice group empties when its chosen item is clicked again; a sort always has one.
const onSort = (value: unknown) => {
  const next = sorts.find((s) => s === value)
  if (next) emit('update:sort', next)
}

const chips = computed(() =>
  collectionFacetOrder.flatMap((facet) =>
    props.filter.picks[facet].map((value) => ({
      facet,
      value,
      label: collectionFacetValueLabel(t, facet, value),
    })),
  ),
)
</script>

<template>
  <CardHeader class="flex-wrap">
    <CardTitle class="tabular-nums"
      >{{ shown }} / {{ total }} {{ t('collection.items') }}</CardTitle
    >
    <div class="flex flex-wrap items-center gap-2">
      <Input
        :model-value="query"
        :placeholder="t('collection.search')"
        class="w-search"
        @update:model-value="emit('update:query', String($event))"
      />
      <span class="text-label">{{ t('collection.sortBy') }}</span>
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
      t('collection.activeFilters')
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
