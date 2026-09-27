<script setup lang="ts">
import {
  ArrowDownIcon,
  ArrowUpIcon,
  LayoutGridIcon,
  TableIcon,
} from '@lucide/vue'
import { computed } from 'vue'
import ListEmptyState from '@/components/data-state/ListEmptyState.vue'
import FilterBar from '@/components/facets/FilterBar.vue'
import { Card } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { PageScroll } from '@/components/ui/virtual'
import {
  ToggleGroup,
  ToggleGroupItem,
  ToggleGroupType,
} from '@/components/ui/toggle-group'
import { useFacetedReading } from '@/composables/useFacetedReading'
import { useMessages } from '@/i18n'
import type { WikiPageRef } from '@/lib/ipc/types'
import { emptyList, isFiltering } from '@/lib/facets/emptyList'
import type { ScrollOffset } from '@/lib/scale/scrollOffset'
import { filterPages } from '@/lib/wiki/listFilter'
import {
  emptyWikiListFilter,
  wikiBar,
  wikiFaceting,
  wikiFacetValueLabel,
} from '@/lib/wiki/listFacets'
import type { WikiFacet } from '@/lib/wiki/listFacets'
import { SortDirection, sortPages } from '@/lib/wiki/listSort'
import type { WikiSortSpec } from '@/lib/wiki/listSort'
import {
  ListViewMode,
  listViewFor,
  setListViewFor,
} from '@/lib/wiki/listViewMode'
import { categoryProgress } from '@/lib/wiki/progress'
import { WikiCategory } from '@/router/routeTable'
import { useTabsStore } from '@/stores/tabs'
import { useWikiStore } from '@/stores/wiki'
import WikiCardGrid from './list/WikiCardGrid.vue'
import WikiListHero from './list/WikiListHero.vue'
import WikiTable from './list/WikiTable.vue'
import { wikiView } from './tabView'

const props = defineProps<{ category: WikiCategory }>()
const wiki = useWikiStore()
const tabs = useTabsStore()
const { t } = useMessages()

// The filter, the sort and the position are the entry's reading (`tabView.ts`): they come
// back after a tab switch, a back, a tear-off. A tab that moves to another category is a new
// entry, so it starts clean without anything here having to clear it.
const { reading, update, filter, setPicks, setQuery, reset } =
  useFacetedReading(wikiView, emptyWikiListFilter)
const setOffset = (offset: ScrollOffset) => update({ offset })
const setSort = (sort: WikiSortSpec) => update({ sort })
const setSortKey = (key: string) =>
  setSort({ key, direction: reading.value.sort.direction })

const progressFor = (target: WikiPageRef['target']) => wiki.progressFor(target)

const all = computed(() => wiki.index?.pages ?? [])
const categoryPages = computed(() =>
  all.value.filter((page) => page.category === props.category),
)
const total = computed(() => categoryPages.value.length)

// One faceting instance per render of the save's own answers: `progressFor` closes over the
// store, so a save switch — which changes what every page's profile facet reads — reaches the
// filter without a second lookup.
const faceting = computed(() => wikiFaceting(progressFor))
const filtered = computed(() =>
  filterPages(all.value, props.category, filter.value, faceting.value),
)
const pages = computed(() =>
  sortPages(filtered.value, props.category, reading.value.sort),
)

const progress = computed(() =>
  categoryProgress(all.value, props.category, progressFor),
)
const bar = computed(() =>
  wikiBar(props.category, progress.value !== null, faceting.value),
)
const valueLabel = (facet: WikiFacet, value: string) =>
  wikiFacetValueLabel(t, facet, value)

const empty = computed(() =>
  emptyList(total.value, isFiltering(filter.value), {
    empty: 'wiki.emptyCategory',
    noResults: 'wiki.noResults',
  }),
)
// No picture on any page is the game's absence, not 900 pages without art.
const noCatalog = computed(
  () => all.value.length > 0 && all.value.every((p) => p.iconUrl === null),
)

// The hero's picture (design decision 8b): the curated sample this category draws on the
// landing, framed through a page of the same category so `WikiFigure` still picks the right
// frame and fallback (`WikiListHero`'s own doc comment says why a target is borrowed at all).
const heroSample = computed(
  () =>
    wiki.index?.samples.find((sample) => sample.category === props.category) ??
    null,
)
const representative = computed(() => categoryPages.value[0]?.target ?? null)

const viewMode = computed(() => listViewFor(props.category))
const onViewMode = (value: unknown) => {
  if (value === ListViewMode.Grid || value === ListViewMode.Table)
    setListViewFor(props.category, value)
}
const onDirection = (value: unknown) => {
  if (value === SortDirection.Asc || value === SortDirection.Desc)
    setSort({ ...reading.value.sort, direction: value })
}

const open = (page: WikiPageRef, event: MouseEvent) =>
  tabs.openPage(page.target, event.ctrlKey)
</script>

<template>
  <!-- The whole screen scrolls (`PageScroll`): the band and the filters go by with the list,
       and only the table's column header stays pinned. -->
  <PageScroll>
    <WikiListHero
      :category="category"
      :representative="representative"
      :sample-url="heroSample?.iconUrl ?? null"
      :count="total"
      :progress="progress"
    />
    <div class="flex flex-col gap-3 px-5.5 pt-4 pb-5">
      <p v-if="noCatalog" class="text-caption text-subtle-foreground">
        {{ t('wiki.noCatalog') }}
      </p>
      <Card v-if="wiki.index">
        <FilterBar
          :bar="bar"
          :rows="categoryPages"
          :filter="filter"
          :shown="pages.length"
          :sort="reading.sort.key"
          :value-label="valueLabel"
          @update:query="setQuery"
          @update:picks="setPicks"
          @update:sort="setSortKey"
          @reset="reset"
        />
        <div
          class="flex flex-wrap items-center justify-end gap-2 border-b border-hairline bg-muted px-3 py-2"
        >
          <ToggleGroup
            :type="ToggleGroupType.Single"
            :model-value="reading.sort.direction"
            @update:model-value="onDirection"
          >
            <ToggleGroupItem
              :value="SortDirection.Asc"
              :aria-label="t('wiki.list.ascending')"
              ><ArrowUpIcon
            /></ToggleGroupItem>
            <ToggleGroupItem
              :value="SortDirection.Desc"
              :aria-label="t('wiki.list.descending')"
              ><ArrowDownIcon
            /></ToggleGroupItem>
          </ToggleGroup>
          <ToggleGroup
            :type="ToggleGroupType.Single"
            :model-value="viewMode"
            @update:model-value="onViewMode"
          >
            <ToggleGroupItem :value="ListViewMode.Grid" class="gap-2"
              ><LayoutGridIcon />{{ t('wiki.list.grid') }}</ToggleGroupItem
            >
            <ToggleGroupItem :value="ListViewMode.Table" class="gap-2"
              ><TableIcon />{{ t('wiki.list.table') }}</ToggleGroupItem
            >
          </ToggleGroup>
        </div>
        <template v-if="pages.length > 0">
          <WikiCardGrid
            v-if="viewMode === ListViewMode.Grid"
            :pages="pages"
            :offset="reading.offset"
            @offset-change="setOffset"
            @open="open"
          />
          <WikiTable
            v-else
            :pages="pages"
            :category="category"
            :sort="reading.sort"
            :offset="reading.offset"
            @offset-change="setOffset"
            @open="open"
          />
        </template>
        <ListEmptyState v-else :empty="empty" @reset="reset" />
      </Card>
      <Skeleton v-else class="h-150 w-full" />
    </div>
  </PageScroll>
</template>
