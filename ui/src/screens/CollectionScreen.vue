<script setup lang="ts">
import { LayersIcon } from '@lucide/vue'
import ListEmptyState from '@/components/data-state/ListEmptyState.vue'
import ScreenSkeleton from '@/components/data-state/ScreenSkeleton.vue'
import { computed, nextTick, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import FindBar from '@/components/find/FindBar.vue'
import { Card } from '@/components/ui/card'
import { useOnActiveProfile } from '@/composables/useOnActiveProfile'
import { useShortcut } from '@/composables/useShortcut'
import { useTabView } from '@/composables/useTabView'
import { opensFind } from '@/lib/find/keyboard'
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

// Find-in-page (B67). It does not compete with the filter above it, it composes with it: the
// filter decides which rows exist, and the find walks the ones that are left. That is why the
// haystack is `rows` and not `items` — searching rows a filter has hidden would scroll to
// something that is not on the screen.
//
// The bar is the tab's reading (#79): open or closed, its words and the match it is on come back
// after a tab switch, a back or a tear-off, like the filter above it.
const findOpen = computed(() => reading.value.find !== null)
const findQuery = computed({
  get: () => reading.value.find?.query ?? '',
  set: (query: string) => {
    reading.value = {
      ...reading.value,
      find: { query, current: reading.value.find?.current ?? null },
    }
  },
})
const findCurrent = computed({
  get: () => reading.value.find?.current ?? null,
  set: (current: string | null) => {
    reading.value = {
      ...reading.value,
      find: { query: reading.value.find?.query ?? '', current },
    }
  },
})
const table = ref<InstanceType<typeof CollectionTable> | null>(null)

// The key is a string because the bar is not the Collection's: a wiki page and a run do not
// have numeric ids, and the bar must not learn what kind of thing it is walking.
const haystack = computed(() =>
  rows.value.map((item) => ({ key: String(item.id), text: item.name })),
)

useShortcut((event) => {
  if (!opensFind(event)) return false
  reading.value = {
    ...reading.value,
    find: reading.value.find ?? { query: '', current: null },
  }
  return true
})

const closeFind = () => {
  reading.value = { ...reading.value, find: null }
}

// The row exists in the model before it exists as a node, so the scroll waits a tick for the
// virtualizer to have been told the list it is scrolling in.
const goToMatch = async (index: number) => {
  await nextTick()
  table.value?.scrollToIndex(index)
}

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
  <div
    class="flex h-full min-h-0 flex-col gap-4 overflow-hidden px-5.5 pt-5 pb-5"
  >
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
      <Card class="min-h-0 flex-1">
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
        <FindBar
          v-if="findOpen"
          v-model:query="findQuery"
          v-model:current="findCurrent"
          :rows="haystack"
          class="px-4 py-2"
          @move="goToMatch"
          @close="closeFind"
        />
        <CollectionTable
          v-if="rows.length > 0"
          ref="table"
          :items="rows"
          :offset="reading.offset"
          :find-query="findOpen ? findQuery : ''"
          :find-current="findCurrent"
          @offset-change="setOffset"
        />
        <ListEmptyState v-else :empty="empty" @reset="reset" />
      </Card>
    </template>
    <ScreenSkeleton v-else />
  </div>
</template>
