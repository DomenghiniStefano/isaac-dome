<script setup lang="ts">
import { PlayIcon } from '@lucide/vue'
import { computed, ref } from 'vue'
import DiagnosticsList from '@/components/diagnostics/DiagnosticsList.vue'
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import FacetDrawer from '@/components/facets/FacetDrawer.vue'
import FilterToolbar from '@/components/facets/FilterToolbar.vue'
import type { DrawerLabels, ToolbarLabels } from '@/components/facets/labels'
import { Button, ButtonVariant } from '@/components/ui/button'
import KpiTile from '@/components/kpi/KpiTile.vue'
import { Card } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { useMessages } from '@/i18n'
import { emptyFilter } from '@/lib/facets/faceting'
import type { FacetFilter } from '@/lib/facets/faceting'
import type { RunView } from '@/lib/ipc/types'
import { runsEntries } from '@/lib/diagnostics/runs'
import { orderRuns } from '@/lib/runs/runOrder'
import { RunFacet, runFaceting } from '@/lib/runs/runFacets'
import { facetTitle, facetValueLabel } from '@/lib/runs/runLabels'
import { LoadStatus } from '@/stores/loadStatus'
import { useRunsStore } from '@/stores/views'
import ProfileError from './profile/ProfileError.vue'
import RunDetail from './runs/RunDetail.vue'
import RunsTable from './runs/RunsTable.vue'
import ScreenHeader from './ScreenHeader.vue'

const store = useRunsStore()
const { t } = useMessages()

// The archive is not a view of the profile: it exists without one, and it is what the app
// read while nobody was watching. So it loads on mount and not on a profile becoming active.
void store.load()

const facetOrder = Object.values(RunFacet)
const filter = ref<FacetFilter<RunFacet>>(emptyFilter<RunFacet>(facetOrder))
const selected = ref<RunView | null>(null)

const all = computed(() => store.view?.runs ?? [])
const rows = computed(() =>
  orderRuns(all.value.filter((run) => runFaceting.matches(run, filter.value))),
)
const totals = computed(() => store.view?.totals ?? null)

const valueLabel = (facet: RunFacet, value: string) =>
  facetValueLabel(t, facet, value)

const toggle = (facet: RunFacet, value: string) => {
  const picked = filter.value.picks[facet]
  filter.value = {
    ...filter.value,
    picks: {
      ...filter.value.picks,
      [facet]: picked.includes(value)
        ? picked.filter((v) => v !== value)
        : [...picked, value],
    },
  }
}
const setQuery = (query: string) => {
  filter.value = { ...filter.value, query }
}
const reset = () => {
  filter.value = emptyFilter<RunFacet>(facetOrder)
}
const select = (run: RunView) => {
  selected.value = run
}

const toolbarLabels: ToolbarLabels = {
  rows: 'runs.rows',
  search: 'runs.search',
  // No sort group on this list: the archive decides the order (runOrder.ts). The key is
  // required by the type and unused by the template, which is why the group is optional.
  sortBy: 'runs.column.source',
  activeFilters: 'runs.activeFilters',
}
const drawerLabels: DrawerLabels = {
  facets: 'runs.facets',
  activeFilters: 'runs.activeFilters',
  noFilters: 'runs.noFilters',
  reset: 'runs.reset',
}
</script>

<template>
  <div class="flex max-w-250 flex-col gap-4">
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
      <div v-if="totals !== null" class="grid grid-cols-2 gap-3 sm:grid-cols-4">
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
      <FacetDrawer
        :rows="all"
        :faceting="runFaceting"
        :facets="facetOrder"
        :filter="filter"
        :title="facetTitle"
        :value-label="valueLabel"
        :labels="drawerLabels"
        @toggle="toggle"
        @reset="reset"
      />
      <Card>
        <FilterToolbar
          :shown="rows.length"
          :total="all.length"
          :query="filter.query"
          :order="facetOrder"
          :picks="filter.picks"
          :value-label="valueLabel"
          :labels="toolbarLabels"
          @update:query="setQuery"
          @toggle="toggle"
        />
        <RunsTable
          v-if="rows.length > 0"
          :runs="rows"
          :selected="selected"
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
            >{{ t('runs.reset') }}</Button
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
