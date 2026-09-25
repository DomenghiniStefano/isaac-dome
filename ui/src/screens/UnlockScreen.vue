<script setup lang="ts">
import { LockOpenIcon } from '@lucide/vue'
import { computed, watch } from 'vue'
import { useRoute } from 'vue-router'
import ListEmptyState from '@/components/data-state/ListEmptyState.vue'
import ScreenSkeleton from '@/components/data-state/ScreenSkeleton.vue'
import DiagnosticsList from '@/components/diagnostics/DiagnosticsList.vue'
import FilterBar from '@/components/facets/FilterBar.vue'
import QueueError from '@/components/plan/QueueError.vue'
import { Card } from '@/components/ui/card'
import { useFacetedReading } from '@/composables/useFacetedReading'
import { useOnActiveProfile } from '@/composables/useOnActiveProfile'
import { useQueueOffer } from '@/composables/useQueueOffer'
import { useMessages } from '@/i18n'
import { unlockEntries } from '@/lib/diagnostics/unlock'
import { emptyList, isFiltering } from '@/lib/facets/emptyList'
import { characterForms } from '@/lib/graph/characterName'
import { NodeState } from '@/lib/graph/nodeState'
import { oneOf } from '@/lib/oneOf'
import { FacetId, sortNodes, unlockFaceting } from '@/lib/graph/unlockFacets'
import { singleQuery } from '@/lib/search/queryParam'
import { LoadStatus } from '@/stores/loadStatus'
import { useGraphStore } from '@/stores/views'
import ScreenHeader from '@/components/screen/ScreenHeader.vue'
import ProfileError from '@/components/data-state/ProfileError.vue'
import UnlockTable from './unlock/UnlockTable.vue'
import { unlockBar, unlockFacetValueLabel } from '@/lib/graph/unlockLabels'
import { unlockView } from './unlock/tabView'

const graph = useGraphStore()
const { queue, queued, canWrite } = useQueueOffer()
const { t } = useMessages()

useOnActiveProfile(async () => {
  await Promise.all([graph.load(), queue.load()])
})

const { reading, update, filter, setPicks, setQuery, reset } =
  useFacetedReading(unlockView, unlockFaceting.empty)

// A Search row opens this list already filtered on the name it found (B3, spec 3.5 Decision 8).
const route = useRoute()
watch(
  () => route.query.q,
  (value) => {
    const q = singleQuery(value)
    if (q !== null) setQuery(q)
  },
  { immediate: true },
)
// "Vedile tutte" on the landing page lands here with the state facet already picked. A value
// we never wrote is ignored rather than picked: the facet holds `NodeState`s, and an unknown
// string would filter everything away and read as an empty profile.
watch(
  () => route.query.state,
  (value) => {
    const state = oneOf(NodeState, singleQuery(value) ?? '')
    if (state) setPicks(FacetId.State, [state])
  },
  { immediate: true },
)

const nodes = computed(() => graph.view?.unlock.nodes ?? [])
// The character facet's labels: the value is an id, the name is read from the nodes.
const characters = computed(() => characterForms(nodes.value))

// A picked value in words. The Character facet stores ids (B28), so the label needs the forms
// the nodes carry: it is the screen that has them, not the control that draws the chip.
const valueLabel = (facet: FacetId, value: string) =>
  unlockFacetValueLabel(t, facet, value, characters.value)
const rows = computed(() =>
  sortNodes(
    nodes.value.filter((node) => unlockFaceting.matches(node, filter.value)),
    reading.value.sort,
  ),
)

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
          :bar="unlockBar"
          :rows="nodes"
          :filter="filter"
          :shown="rows.length"
          :sort="reading.sort"
          :value-label="valueLabel"
          @update:query="setQuery"
          @update:sort="update({ sort: $event })"
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
          @offset-change="update({ offset: $event })"
          @add="queue.add"
        />
        <ListEmptyState v-else :empty="empty" @reset="reset" />
      </Card>
    </template>
    <ScreenSkeleton v-else />
  </div>
</template>
