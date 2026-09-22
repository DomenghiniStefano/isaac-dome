<script setup lang="ts">
import { LockOpenIcon } from '@lucide/vue'
import { computed, watch } from 'vue'
import { useRoute } from 'vue-router'
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import QueueError from '@/components/plan/QueueError.vue'
import { Button, ButtonVariant } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { useOnActiveProfile } from '@/composables/useOnActiveProfile'
import { useTabView } from '@/composables/useTabView'
import type { ScrollOffset } from '@/lib/scale/scrollOffset'
import { useMessages } from '@/i18n'
import { singleQuery } from '@/lib/search/queryParam'
import { emptyList, isFiltering } from '@/lib/facets/emptyList'
import { characterForms } from '@/lib/graph/characterName'
import { stateOrder } from '@/lib/graph/nodeState'
import {
  barLabels,
  facetTitle,
  facetValueLabel,
  sortOrder,
  sortText,
  stateDot,
  stateText,
  unlockSlots,
} from './unlock/facetLabels'
import { unlockView } from './unlock/tabView'
import {
  FacetId,
  UnlockSort,
  sortNodes,
  unlockFaceting,
} from '@/lib/graph/unlockFacets'
import type { UnlockFilter } from '@/lib/graph/unlockFacets'
import { queuedIds } from '@/lib/plan/queueRows'
import { useGraphStore } from '@/stores/views'
import { LoadStatus } from '@/stores/loadStatus'
import { useQueueStore } from '@/stores/queue'
import ScreenHeader from './ScreenHeader.vue'
import ProfileError from './profile/ProfileError.vue'
import FilterBar from '@/components/facets/FilterBar.vue'
import DiagnosticsList from '@/components/diagnostics/DiagnosticsList.vue'
import { unlockEntries } from '@/lib/diagnostics/unlock'
import UnlockTable from './unlock/UnlockTable.vue'

const graph = useGraphStore()
const queue = useQueueStore()
const { t } = useMessages()

useOnActiveProfile(async () => {
  await Promise.all([graph.load(), queue.load()])
})

// The filter and the sort belong to the tab, not to this component: leaving and coming back —
// through a tear-off, a restart, or the back button — finds them where they were left (B39).
const reading = useTabView(unlockView)
const setOffset = (offset: ScrollOffset) => {
  reading.value = { ...reading.value, offset }
}
const filter = computed({
  get: () => reading.value.filter,
  set: (value: UnlockFilter) => {
    reading.value = { ...reading.value, filter: value }
  },
})

// A Search row opens this list already filtered on the name it found (B3, spec 3.5 Decision 8).
const route = useRoute()
watch(
  () => route.query.q,
  (value) => {
    const q = singleQuery(value)
    if (q !== null) filter.value = { ...filter.value, query: q }
  },
  { immediate: true },
)
// "Vedile tutte" on the landing page lands here with the state facet already picked. A value
// we never wrote is ignored rather than picked: the facet holds `NodeState`s, and an unknown
// string would filter everything away and read as an empty profile.
watch(
  () => route.query.state,
  (value) => {
    const wanted = singleQuery(value)
    const state = stateOrder.find((s) => s === wanted)
    if (state)
      filter.value = {
        ...filter.value,
        picks: { ...filter.value.picks, [FacetId.State]: [state] },
      }
  },
  { immediate: true },
)
const sort = computed({
  get: () => reading.value.sort,
  set: (value: UnlockSort) => {
    reading.value = { ...reading.value, sort: value }
  },
})

const nodes = computed(() => graph.view?.unlock.nodes ?? [])
// The character facet's labels: the value is an id, the name is read from the nodes.
const characters = computed(() => characterForms(nodes.value))

// A picked value in words. The Character facet stores ids (B28), so the label needs the forms
// the nodes carry: it is the screen that has them, not the control that draws the chip.
const valueLabel = (facet: FacetId, value: string) =>
  facetValueLabel(t, facet, value, characters.value)
const rows = computed(() =>
  sortNodes(
    nodes.value.filter((node) => unlockFaceting.matches(node, filter.value)),
    sort.value,
  ),
)

// A queue that couldn't be read or saved offers nothing: the rows still show, without "in
// coda" or the button.
const queued = computed(() => queuedIds(queue.view))
const canWrite = computed(() => queue.view?.storeAvailable === true)

const setPicks = (facet: FacetId, picked: string[]) => {
  filter.value = {
    ...filter.value,
    picks: { ...filter.value.picks, [facet]: picked },
  }
}
// An empty list is not a filter that matched nothing: a view that came back with no nodes at
// all has nothing to clear, and offering the button there would undo nothing. Unreachable
// today — a machine without the game still gets every node, counted as unread — so this is the
// guard that keeps the next view from reintroducing what B48 corrected.
const empty = computed(() =>
  emptyList(nodes.value.length, isFiltering(filter.value), {
    empty: 'unlock.empty',
    noResults: 'unlock.noResults',
  }),
)
const setQuery = (query: string) => {
  filter.value = { ...filter.value, query }
}
const setSort = (next: UnlockSort) => {
  sort.value = next
}
const reset = () => {
  filter.value = unlockFaceting.empty()
}
</script>

<template>
  <!-- A filling screen (spec 3.13a §4): the header, the diagnostics and the filter bar stay put,
       and the table takes the height that is left. `pb-5` and not a flowing screen's `pb-15`,
       because nothing ever scrolls past the bottom here. -->
  <div
    class="flex h-full min-h-0 flex-col gap-4 overflow-hidden px-5.5 pt-5 pb-5"
  >
    <ScreenHeader :icon="LockOpenIcon" :title="t('routes.unlock')">{{
      t('unlock.intro')
    }}</ScreenHeader>
    <ProfileError
      v-if="graph.status === LoadStatus.Failed"
      :error="graph.error"
      @retry="graph.load()"
    />
    <template v-else-if="graph.view">
      <DiagnosticsList
        :entries="unlockEntries(graph.view.unlock.diagnostics)"
      />
      <QueueError v-if="queue.mutationFailed" :error="queue.mutationError" />
      <Card class="min-h-0 flex-1">
        <FilterBar
          :shown="rows.length"
          :total="nodes.length"
          :query="filter.query"
          :sort="sort"
          :sorts="sortOrder"
          :sort-text="sortText"
          :rows="nodes"
          :faceting="unlockFaceting"
          :filter="filter"
          :facets="unlockSlots"
          :state="{
            facet: FacetId.State,
            order: stateOrder,
            dot: stateDot,
            text: stateText,
          }"
          :title="facetTitle"
          :value-label="valueLabel"
          :labels="barLabels"
          @update:query="setQuery"
          @update:sort="setSort"
          @update:picks="setPicks"
          @reset="reset"
        />
        <UnlockTable
          v-if="rows.length > 0"
          :nodes="rows"
          :queued="queued"
          :can-write="canWrite"
          :busy="queue.busy"
          :offset="reading.offset"
          @offset-change="setOffset"
          @add="queue.add"
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
