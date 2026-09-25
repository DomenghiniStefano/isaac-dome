<script setup lang="ts">
import { FlagIcon } from '@lucide/vue'
import { computed } from 'vue'
import DiagnosticsList from '@/components/diagnostics/DiagnosticsList.vue'
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import FilterBar from '@/components/facets/FilterBar.vue'
import QueueError from '@/components/plan/QueueError.vue'
import { Button, ButtonVariant } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { useOnActiveProfile } from '@/composables/useOnActiveProfile'
import { useTabView } from '@/composables/useTabView'
import { vScrollMemory } from '@/directives/scrollMemory'
import { useMessages } from '@/i18n'
import {
  ChallengeFacet,
  challengeFaceting,
} from '@/lib/challenges/challengeFacets'
import { challengeEntries } from '@/lib/diagnostics/challenges'
import { emptyList, isFiltering } from '@/lib/facets/emptyList'
import type { FacetFilter } from '@/lib/facets/faceting'
import type { Target } from '@/lib/ipc/types'

import { pageLocation } from '@/lib/wiki/category'
import { LoadStatus } from '@/stores/loadStatus'
import { useQueueOffer } from '@/composables/useQueueOffer'
import { useTabsStore } from '@/stores/tabs'
import { useChallengesStore } from '@/stores/views'
import ScreenHeader from './ScreenHeader.vue'
import ChallengesTable from './challenges/ChallengesTable.vue'
import {
  barLabels,
  challengeFacetTitle,
  challengeFacetValueLabel,
  challengeSlots,
  challengeStateDot,
  challengeStateOrder,
  challengeStateText,
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

// The filter belongs to the tab, not to this component: leaving and coming back — through a
// tear-off, a restart, or the back button — finds it where it was left (B39).
const reading = useTabView(challengesView)
const filter = computed({
  get: () => reading.value.filter,
  set: (value: FacetFilter<ChallengeFacet>) => {
    reading.value = { ...reading.value, filter: value }
  },
})

const all = computed(() => store.view?.challenges ?? [])
const rows = computed(() =>
  all.value.filter((row) => challengeFaceting.matches(row, filter.value)),
)

// The character facet stores the wiki's id; the names are on the rows, so the screen is the
// only place that can label one — the same shape Unlock has for its own character facet (B28).
const characterNames = computed(
  () =>
    new Map(
      all.value.flatMap((row) =>
        row.character !== null &&
        row.character.kind === 'character' &&
        row.characterName !== null
          ? [[String(row.character.id), row.characterName]]
          : [],
      ),
    ),
)
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

const setPicks = (facet: ChallengeFacet, picked: string[]) => {
  filter.value = {
    ...filter.value,
    picks: { ...filter.value.picks, [facet]: picked },
  }
}
const setQuery = (query: string) => {
  filter.value = { ...filter.value, query }
}
const reset = () => {
  filter.value = challengeFaceting.empty()
}
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
            :shown="rows.length"
            :total="all.length"
            :query="filter.query"
            :rows="all"
            :faceting="challengeFaceting"
            :filter="filter"
            :facets="challengeSlots"
            :state="{
              facet: ChallengeFacet.State,
              order: challengeStateOrder,
              dot: challengeStateDot,
              text: challengeStateText,
            }"
            :title="challengeFacetTitle"
            :value-label="valueLabel"
            :labels="barLabels"
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
  </div>
</template>
