<script setup lang="ts">
import { LayersIcon } from '@lucide/vue'
import { computed, watch } from 'vue'
import { useRoute } from 'vue-router'
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import { Button, ButtonVariant } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { useOnActiveProfile } from '@/composables/useOnActiveProfile'
import { useTabView } from '@/composables/useTabView'
import type { ScrollOffset } from '@/lib/scale/scrollOffset'
import { useMessages } from '@/i18n'
import { singleQuery } from '@/lib/search/queryParam'
import { emptyList, isFiltering } from '@/lib/facets/emptyList'
import {
  CollectionFacet,
  CollectionSort,
  collectionFaceting,
  filterForQuery,
  emptyCollectionFilter,
  sortItems,
} from '@/lib/collection/collectionFacets'
import type { CollectionFilter } from '@/lib/collection/collectionFacets'
import { itemStateOrder } from '@/lib/collection/itemState'
import { useCollectionStore } from '@/stores/views'
import { LoadStatus } from '@/stores/loadStatus'
import ScreenHeader from './ScreenHeader.vue'
import DiagnosticsList from '@/components/diagnostics/DiagnosticsList.vue'
import { collectionEntries } from '@/lib/diagnostics/collection'

import FilterBar from '@/components/facets/FilterBar.vue'
import {
  barLabels,
  collectionFacetTitle,
  collectionFacetValueLabel,
  collectionSlots,
  itemStateDot,
  itemStateText,
  sortOrder,
  sortText,
} from './collection/collectionLabels'
import CollectionTable from './collection/CollectionTable.vue'
import { collectionView } from './collection/tabView'

import ProfileError from './profile/ProfileError.vue'

const store = useCollectionStore()
const { t } = useMessages()

useOnActiveProfile(() => store.load())

// The filter and the sort belong to the tab, not to this component: leaving and coming back —
// through a tear-off, a restart, or the back button — finds them where they were left (B39). It
// opens on what hasn't been found.
const reading = useTabView(collectionView)
const setOffset = (offset: ScrollOffset) => {
  reading.value = { ...reading.value, offset }
}
const filter = computed({
  get: () => reading.value.filter,
  set: (value: CollectionFilter) => {
    reading.value = { ...reading.value, filter: value }
  },
})

// A Search row opens this list already filtered on the name it found (B3, spec 3.5 Decision 8).
const route = useRoute()
watch(
  () => route.query.q,
  (value) => {
    const q = singleQuery(value)
    if (q !== null) filter.value = filterForQuery(q)
  },
  { immediate: true },
)
const sort = computed({
  get: () => reading.value.sort,
  set: (value: CollectionSort) => {
    reading.value = { ...reading.value, sort: value }
  },
})

const items = computed(() => store.view?.items ?? [])
// The pools are the view's, so the faceting is too: its options cannot be read off the items.
const faceting = computed(() => collectionFaceting(store.view?.pools ?? []))
const valueLabel = (facet: CollectionFacet, value: string) =>
  collectionFacetValueLabel(t, facet, value)
const rows = computed(() =>
  sortItems(
    items.value.filter((item) => faceting.value.matches(item, filter.value)),
    sort.value,
  ),
)

const setPicks = (facet: CollectionFacet, picked: string[]) => {
  filter.value = {
    ...filter.value,
    picks: { ...filter.value.picks, [facet]: picked },
  }
}
// A machine without the game answers this view with no items at all (`noCatalog`), and an empty
// list is not a filter that matched nothing: what was never read must not be drawn as "not
// found", and the button that clears a filter belongs where there is a filter.
const empty = computed(() =>
  emptyList(items.value.length, isFiltering(filter.value), {
    empty: 'collection.empty',
    noResults: 'collection.noResults',
  }),
)
const setQuery = (query: string) => {
  filter.value = { ...filter.value, query }
}
const setSort = (next: CollectionSort) => {
  sort.value = next
}
const reset = () => {
  filter.value = emptyCollectionFilter()
}
</script>

<template>
  <div class="flex max-w-250 flex-col gap-4">
    <ScreenHeader :icon="LayersIcon" :title="t('routes.collection')">{{
      t('collection.intro')
    }}</ScreenHeader>
    <ProfileError
      v-if="store.status === LoadStatus.Failed"
      :error="store.error"
      @retry="store.load()"
    />
    <template v-else-if="store.view">
      <DiagnosticsList :entries="collectionEntries(store.view.diagnostics)" />
      <Card>
        <FilterBar
          :shown="rows.length"
          :total="items.length"
          :query="filter.query"
          :sort="sort"
          :sorts="sortOrder"
          :sort-text="sortText"
          :rows="items"
          :faceting="faceting"
          :filter="filter"
          :facets="collectionSlots"
          :state="{
            facet: CollectionFacet.State,
            order: itemStateOrder,
            dot: itemStateDot,
            text: itemStateText,
          }"
          :title="collectionFacetTitle"
          :value-label="valueLabel"
          :labels="barLabels"
          @update:query="setQuery"
          @update:sort="setSort"
          @update:picks="setPicks"
          @reset="reset"
        />
        <CollectionTable
          v-if="rows.length > 0"
          :items="rows"
          :offset="reading.offset"
          @offset-change="setOffset"
        />
        <div v-else class="flex flex-col items-start gap-3 p-4">
          <EmptyCategory>{{ t(empty.text) }}</EmptyCategory>
          <Button
            v-if="empty.reset"
            :variant="ButtonVariant.Outline"
            @click="reset"
            >{{ t('filters.reset') }}</Button
          >
        </div>
      </Card>
    </template>
    <div v-else class="flex flex-col gap-4">
      <Skeleton class="h-8 w-120" />
      <Skeleton class="h-12 w-full" />
      <Skeleton class="h-150 w-full" />
    </div>
  </div>
</template>
