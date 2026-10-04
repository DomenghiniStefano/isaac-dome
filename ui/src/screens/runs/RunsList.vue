<script setup lang="ts">
import { PlayIcon } from '@lucide/vue'
import { computed } from 'vue'
import ListEmptyState from '@/components/data-state/ListEmptyState.vue'
import ScreenSkeleton from '@/components/data-state/ScreenSkeleton.vue'
import ProfileError from '@/components/data-state/ProfileError.vue'
import DiagnosticsList from '@/components/diagnostics/DiagnosticsList.vue'
import FilterBar from '@/components/facets/FilterBar.vue'
import KpiTile from '@/components/kpi/KpiTile.vue'
import ScreenHeader from '@/components/screen/ScreenHeader.vue'
import { Card } from '@/components/ui/card'
import { PageScroll } from '@/components/ui/virtual'
import { useFacetedReading } from '@/composables/useFacetedReading'
import { useMessages } from '@/i18n'
import { runsEntries } from '@/lib/diagnostics/runs'
import { emptyList, isFiltering } from '@/lib/facets/emptyList'
import type { RunView } from '@/lib/ipc/types'
import { runFaceting } from '@/lib/runs/runFacets'
import type { RunFacet } from '@/lib/runs/runFacets'
import { runFacetValueLabel, runsBar } from '@/lib/runs/runLabels'
import { runLocation } from '@/lib/runs/runLocation'
import { orderRuns } from '@/lib/runs/runOrder'
import { LoadStatus } from '@/stores/loadStatus'
import { useTabsStore } from '@/stores/tabs'
import { useRunsStore } from '@/stores/views'
import RunsTable from './RunsTable.vue'
import { runsView } from './tabView'

// The run diary: what was played and how it ended. A run opens on a page of its own — in this
// tab, or with Ctrl in a new one, as every link in the app does.
const store = useRunsStore()
const tabs = useTabsStore()
const { t } = useMessages()

const { filter, setPicks, setQuery, reset } = useFacetedReading(
  runsView,
  runFaceting.empty,
)

const all = computed(() => store.view?.runs ?? [])
const rows = computed(() =>
  orderRuns(all.value.filter((run) => runFaceting.matches(run, filter.value))),
)
const totals = computed(() => store.view?.totals ?? null)
const open = (run: RunView, event: MouseEvent) =>
  tabs.go(runLocation(run), event.ctrlKey)

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
  <!-- The screen scrolls as a page (`PageScroll`): the header, the totals and the filters go by
       with the list, and only the table's column header stays pinned. -->
  <PageScroll>
    <div class="flex flex-col gap-4 px-5.5 pt-5 pb-5">
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
        <Card>
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
          <RunsTable v-if="rows.length > 0" :runs="rows" @open="open" />
          <ListEmptyState v-else :empty="empty" @reset="reset" />
        </Card>
      </template>
      <ScreenSkeleton v-else />
    </div>
  </PageScroll>
</template>
