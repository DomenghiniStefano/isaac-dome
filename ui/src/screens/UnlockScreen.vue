<script setup lang="ts">
import { LockOpenIcon } from '@lucide/vue'
import { computed, ref } from 'vue'
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import { Button, ButtonVariant } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { useOnActiveProfile } from '@/composables/useOnActiveProfile'
import { useMessages } from '@/i18n'
import { stateCounts } from '@/lib/graph/nodeState'
import {
  FacetId,
  UnlockSort,
  emptyFilter,
  matchesFilter,
  sortNodes,
} from '@/lib/graph/unlockFilter'
import type { UnlockFilter } from '@/lib/graph/unlockFilter'
import { useGraphStore } from '@/stores/graph'
import { LoadStatus } from '@/stores/profile'
import ScreenHeader from './ScreenHeader.vue'
import ProfileError from './profile/ProfileError.vue'
import FacetDrawer from './unlock/FacetDrawer.vue'
import StateToggle from './unlock/StateToggle.vue'
import UnlockDiagnostics from './unlock/UnlockDiagnostics.vue'
import UnlockTable from './unlock/UnlockTable.vue'
import UnlockToolbar from './unlock/UnlockToolbar.vue'

const graph = useGraphStore()
const { t } = useMessages()

useOnActiveProfile(() => graph.load())

// The filter belongs to this screen: leaving the tab resets it, until tabs keep their state.
const filter = ref<UnlockFilter>(emptyFilter())
const sort = ref<UnlockSort>(UnlockSort.FanOut)

const nodes = computed(() => graph.unlock?.nodes ?? [])
const counts = computed(() => stateCounts(nodes.value))
const rows = computed(() =>
  sortNodes(
    nodes.value.filter((node) => matchesFilter(node, filter.value)),
    sort.value,
  ),
)

const setPicks = (facet: FacetId, picked: string[]) => {
  filter.value = {
    ...filter.value,
    picks: { ...filter.value.picks, [facet]: picked },
  }
}
const toggle = (facet: FacetId, value: string) => {
  const picked = filter.value.picks[facet]
  setPicks(
    facet,
    picked.includes(value)
      ? picked.filter((v) => v !== value)
      : [...picked, value],
  )
}
const setQuery = (query: string) => {
  filter.value = { ...filter.value, query }
}
const setSort = (next: UnlockSort) => {
  sort.value = next
}
const reset = () => {
  filter.value = emptyFilter()
}
</script>

<template>
  <div class="flex max-w-250 flex-col gap-4">
    <ScreenHeader :icon="LockOpenIcon" :title="t('routes.unlock')">{{
      t('unlock.intro')
    }}</ScreenHeader>
    <ProfileError
      v-if="graph.status === LoadStatus.Failed"
      :error="graph.error"
      @retry="graph.load()"
    />
    <template v-else-if="graph.unlock">
      <UnlockDiagnostics :diagnostics="graph.unlock.diagnostics" />
      <StateToggle
        :counts="counts"
        :picked="filter.picks[FacetId.State]"
        @update="setPicks(FacetId.State, $event)"
      />
      <FacetDrawer
        :nodes="nodes"
        :filter="filter"
        @toggle="toggle"
        @reset="reset"
      />
      <Card>
        <UnlockToolbar
          :shown="rows.length"
          :total="nodes.length"
          :filter="filter"
          :query="filter.query"
          :sort="sort"
          @update:query="setQuery"
          @update:sort="setSort"
          @toggle="toggle"
        />
        <UnlockTable v-if="rows.length > 0" :nodes="rows" />
        <div v-else class="flex flex-col items-start gap-3 p-4">
          <EmptyCategory>{{ t('unlock.noResults') }}</EmptyCategory>
          <Button :variant="ButtonVariant.Outline" @click="reset">{{
            t('unlock.resetFilters')
          }}</Button>
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
