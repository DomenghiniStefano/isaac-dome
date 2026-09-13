<script setup lang="ts">
import { LayersIcon } from '@lucide/vue'
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import { Button, ButtonVariant } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { useOnActiveProfile } from '@/composables/useOnActiveProfile'
import { useMessages } from '@/i18n'
import { singleQuery } from '@/lib/search/queryParam'
import {
  CollectionFacet,
  CollectionSort,
  defaultCollectionFilter,
  filterForQuery,
  emptyCollectionFilter,
  matchesCollectionFilter,
  sortItems,
} from '@/lib/collection/collectionFilter'
import type { CollectionFilter } from '@/lib/collection/collectionFilter'
import { itemStateCounts } from '@/lib/collection/itemState'
import { useCollectionStore } from '@/stores/views'
import { LoadStatus } from '@/stores/loadStatus'
import ScreenHeader from './ScreenHeader.vue'
import DiagnosticsList from '@/components/diagnostics/DiagnosticsList.vue'
import { collectionEntries } from '@/lib/diagnostics/collection'
import CollectionFacetDrawer from './collection/CollectionFacetDrawer.vue'
import CollectionStateToggle from './collection/CollectionStateToggle.vue'
import CollectionTable from './collection/CollectionTable.vue'
import CollectionToolbar from './collection/CollectionToolbar.vue'
import ProfileError from './profile/ProfileError.vue'

const store = useCollectionStore()
const { t } = useMessages()

useOnActiveProfile(() => store.load())

// The filter belongs to this screen: leaving the tab resets it, until tabs keep their state. It
// opens on what hasn't been found.
const filter = ref<CollectionFilter>(defaultCollectionFilter())

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
const sort = ref<CollectionSort>(CollectionSort.Quality)

const items = computed(() => store.view?.items ?? [])
const counts = computed(() => itemStateCounts(items.value))
const rows = computed(() =>
  sortItems(
    items.value.filter((item) => matchesCollectionFilter(item, filter.value)),
    sort.value,
  ),
)

const setPicks = (facet: CollectionFacet, picked: string[]) => {
  filter.value = {
    ...filter.value,
    picks: { ...filter.value.picks, [facet]: picked },
  }
}
const toggle = (facet: CollectionFacet, value: string) => {
  const picked = filter.value.picks[facet]
  setPicks(
    facet,
    picked.includes(value)
      ? picked.filter((v) => v !== value)
      : [...picked, value],
  )
}
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
      <CollectionStateToggle
        :counts="counts"
        :picked="filter.picks[CollectionFacet.State]"
        @update="setPicks(CollectionFacet.State, $event)"
      />
      <CollectionFacetDrawer
        :items="items"
        :pools="store.view.pools"
        :filter="filter"
        @toggle="toggle"
        @reset="reset"
      />
      <Card>
        <CollectionToolbar
          :shown="rows.length"
          :total="items.length"
          :filter="filter"
          :query="filter.query"
          :sort="sort"
          @update:query="setQuery"
          @update:sort="setSort"
          @toggle="toggle"
        />
        <CollectionTable v-if="rows.length > 0" :items="rows" />
        <div v-else class="flex flex-col items-start gap-3 p-4">
          <EmptyCategory>{{ t('collection.noResults') }}</EmptyCategory>
          <Button :variant="ButtonVariant.Outline" @click="reset">{{
            t('collection.resetFilters')
          }}</Button>
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
