<script setup lang="ts">
import { LayersIcon } from '@lucide/vue'
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import ListEmptyState from '@/components/data-state/ListEmptyState.vue'
import ScreenSkeleton from '@/components/data-state/ScreenSkeleton.vue'
import DiagnosticsList from '@/components/diagnostics/DiagnosticsList.vue'
import FilterBar from '@/components/facets/FilterBar.vue'
import FindBar from '@/components/find/FindBar.vue'
import { Card } from '@/components/ui/card'
import { useFacetedReading } from '@/composables/useFacetedReading'
import { useFind } from '@/composables/useFind'
import { useOnActiveProfile } from '@/composables/useOnActiveProfile'
import { useMessages } from '@/i18n'
import {
  collectionFaceting,
  emptyCollectionFilter,
  filterForQuery,
  sortItems,
} from '@/lib/collection/collectionFacets'
import type { CollectionFacet } from '@/lib/collection/collectionFacets'
import { collectionEntries } from '@/lib/diagnostics/collection'
import { emptyList, isFiltering } from '@/lib/facets/emptyList'
import type { FindState } from '@/lib/find/findState'
import { singleQuery } from '@/lib/search/queryParam'
import { LoadStatus } from '@/stores/loadStatus'
import { useCollectionStore } from '@/stores/views'
import ScreenHeader from '@/components/screen/ScreenHeader.vue'
import CollectionTable from './collection/CollectionTable.vue'
import {
  collectionBar,
  collectionFacetValueLabel,
} from '@/lib/collection/collectionLabels'
import { collectionView } from './collection/tabView'
import ProfileError from '@/components/data-state/ProfileError.vue'

const store = useCollectionStore()
const { t } = useMessages()

useOnActiveProfile(() => store.load())

// It opens on what hasn't been found (the view's default filter); a reset clears every pick.
const { reading, update, filter, setPicks, setQuery, reset } =
  useFacetedReading(collectionView, emptyCollectionFilter)

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

const items = computed(() => store.view?.items ?? [])
// The pools are the view's, so the faceting is too: its options cannot be read off the items.
const faceting = computed(() => collectionFaceting(store.view?.pools ?? []))
const bar = computed(() => collectionBar(faceting.value))
const valueLabel = (facet: CollectionFacet, value: string) =>
  collectionFacetValueLabel(t, facet, value)
const rows = computed(() =>
  sortItems(
    items.value.filter((item) => faceting.value.matches(item, filter.value)),
    reading.value.sort,
  ),
)

// Find-in-page (B67). It does not compete with the filter above it, it composes with it: the
// filter decides which rows exist, and the find walks the ones that are left. That is why the
// haystack is `rows` and not `items` — searching rows a filter has hidden would scroll to
// something that is not on the screen.
const table = ref<InstanceType<typeof CollectionTable> | null>(null)
const findState = computed({
  get: () => reading.value.find,
  set: (find: FindState | null) => update({ find }),
})
// Destructured on purpose: a template unwraps refs that are setup bindings, not refs sitting
// inside an object.
const {
  open: findOpen,
  query: findQuery,
  current: findCurrent,
  close: closeFind,
  goTo: goToMatch,
} = useFind(findState, (index) => table.value?.scrollToIndex(index))

// The key is a string because the bar is not the Collection's: a wiki page and a run do not
// have numeric ids, and the bar must not learn what kind of thing it is walking.
const haystack = computed(() =>
  rows.value.map((item) => ({ key: String(item.id), text: item.name })),
)

// A machine without the game answers this view with no items at all (`noCatalog`), and an empty
// list is not a filter that matched nothing: what was never read must not be drawn as "not
// found", and the button that clears a filter belongs where there is a filter.
const empty = computed(() =>
  emptyList(items.value.length, isFiltering(filter.value), {
    empty: 'collection.empty',
    noResults: 'collection.noResults',
  }),
)
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
          :bar="bar"
          :rows="items"
          :filter="filter"
          :shown="rows.length"
          :sort="reading.sort"
          :value-label="valueLabel"
          @update:query="setQuery"
          @update:sort="update({ sort: $event })"
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
          @offset-change="update({ offset: $event })"
        />
        <ListEmptyState v-else :empty="empty" @reset="reset" />
      </Card>
    </template>
    <ScreenSkeleton v-else />
  </div>
</template>
