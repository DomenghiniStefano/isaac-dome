<script setup lang="ts">
import { MapIcon } from '@lucide/vue'
import { computed } from 'vue'
import QueueError from '@/components/plan/QueueError.vue'
import { Skeleton } from '@/components/ui/skeleton'
import { useOnActiveProfile } from '@/composables/useOnActiveProfile'
import { useMessages } from '@/i18n'
import { queueSummary, queuedIds } from '@/lib/plan/queueRows'
import { useGraphStore } from '@/stores/graph'
import { LoadStatus } from '@/stores/profile'
import { useQueueStore } from '@/stores/queue'
import ScreenHeader from './ScreenHeader.vue'
import PlanAlerts from './plan/PlanAlerts.vue'
import ProposalAside from './plan/ProposalAside.vue'
import QueueCard from './plan/QueueCard.vue'
import ProfileError from './profile/ProfileError.vue'

const queue = useQueueStore()
const graph = useGraphStore()
const { t } = useMessages()

// The queue and the graph together: the proposal beside the queue is Next steps, and the
// texts of rows that left the queue come from the Unlock view.
useOnActiveProfile(async () => {
  await Promise.all([queue.load(), graph.load()])
})

// The queue card needs a queue that could be read: no database, an unreadable document and no
// catalog each say so in an alert instead of an empty list.
const readable = computed((): boolean => {
  const view = queue.view
  return (
    view !== null &&
    view.storeAvailable &&
    !view.diagnostics.some(
      (d) => d.kind === 'unreadable' || d.kind === 'noCatalog',
    )
  )
})
const summary = computed(() => queueSummary(queue.view?.rows ?? []))
const queued = computed(() => queuedIds(queue.view))
const nodes = computed(() => graph.unlock?.nodes ?? [])
</script>

<template>
  <div class="flex max-w-300 flex-col gap-4">
    <ScreenHeader :icon="MapIcon" :title="t('routes.plan')">{{
      t('plan.intro')
    }}</ScreenHeader>
    <ProfileError
      v-if="queue.status === LoadStatus.Failed"
      :error="queue.error"
      @retry="queue.load()"
    />
    <template v-else-if="queue.view">
      <p v-if="readable" class="text-caption text-foreground-soft tabular-nums">
        {{ t('plan.summary.rows') }}: {{ summary.rows }} ·
        {{ t('plan.summary.wanted') }}: {{ summary.wanted }} ·
        {{ t('plan.summary.pulledIn') }}: {{ summary.pulledIn }}
      </p>
      <PlanAlerts
        :diagnostics="queue.view.diagnostics"
        :busy="queue.busy"
        @import-goals="queue.importGoals()"
      />
      <QueueError v-if="queue.mutationFailed" :error="queue.mutationError" />
      <div v-if="readable" class="flex flex-col items-start gap-4 lg:flex-row">
        <QueueCard
          class="w-full min-w-0 flex-1"
          :rows="queue.view.rows"
          :diagnostics="queue.view.diagnostics"
          :nodes="nodes"
          :busy="queue.busy"
          :last-move="queue.lastMove"
          @move="queue.move"
          @remove="queue.remove"
        />
        <ProposalAside
          class="w-full lg:w-plan-aside lg:shrink-0"
          :steps="graph.steps?.steps ?? []"
          :queued="queued"
          :can-write="queue.view.storeAvailable"
          :busy="queue.busy"
          @add="queue.add"
        />
      </div>
    </template>
    <div v-else class="flex flex-col gap-4">
      <Skeleton class="h-8 w-120" />
      <Skeleton class="h-40 w-full" />
    </div>
  </div>
</template>
