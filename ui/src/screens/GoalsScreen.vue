<script setup lang="ts">
import { InfoIcon, ListChecksIcon } from '@lucide/vue'
import { computed } from 'vue'
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import QueueError from '@/components/plan/QueueError.vue'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Skeleton } from '@/components/ui/skeleton'
import { useOnActiveProfile } from '@/composables/useOnActiveProfile'
import { useMessages } from '@/i18n'
import { NodeState } from '@/lib/graph/nodeState'
import { nodeSlot } from '@/lib/graph/unlockFilter'
import { planNow } from '@/lib/plan/planNow'
import { canQueue, isQueued, queuedIds } from '@/lib/plan/queueRows'
import { RouteName } from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'
import { useTabsStore } from '@/stores/tabs'
import { useGraphStore } from '@/stores/views'
import { LoadStatus } from '@/stores/loadStatus'
import { useQueueStore } from '@/stores/queue'
import ScreenHeader from './ScreenHeader.vue'
import GoalCard from './goals/GoalCard.vue'
import { sectionTitle } from './goals/sectionTitle'
import ProfileError from './profile/ProfileError.vue'

const graph = useGraphStore()
const queue = useQueueStore()
const tabs = useTabsStore()
const { t } = useMessages()

// The same cap a section has: this is a reminder, not the queue.
const PLAN_ROWS = 5

useOnActiveProfile(async () => {
  await Promise.all([graph.load(), queue.load()])
})

// An empty page has two reasons, and the sections don't say which: the unlock view's
// diagnostics do (DESIGN-BRIEF.md §7.3).
const noCatalog = computed(
  () =>
    graph.view?.unlock.diagnostics.some((d) => d.kind === 'noCatalog') ?? false,
)

// A queue that couldn't be read or saved offers nothing: the rows still show, without the
// "in the queue" line or the button.
const queued = computed(() => queuedIds(queue.view))
const canWrite = computed(() => queue.view?.storeAvailable === true)

// This page recommends; Unlock is the exhaustive list. Every section ends on the way there,
// with the same filter already picked, so the two stop competing.
const seeAll: TabLocation = {
  name: RouteName.Unlock,
  query: { state: NodeState.Now },
}
const plan: TabLocation = { name: RouteName.Plan }

// At most as many as one section: the landing page reminds, the Plan is where the queue is
// read and moved.
const inPlan = computed(() => planNow(queue.view, PLAN_ROWS))
const open = (location: TabLocation, newTab: boolean) => {
  if (newTab) tabs.open(location)
  else tabs.navigate(location)
}
</script>

<template>
  <div class="flex max-w-190 flex-col gap-4">
    <ScreenHeader :icon="ListChecksIcon" :title="t('routes.goals')">{{
      t('goals.intro')
    }}</ScreenHeader>
    <ProfileError
      v-if="graph.status === LoadStatus.Failed"
      :error="graph.error"
      @retry="graph.load()"
    />
    <template v-else-if="graph.view">
      <QueueError v-if="queue.mutationFailed" :error="queue.mutationError" />
      <template v-if="graph.view.steps.sections.length > 0">
        <!-- One block per reason. A section that would be empty never arrives: Rust decides
             "absent" once, so there is no heading over nothing to handle here. -->
        <section
          v-for="section in graph.view.steps.sections"
          :key="section.basis"
          class="flex flex-col gap-2"
        >
          <h2 class="text-label text-subtle-foreground">
            {{ t(sectionTitle[section.basis]) }}
          </h2>
          <GoalCard
            v-for="step in section.steps"
            :key="nodeSlot(step)"
            :node="step"
            :queued="isQueued(step, queued)"
            :can-add="canWrite && canQueue(step, queued)"
            :busy="queue.busy"
            @add="queue.add(nodeSlot(step))"
            @navigate="open"
          />
          <Button
            :variant="ButtonVariant.Ref"
            :size="ButtonSize.Inline"
            class="self-start"
            @click="open(seeAll, $event.ctrlKey)"
            >{{ t('goals.seeAll') }}</Button
          >
        </section>
      </template>
      <Alert v-else-if="noCatalog">
        <InfoIcon />
        <AlertTitle>{{ t('goals.noCatalogTitle') }}</AlertTitle>
        <AlertDescription>{{ t('goals.noCatalog') }}</AlertDescription>
      </Alert>
      <EmptyCategory v-else>{{ t('goals.nothingNow') }}</EmptyCategory>

      <!-- Outside the chain above on purpose: what *you* had decided is worth showing even
           on a page with nothing to suggest. Absent when the queue holds nothing playable
           now, or could not be read at all (spec §4.3). -->
      <section v-if="inPlan.length > 0" class="flex flex-col gap-2">
        <h2 class="text-label text-subtle-foreground">
          {{ t('goals.inPlan') }}
        </h2>
        <GoalCard
          v-for="row in inPlan"
          :key="nodeSlot(row.node)"
          :node="row.node"
          :queued="true"
          :can-add="false"
          :busy="queue.busy"
          compact
          @navigate="open"
        />
        <Button
          :variant="ButtonVariant.Ref"
          :size="ButtonSize.Inline"
          class="self-start"
          @click="open(plan, $event.ctrlKey)"
          >{{ t('goals.openPlan') }}</Button
        >
      </section>
    </template>
    <div v-else class="flex flex-col gap-2">
      <Skeleton class="h-28 w-full" />
      <Skeleton class="h-28 w-full" />
      <Skeleton class="h-28 w-full" />
    </div>
  </div>
</template>
