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
import { PageScroll } from '@/components/ui/virtual'
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
const { queue } = useQueueOffer()
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
  <!-- The screen scrolls as a page (`PageScroll`): the header, the diagnostics and the filter bar
       go by with the list, and only the table's column header stays pinned. The gutter is the
       children's, so the sticky header pins at the box's edge and not one padding below it. -->
  <PageScroll>
    <div class="flex flex-col gap-4 px-5.5 pt-5 pb-5">
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
        <Card>
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
          <UnlockTable v-if="rows.length > 0" :nodes="rows" />
          <ListEmptyState v-else :empty="empty" @reset="reset" />
        </Card>
      </template>
      <ScreenSkeleton v-else />
    </div>
  </PageScroll>
</template>
