<script setup lang="ts">
import { FlagIcon } from '@lucide/vue'
import { computed } from 'vue'
import ListEmptyState from '@/components/data-state/ListEmptyState.vue'
import ScreenSkeleton from '@/components/data-state/ScreenSkeleton.vue'
import DiagnosticsList from '@/components/diagnostics/DiagnosticsList.vue'
import FilterBar from '@/components/facets/FilterBar.vue'
import QueueError from '@/components/plan/QueueError.vue'
import { Card } from '@/components/ui/card'
import { useFacetedReading } from '@/composables/useFacetedReading'
import { useOnActiveProfile } from '@/composables/useOnActiveProfile'
import { useQueueOffer } from '@/composables/useQueueOffer'
import { vScrollMemory } from '@/directives/scrollMemory'
import { useMessages } from '@/i18n'
import {
  ChallengeFacet,
  challengeFaceting,
} from '@/lib/challenges/challengeFacets'
import { challengeCharacterNames } from '@/lib/challenges/characterNames'
import { challengeEntries } from '@/lib/diagnostics/challenges'
import { emptyList, isFiltering } from '@/lib/facets/emptyList'
import type { Target } from '@/lib/ipc/types'
import { pageLocation } from '@/lib/wiki/category'
import { LoadStatus } from '@/stores/loadStatus'
import { useTabsStore } from '@/stores/tabs'
import { useChallengesStore } from '@/stores/views'
import ScreenHeader from './ScreenHeader.vue'
import ChallengesTable from './challenges/ChallengesTable.vue'
import {
  challengeBar,
  challengeFacetValueLabel,
} from './challenges/challengeLabels'
import { challengesView } from './challenges/tabView'
import ProfileError from './profile/ProfileError.vue'

const store = useChallengesStore()
const { queue, queued, canWrite } = useQueueOffer()
const tabs = useTabsStore()
const { t } = useMessages()

useOnActiveProfile(async () => {
  await Promise.all([store.load(), queue.load()])
})

const { filter, setPicks, setQuery, reset } = useFacetedReading(
  challengesView,
  challengeFaceting.empty,
)

const all = computed(() => store.view?.challenges ?? [])
const rows = computed(() =>
  all.value.filter((row) => challengeFaceting.matches(row, filter.value)),
)

const characterNames = computed(() => challengeCharacterNames(all.value))
const valueLabel = (facet: ChallengeFacet, value: string) =>
  challengeFacetValueLabel(t, facet, value, characterNames.value)

// A machine without the game answers this view with no challenges at all (`noCatalog`), and an
// empty list is not a filter that matched nothing.
const empty = computed(() =>
  emptyList(all.value.length, isFiltering(filter.value), {
    empty: 'challenges.empty',
    noResults: 'challenges.noResults',
  }),
)

// A reference replaces this tab's page, or opens one beside it with Ctrl — the same action a
// search result has (DESIGN-BRIEF.md §4.2).
const navigate = (target: Target, newTab: boolean) => {
  const location = pageLocation(target)
  if (location !== null) tabs.go(location, newTab)
}
</script>

<template>
  <!-- The screen is the box that scrolls, and the columns' header pins to its top (card #80,
       P3: the rows past the card could not be reached). The gutter is the children's, not the
       box's, as on Completion: a padded box pins its sticky header one padding below its edge,
       and the rows would show through the strip above it. -->
  <div v-scroll-memory="'page'" class="h-full overflow-y-auto">
    <div class="flex flex-col gap-4 px-5.5 pt-5 pb-5">
      <ScreenHeader :icon="FlagIcon" :title="t('routes.challenges')">{{
        t('challenges.intro')
      }}</ScreenHeader>
      <ProfileError
        v-if="store.status === LoadStatus.Failed"
        :error="store.error"
        @retry="store.load()"
      />
      <template v-else-if="store.view">
        <DiagnosticsList :entries="challengeEntries(store.view.diagnostics)" />
        <QueueError v-if="queue.mutationFailed" :error="queue.mutationError" />
        <Card>
          <FilterBar
            :bar="challengeBar"
            :rows="all"
            :filter="filter"
            :shown="rows.length"
            :value-label="valueLabel"
            @update:query="setQuery"
            @update:picks="setPicks"
            @reset="reset"
          />
          <ChallengesTable
            v-if="rows.length > 0"
            :rows="rows"
            :queued="[...queued]"
            :can-write="canWrite"
            :busy="queue.busy"
            @add="queue.add"
            @navigate="navigate"
          />
          <ListEmptyState v-else :empty="empty" @reset="reset" />
        </Card>
      </template>
      <ScreenSkeleton v-else />
    </div>
  </div>
</template>
