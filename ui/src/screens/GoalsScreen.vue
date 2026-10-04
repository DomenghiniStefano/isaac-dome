<script setup lang="ts">
import { PageScroll } from '@/components/ui/virtual'
import { computed, watch } from 'vue'
import { useRoute } from 'vue-router'
import DiagnosticsList from '@/components/diagnostics/DiagnosticsList.vue'
import QueueError from '@/components/plan/QueueError.vue'
import { Button, ButtonVariant } from '@/components/ui/button'
import ScreenSkeleton from '@/components/data-state/ScreenSkeleton.vue'
import { SkeletonBlock } from '@/components/data-state/skeletonBlock'
import { useOnActiveProfile } from '@/composables/useOnActiveProfile'
import { useWant } from '@/composables/useWant'
import { useMessages } from '@/i18n'
import { planEntries } from '@/lib/diagnostics/plan'
import { wantBanner, wantBlocks } from '@/lib/graph/wantBlocks'
import { wantLocation, wantOf } from '@/lib/graph/wantLocation'
import type { Target } from '@/lib/ipc/types'

import {
  membershipChanged,
  queueMembership,
  queueReadable,
} from '@/lib/plan/queueView'
import { RouteName } from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'
import { LoadStatus } from '@/stores/loadStatus'
import { useQueueOffer } from '@/composables/useQueueOffer'
import { useTabsStore } from '@/stores/tabs'
import { useGraphStore } from '@/stores/views'
import AddPane from './goals/AddPane.vue'
import GoalsHero from './goals/GoalsHero.vue'
import QueueCard from './goals/QueueCard.vue'
import ProfileError from '@/components/data-state/ProfileError.vue'

const graph = useGraphStore()
const { queue, queued, canWrite } = useQueueOffer()
const tabs = useTabsStore()
const route = useRoute()
const { t } = useMessages()

// The queue and the graph together: the pane beside the queue is the recommendations, and
// the texts of rows that left the queue come from the Unlock view.
useOnActiveProfile(async () => {
  await Promise.all([graph.load(), queue.load()])
})

// B37: the want lives in the URL, so back, forward and tab restore all reach it, and a link
// from anywhere can ask the question without this screen knowing who called.
const target = computed(() => wantOf(route.query as TabLocation['query']))
// Destructured on purpose: a template unwraps refs that are setup bindings, not refs sitting
// inside an object, and `asked.view` there would be the ref itself.
const {
  view: wantView,
  status: wantStatus,
  error: wantError,
  load: reloadWant,
} = useWant(target)
const blocks = computed(() =>
  wantView.value === null ? [] : wantBlocks(wantView.value, queued.value),
)
const banner = computed(() =>
  wantView.value === null ? null : wantBanner(wantView.value),
)
const ask = (wanted: Target) => {
  const location = wantLocation(wanted)
  if (location !== null) tabs.navigate(location)
}
const stopAsking = () => tabs.navigate({ name: RouteName.Goals })

// An empty pane has two reasons, and the sections don't say which: the unlock view's
// diagnostics do (DESIGN-BRIEF.md §7.3). Read once here and handed down, so nothing computes
// "is the game installed" a second time.
const noCatalog = computed(
  () =>
    graph.view?.unlock.diagnostics.some((d) => d.kind === 'noCatalog') ?? false,
)

// The suggestions leave out what the queue holds, and Rust decides which ones fill the
// place (`next_steps`). So when what the queue holds changes — here, from another screen, or
// from another window through `plan-changed` — the suggestions are asked again. A reorder
// changes nothing they depend on, and a queue arriving with the profile arrives with the
// graph beside it: neither asks.
watch(
  () => queueMembership(queue.view),
  (now, before) => {
    if (membershipChanged(now, before)) void graph.refresh()
  },
)

const readable = computed(() => queueReadable(queue.view))
const nodes = computed(() => graph.view?.unlock.nodes ?? [])
</script>

<template>
  <!-- The screen scrolls as a page (`PageScroll`): the band and the alerts go by, and the queue
       and what could be added scroll with them. The gutter is the children's, so the band can be
       the full width of the page without overflowing it (spec §4.2). -->
  <PageScroll>
    <GoalsHero />
    <ProfileError
      v-if="queue.status === LoadStatus.Failed"
      :error="queue.error"
      class="mx-5.5 mt-4"
      @retry="queue.load()"
    />
    <ProfileError
      v-else-if="wantStatus === LoadStatus.Failed"
      :error="wantError"
      class="mx-5.5 mt-4"
      @retry="reloadWant()"
    />
    <div v-else-if="queue.view" class="flex flex-col gap-3 px-5.5 pt-4 pb-5">
      <DiagnosticsList :entries="planEntries(queue.view.diagnostics)">
        <template #action>
          <Button
            :variant="ButtonVariant.Outline"
            :disabled="queue.busy"
            @click="queue.importGoals()"
            >{{ t('plan.alerts.import') }}</Button
          >
        </template>
      </DiagnosticsList>
      <QueueError v-if="queue.mutationFailed" :error="queue.mutationError" />
      <!-- **The queue comes first, and the suggestions follow it** — stacked, the queue is the
           top of the column and you scroll past it to what you could add; side by side, the
           queue is the left-hand pane. The screen answers "what am I doing" before it answers
           "what else could I do", and the second question is only worth reading once the first
           has been.

           Stacked, the two panes are one column; side by side, one row. Either way the page is
           what scrolls, and side by side the two columns scroll together — the owner's choice of
           one shape for every screen, over the fixed queue that two scrollbars used to keep. -->
      <div
        class="flex flex-col gap-4 @wide/page:flex-row @wide/page:items-stretch"
      >
        <div v-if="readable" class="min-w-0 flex-1">
          <!-- Both panes fill the row: two panels of the same height read as one workbench,
               where one tall and one short read as a panel and a leftover. -->
          <QueueCard
            class="h-full"
            :rows="queue.view.rows"
            :diagnostics="queue.view.diagnostics"
            :nodes="nodes"
            :busy="queue.busy"
            :last-move="queue.lastMove"
            @move="queue.move"
            @remove="queue.remove"
          />
        </div>
        <AddPane
          class="@wide/page:w-add-pane @wide/page:shrink-0"
          :sections="graph.view?.steps.sections ?? []"
          :queued="queued"
          :can-write="canWrite"
          :busy="queue.busy"
          :want-active="target !== null"
          :no-catalog="noCatalog"
          :blocks="blocks"
          :banner="banner"
          @add="queue.add($event)"
          @pick="ask"
          @clear="stopAsking"
        />
      </div>
    </div>
    <ScreenSkeleton v-else class="px-5.5 pt-4" :blocks="[SkeletonBlock.Card]" />
  </PageScroll>
</template>
