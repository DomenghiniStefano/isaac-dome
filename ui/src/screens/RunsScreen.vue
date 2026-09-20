<script setup lang="ts">
import { PlayIcon } from '@lucide/vue'
import { computed } from 'vue'
import DiagnosticsList from '@/components/diagnostics/DiagnosticsList.vue'
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import FilterBar from '@/components/facets/FilterBar.vue'
import { Button, ButtonVariant } from '@/components/ui/button'
import KpiTile from '@/components/kpi/KpiTile.vue'
import { Card } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { useMessages } from '@/i18n'
import { useTabView } from '@/composables/useTabView'
import type { ScrollOffset } from '@/lib/scale/scrollOffset'
import { emptyFilter } from '@/lib/facets/faceting'
import type { FacetFilter } from '@/lib/facets/faceting'
import type { RunView } from '@/lib/ipc/types'
import { runsEntries } from '@/lib/diagnostics/runs'
import { orderRuns } from '@/lib/runs/runOrder'
import { runKey } from '@/lib/runs/runKey'
import { RunFacet, outcomeOrder, runFaceting } from '@/lib/runs/runFacets'
import {
  barLabels,
  facetTitle,
  facetValueLabel,
  outcomeDot,
  outcomeTextByKind,
  runSlots,
} from '@/lib/runs/runLabels'
import { LoadStatus } from '@/stores/loadStatus'
import { useRunsStore } from '@/stores/views'
import ProfileError from './profile/ProfileError.vue'
import RunDetail from './runs/RunDetail.vue'
import RunsTable from './runs/RunsTable.vue'
import { runsView } from './runs/tabView'
import ScreenHeader from './ScreenHeader.vue'

const store = useRunsStore()
const { t } = useMessages()

// The archive is not a view of the profile: it exists without one, and it is what the app
// read while nobody was watching. So it loads on mount and not on a profile becoming active.
void store.load()

const facetOrder = Object.values(RunFacet)
// The filter and the selection belong to the tab, not to this component: leaving and coming
// back — through a tear-off, a restart, or the back button — finds them where they were left
// (B39).
const reading = useTabView(runsView)
const setOffset = (offset: ScrollOffset) => {
  reading.value = { ...reading.value, offset }
}
const filter = computed({
  get: () => reading.value.filter,
  set: (value: FacetFilter<RunFacet>) => {
    reading.value = { ...reading.value, filter: value }
  },
})

const all = computed(() => store.view?.runs ?? [])
const rows = computed(() =>
  orderRuns(all.value.filter((run) => runFaceting.matches(run, filter.value))),
)
// The selected run travels as its key, never as the row: a row is the archive's answer of the
// moment, and a stored one would come back describing a run the fold has since re-derived. A
// key that matches nothing — the run was filtered away, or the archive grew — is simply no
// selection, which is what the screen already draws.
const selectedKey = computed({
  get: () => reading.value.selected,
  set: (value: string | null) => {
    reading.value = { ...reading.value, selected: value }
  },
})
const selected = computed(
  () => rows.value.find((run) => runKey(run) === selectedKey.value) ?? null,
)
const totals = computed(() => store.view?.totals ?? null)

const valueLabel = (facet: RunFacet, value: string) =>
  facetValueLabel(t, facet, value)

const setPicks = (facet: RunFacet, picked: string[]) => {
  filter.value = {
    ...filter.value,
    picks: { ...filter.value.picks, [facet]: picked },
  }
}
const setQuery = (query: string) => {
  filter.value = { ...filter.value, query }
}
const reset = () => {
  filter.value = emptyFilter<RunFacet>(facetOrder)
}
const select = (run: RunView) => {
  selectedKey.value = runKey(run)
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-col gap-4 overflow-hidden pt-5 pb-5">
    <ScreenHeader :icon="PlayIcon" :title="t('routes.runs')">{{
      t('runs.intro')
    }}</ScreenHeader>
    <ProfileError
      v-if="store.status === LoadStatus.Failed"
      :error="store.error"
      @retry="store.load()"
    />
    <template v-else-if="store.view">
      <!-- The archive says what it could not read before it says what it holds: a short list
           with an unread source behind it must never read as "you have played nothing". -->
      <DiagnosticsList :entries="runsEntries(store.view.diagnostics)" />
      <div v-if="totals !== null" class="grid grid-cols-2 gap-3 @regular/page:grid-cols-4">
        <KpiTile :value="totals.runs" :label="t('runs.totals.runs')" />
        <KpiTile
          :value="totals.won"
          :denominator="totals.runs"
          :label="t('runs.totals.won')"
        />
        <KpiTile
          :value="totals.died"
          :denominator="totals.runs"
          :label="t('runs.totals.died')"
        />
        <KpiTile
          :value="totals.abandoned"
          :denominator="totals.runs"
          :label="t('runs.totals.abandoned')"
        />
      </div>
      <Card class="min-h-0 flex-1">
        <FilterBar
          :shown="rows.length"
          :total="all.length"
          :query="filter.query"
          :rows="all"
          :faceting="runFaceting"
          :filter="filter"
          :facets="runSlots"
          :state="{
            facet: RunFacet.Outcome,
            order: outcomeOrder,
            dot: outcomeDot,
            text: outcomeTextByKind,
          }"
          :title="facetTitle"
          :value-label="valueLabel"
          :labels="barLabels"
          @update:query="setQuery"
          @update:picks="setPicks"
          @reset="reset"
        />
        <RunsTable
          v-if="rows.length > 0"
          :runs="rows"
          :selected="selected"
          :offset="reading.offset"
          @offset-change="setOffset"
          @select="select"
        />
        <div v-else class="flex flex-col items-start gap-3 p-4">
          <EmptyCategory>{{
            all.length === 0 ? t('runs.empty') : t('runs.noMatch')
          }}</EmptyCategory>
          <Button
            v-if="all.length > 0"
            :variant="ButtonVariant.Outline"
            @click="reset"
            >{{ t('filters.reset') }}</Button
          >
        </div>
      </Card>
      <RunDetail v-if="selected !== null" :run="selected" />
    </template>
    <div v-else class="flex flex-col gap-4">
      <Skeleton class="h-8 w-120" />
      <Skeleton class="h-12 w-full" />
      <Skeleton class="h-150 w-full" />
    </div>
  </div>
</template>
