<script setup lang="ts">
import { InfoIcon, ListChecksIcon } from '@lucide/vue'
import { computed } from 'vue'
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import QueueError from '@/components/plan/QueueError.vue'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Skeleton } from '@/components/ui/skeleton'
import { useOnActiveProfile } from '@/composables/useOnActiveProfile'
import { useMessages } from '@/i18n'
import { nodeSlot } from '@/lib/graph/unlockFilter'
import { canQueue, isQueued, queuedIds } from '@/lib/plan/queueRows'
import { useGraphStore } from '@/stores/graph'
import { LoadStatus } from '@/stores/profile'
import { useQueueStore } from '@/stores/queue'
import ScreenHeader from './ScreenHeader.vue'
import StepCard from './nextSteps/StepCard.vue'
import ProfileError from './profile/ProfileError.vue'

const graph = useGraphStore()
const queue = useQueueStore()
const { t } = useMessages()

useOnActiveProfile(async () => {
  await Promise.all([graph.load(), queue.load()])
})

// An empty list has two reasons, and the steps don't say which: the unlock view's diagnostics
// do (DESIGN-BRIEF.md §7.3).
const noCatalog = computed(
  () => graph.unlock?.diagnostics.some((d) => d.kind === 'noCatalog') ?? false,
)

// A queue that couldn't be read or saved offers nothing: the steps still show, without the
// "in the queue" line or the button.
const queued = computed(() => queuedIds(queue.view))
const canWrite = computed(() => queue.view?.storeAvailable === true)
</script>

<template>
  <div class="flex max-w-190 flex-col gap-4">
    <ScreenHeader :icon="ListChecksIcon" :title="t('routes.nextSteps')">{{
      t('nextSteps.intro')
    }}</ScreenHeader>
    <ProfileError
      v-if="graph.status === LoadStatus.Failed"
      :error="graph.error"
      @retry="graph.load()"
    />
    <template v-else-if="graph.steps && graph.unlock">
      <QueueError v-if="queue.mutationFailed" :error="queue.mutationError" />
      <div v-if="graph.steps.steps.length > 0" class="flex flex-col gap-2">
        <StepCard
          v-for="(step, index) in graph.steps.steps"
          :key="nodeSlot(step)"
          :rank="index + 1"
          :node="step"
          :queued="isQueued(step, queued)"
          :can-add="canWrite && canQueue(step, queued)"
          :busy="queue.busy"
          @add="queue.add(nodeSlot(step))"
        />
      </div>
      <Alert v-else-if="noCatalog">
        <InfoIcon />
        <AlertTitle>{{ t('nextSteps.noCatalogTitle') }}</AlertTitle>
        <AlertDescription>{{ t('nextSteps.noCatalog') }}</AlertDescription>
      </Alert>
      <EmptyCategory v-else>{{ t('nextSteps.nothingNow') }}</EmptyCategory>
    </template>
    <div v-else class="flex flex-col gap-2">
      <Skeleton class="h-28 w-full" />
      <Skeleton class="h-28 w-full" />
      <Skeleton class="h-28 w-full" />
    </div>
  </div>
</template>
