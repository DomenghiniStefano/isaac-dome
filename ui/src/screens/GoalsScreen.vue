<script setup lang="ts">
import { vScrollMemory } from '@/directives/scrollMemory'
import { computed, watch } from 'vue'
import { useRoute } from 'vue-router'
import DiagnosticsList from '@/components/diagnostics/DiagnosticsList.vue'
import QueueError from '@/components/plan/QueueError.vue'
import { Button, ButtonVariant } from '@/components/ui/button'
import { Skeleton } from '@/components/ui/skeleton'
import { useOnActiveProfile } from '@/composables/useOnActiveProfile'
import { useWant } from '@/composables/useWant'
import { useMessages } from '@/i18n'
import { planEntries } from '@/lib/diagnostics/plan'
import { wantBanner, wantBlocks } from '@/lib/graph/wantBlocks'
import { wantLocation, wantOf } from '@/lib/graph/wantLocation'
import type { Target } from '@/lib/ipc/types'
import { queuedIds } from '@/lib/plan/queueRows'
import { RouteName } from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'
import { LoadStatus } from '@/stores/loadStatus'
import { useQueueStore } from '@/stores/queue'
import { useTabsStore } from '@/stores/tabs'
import { useGraphStore } from '@/stores/views'
import AddPane from './goals/AddPane.vue'
import GoalsHero from './goals/GoalsHero.vue'
import QueueCard from './plan/QueueCard.vue'
import ProfileError from './profile/ProfileError.vue'

const graph = useGraphStore()
const queue = useQueueStore()
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

// A queue that couldn't be read or saved offers nothing: the rows still show, without the
// "in the queue" line or the button.
const queued = computed(() => queuedIds(queue.view))
const canWrite = computed(() => queue.view?.storeAvailable === true)

// The suggestions leave out what the queue holds, and Rust decides which ones fill the
// place (`next_steps`). So when what the queue holds changes — here, from another screen, or
// from another window through `plan-changed` — the suggestions are asked again. A reorder
// changes nothing they depend on, and a queue arriving with the profile arrives with the
// graph beside it: neither asks.
const queuedKey = computed(() =>
  queue.view === null
    ? null
    : [...queued.value].sort((a, b) => a - b).join(','),
)
watch(queuedKey, (now, before) => {
  if (now !== null && before !== null && now !== before) void graph.refresh()
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
const nodes = computed(() => graph.view?.unlock.nodes ?? [])
</script>

<template>
  <!-- The gutter is the children's, so the band can be the full width of the page without
       overflowing it (spec §4.2). The screen does not scroll as one block: the band stays, and
       the two panes take the height left. -->
  <div class="flex h-full min-h-0 flex-col overflow-hidden">
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
    <div
      v-else-if="queue.view"
      class="flex min-h-0 flex-1 flex-col gap-3 px-5.5 pt-4 pb-5"
    >
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

           Stacked, the two panes are one column and the column scrolls. Side by side they
           are two columns of different lengths, and one scrollbar for both would scroll the
           queue out of sight to reach the bottom of the recommendations — which is the one
           thing the queue's fixed place exists to prevent. So above `wide` the row holds the
           height and each pane scrolls inside itself. Whichever box scrolls at the right-hand
           edge carries the gutter inside it, so its scrollbar sits on the window's edge: the
           column when stacked, and side by side **the suggestions**, which is where the gutter
           went when the two changed places (card #63). -->
      <div
        v-scroll-memory="'body'"
        class="-mx-5.5 flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto px-5.5 @wide/page:flex-row @wide/page:items-stretch @wide/page:overflow-hidden @wide/page:pr-0"
      >
        <div
          v-if="readable"
          v-scroll-memory="'queue'"
          class="min-w-0 flex-1 @wide/page:min-h-0 @wide/page:overflow-y-auto"
        >
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
          v-scroll-memory="'suggestions'"
          class="@wide/page:min-h-0 @wide/page:w-add-pane @wide/page:shrink-0 @wide/page:overflow-y-auto @wide/page:pr-5.5"
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
    <div v-else class="flex flex-col gap-4 px-5.5 pt-4">
      <Skeleton class="h-8 w-120" />
      <Skeleton class="h-40 w-full" />
    </div>
  </div>
</template>
