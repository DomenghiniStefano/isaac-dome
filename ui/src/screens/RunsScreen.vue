<script setup lang="ts">
import { PlayIcon } from '@lucide/vue'
import { computed } from 'vue'
import ListEmptyState from '@/components/data-state/ListEmptyState.vue'
import ScreenSkeleton from '@/components/data-state/ScreenSkeleton.vue'
import DiagnosticsList from '@/components/diagnostics/DiagnosticsList.vue'
import FilterBar from '@/components/facets/FilterBar.vue'
import KpiTile from '@/components/kpi/KpiTile.vue'
import { Card } from '@/components/ui/card'
import { useFacetedReading } from '@/composables/useFacetedReading'
import { useMessages } from '@/i18n'
import { runsEntries } from '@/lib/diagnostics/runs'
import { emptyList, isFiltering } from '@/lib/facets/emptyList'
import type { RunView } from '@/lib/ipc/types'
import { runFaceting } from '@/lib/runs/runFacets'
import type { RunFacet } from '@/lib/runs/runFacets'
import { runKey } from '@/lib/runs/runKey'
import { runFacetValueLabel, runsBar } from '@/lib/runs/runLabels'
import { orderRuns } from '@/lib/runs/runOrder'
import { LoadStatus } from '@/stores/loadStatus'
import { useRunsStore } from '@/stores/views'
import ProfileError from '@/components/data-state/ProfileError.vue'
import RunDetail from './runs/RunDetail.vue'
import RunsTable from './runs/RunsTable.vue'
import { runsView } from './runs/tabView'
import ScreenHeader from '@/components/screen/ScreenHeader.vue'

const store = useRunsStore()
const { t } = useMessages()

// The archive is not a view of the profile: it exists without one, and it is what the app
// read while nobody was watching. So it loads on mount and not on a profile becoming active.
void store.load()

const { reading, update, filter, setPicks, setQuery, reset } =
  useFacetedReading(runsView, runFaceting.empty)

const all = computed(() => store.view?.runs ?? [])
const rows = computed(() =>
  orderRuns(all.value.filter((run) => runFaceting.matches(run, filter.value))),
)
// The selected run travels as its key, never as the row: a row is the archive's answer of the
// moment, and a stored one would come back describing a run the fold has since re-derived. A
// key that matches nothing — the run was filtered away, or the archive grew — is simply no
// selection, which is what the screen already draws.
const selected = computed(
  () =>
    rows.value.find((run) => runKey(run) === reading.value.selected) ?? null,
)
const select = (run: RunView) => update({ selected: runKey(run) })
const totals = computed(() => store.view?.totals ?? null)

// An archive that holds no run is not a filter that matched nothing, as on every other list.
const empty = computed(() =>
  emptyList(all.value.length, isFiltering(filter.value), {
    empty: 'runs.empty',
    noResults: 'runs.noMatch',
  }),
)

const valueLabel = (facet: RunFacet, value: string) =>
  runFacetValueLabel(t, facet, value)
</script>

<template>
  <div
    class="flex h-full min-h-0 flex-col gap-4 overflow-hidden px-5.5 pt-5 pb-5"
  >
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
      <div
        v-if="totals !== null"
        class="grid grid-cols-2 gap-3 @regular/page:grid-cols-4"
      >
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
          :bar="runsBar"
          :rows="all"
          :filter="filter"
          :shown="rows.length"
          :value-label="valueLabel"
          @update:query="setQuery"
          @update:picks="setPicks"
          @reset="reset"
        />
        <RunsTable
          v-if="rows.length > 0"
          :runs="rows"
          :selected="selected"
          :offset="reading.offset"
          @offset-change="update({ offset: $event })"
          @select="select"
        />
        <ListEmptyState v-else :empty="empty" @reset="reset" />
      </Card>
      <RunDetail v-if="selected !== null" :run="selected" />
    </template>
    <ScreenSkeleton v-else />
  </div>
</template>
