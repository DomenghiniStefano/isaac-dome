<script setup lang="ts">
import { SearchIcon } from '@lucide/vue'
import { useDebounceFn } from '@vueuse/core'
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import { Card } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { useSearch } from '@/composables/useSearch'
import { useTabView } from '@/composables/useTabView'
import type { ScrollOffset } from '@/lib/scale/scrollOffset'
import { useMessages } from '@/i18n'
import { Timing } from '@/lib/constants/timing'
import { SearchLimit } from '@/lib/ipc/search'
import { SearchDiagnostic } from '@/lib/ipc/types'
import { singleQuery } from '@/lib/search/queryParam'
import {
  RowGroup,
  filterGroups,
  groupCounts,
  matchingScreens,
  screenEntries,
  searchRows,
} from '@/lib/search/rows'
import type { SearchRow } from '@/lib/search/rows'
import { RouteName } from '@/router/routeTable'
import { useTabsStore } from '@/stores/tabs'
import ScreenHeader from './ScreenHeader.vue'
import DiagnosticsList from '@/components/diagnostics/DiagnosticsList.vue'
import { searchEntries } from '@/lib/diagnostics/search'
import SearchResults from './search/SearchResults.vue'
import SearchToolbar from './search/SearchToolbar.vue'
import { searchView } from './search/tabView'

const route = useRoute()
const tabs = useTabsStore()
const { t } = useMessages()
const { view, ask } = useSearch(SearchLimit.Screen)

// The query lives in the tab's location: typing navigates it, so the tab *is* the search and
// survives as one (spec 3.5, Decision 8). The field keeps what is typed while the debounce
// waits, so the caret never jumps.
const typed = ref(singleQuery(route.query.q) ?? '')

// `refine` and not `navigate`: the debounce can land after the user has gone back or moved
// to another tab, and what was typed here has no business dragging a tab elsewhere.
const navigate = useDebounceFn((q: string) => {
  tabs.refine({ name: RouteName.Search, query: q === '' ? {} : { q } })
}, Timing.SearchDebounce)

watch(
  typed,
  (q) => {
    void navigate(q)
    ask(q)
  },
  { immediate: true },
)

// A location opened from elsewhere — the palette's "all results" row — brings its own query.
watch(
  () => route.query.q,
  (value) => {
    const q = singleQuery(value)
    if (q !== null && q !== typed.value) typed.value = q
  },
)

const diagnostics = computed(() => view.value?.diagnostics ?? [])
const catalog = computed(
  () => !diagnostics.value.includes(SearchDiagnostic.NoCatalog),
)

// The whole answer, uncapped: the per-group cap belongs to the palette.
const allRows = computed(() =>
  searchRows(
    view.value?.hits ?? [],
    matchingScreens(screenEntries(t), typed.value),
    { catalog: catalog.value, cap: null },
  ),
)

// Which groups the search is narrowed to belongs to the tab, not to this component (B39). The
// query is not here: it is in the location already, because a search is a place you can link to.
const { reading, update } = useTabView(searchView)
const setOffset = (offset: ScrollOffset) => update({ offset })
const picked = computed({
  get: () => reading.value.picked,
  set: (value: RowGroup[]) => update({ picked: value }),
})
const rows = computed(() => filterGroups(allRows.value, picked.value))
const counts = computed(() => groupCounts(allRows.value))

const total = computed(() => view.value?.total ?? 0)
// More matched than came back: the line says so rather than pretending this is everything.
const limited = computed(() => total.value > (view.value?.hits.length ?? 0))
const hasQuery = computed(() => typed.value.trim() !== '')

// A click opens the row in the active tab; Ctrl opens it beside, as everywhere else.
const open = (row: SearchRow, event: MouseEvent) =>
  tabs.go(row.location, event.ctrlKey)
</script>

<template>
  <div
    class="flex h-full min-h-0 flex-col gap-4 overflow-hidden px-5.5 pt-5 pb-5"
  >
    <ScreenHeader :icon="SearchIcon" :title="t('routes.search')">{{
      t('search.intro')
    }}</ScreenHeader>
    <Input v-model="typed" :placeholder="t('search.placeholder')" />
    <DiagnosticsList v-if="hasQuery" :entries="searchEntries(diagnostics)" />
    <template v-if="hasQuery">
      <SearchToolbar
        :counts="counts"
        :picked="picked"
        :shown="rows.length"
        :total="total"
        :limited="limited"
        @update="picked = $event"
      />
      <Card class="min-h-0 flex-1">
        <SearchResults
          v-if="rows.length > 0"
          :rows="rows"
          :offset="reading.offset"
          @open="open"
          @offset-change="setOffset"
        />
        <div v-else class="p-4">
          <EmptyCategory>{{ t('search.empty') }}</EmptyCategory>
        </div>
      </Card>
    </template>
  </div>
</template>
