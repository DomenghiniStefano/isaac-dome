<script setup lang="ts">
import { computed } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import {
  CardCollapsible,
  CardCollapsibleContent,
  CardCollapsibleTrigger,
} from '@/components/ui/card'
import { Checkbox } from '@/components/ui/checkbox'
import { Label } from '@/components/ui/label'
import { useMessages } from '@/i18n'
import {
  CollectionFacet,
  activeCollectionFilterCount,
  collectionFacetCounts,
  collectionFacetOptions,
} from '@/lib/collection/collectionFilter'
import type { CollectionFilter } from '@/lib/collection/collectionFilter'
import type { CollectionItem } from '@/lib/ipc/types'
import {
  collectionFacetTitle,
  collectionFacetValueLabel,
} from './collectionLabels'

const props = defineProps<{
  items: CollectionItem[]
  pools: string[]
  filter: CollectionFilter
}>()
const emit = defineEmits<{
  toggle: [facet: CollectionFacet, value: string]
  reset: []
}>()
const { t } = useMessages()

// The state has its own control above the table; the drawer holds the other four.
const drawerFacets: CollectionFacet[] = [
  CollectionFacet.Quality,
  CollectionFacet.Pool,
  CollectionFacet.Kind,
  CollectionFacet.Origin,
]

// Each count is over the items every other facet and the search leave: it says what picking the
// value would give. A value that would give nothing, and isn't picked, is not offered at all:
// it could not be picked, and reading it with a 0 beside it is noise (`docs/BACKLOG.md` B29).
const columns = computed(() =>
  drawerFacets.map((facet) => {
    const counts = collectionFacetCounts(props.items, props.filter, facet)
    const picked = props.filter.picks[facet]
    return {
      facet,
      values: collectionFacetOptions(props.pools, facet)
        .map((value) => ({
          value,
          label: collectionFacetValueLabel(t, facet, value),
          count: counts.get(value) ?? 0,
          picked: picked.includes(value),
        }))
        .filter((option) => option.count > 0 || option.picked),
    }
  }),
)

const active = computed(() => activeCollectionFilterCount(props.filter))
</script>

<template>
  <CardCollapsible>
    <CardCollapsibleTrigger>
      {{ t('collection.facets') }}
      <template #summary>{{
        active > 0
          ? `${t('collection.activeFilters')}: ${active}`
          : t('collection.noFilters')
      }}</template>
    </CardCollapsibleTrigger>
    <CardCollapsibleContent class="flex flex-col gap-3">
      <div class="grid grid-cols-4 gap-4">
        <div
          v-for="column in columns"
          :key="column.facet"
          class="flex min-w-0 flex-col gap-1.5"
        >
          <span class="text-label text-subtle-foreground">{{
            t(collectionFacetTitle[column.facet])
          }}</span>
          <Label
            v-for="entry in column.values"
            :key="entry.value"
            class="flex items-center gap-2"
          >
            <Checkbox
              :model-value="entry.picked"
              @update:model-value="emit('toggle', column.facet, entry.value)"
            />
            <span
              class="min-w-0 flex-1 truncate text-caption text-foreground"
              >{{ entry.label }}</span
            >
            <span class="text-label text-subtle-foreground tabular-nums">{{
              entry.count
            }}</span>
          </Label>
        </div>
      </div>
      <Button
        :variant="ButtonVariant.Outline"
        :size="ButtonSize.Compact"
        class="self-start"
        :disabled="active === 0"
        @click="emit('reset')"
        >{{ t('collection.reset') }}</Button
      >
    </CardCollapsibleContent>
  </CardCollapsible>
</template>
