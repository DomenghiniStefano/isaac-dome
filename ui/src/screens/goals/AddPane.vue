<script setup lang="ts">
import { InfoIcon } from '@lucide/vue'
import { computed } from 'vue'
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import GoalRow from '@/components/plan/GoalRow.vue'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Card, CardHeader, CardTitle } from '@/components/ui/card'
import { useMessages } from '@/i18n'
import { NodeState } from '@/lib/graph/nodeState'
import { nodeNumber } from '@/lib/graph/achievementNode'
import type { WantBlock } from '@/lib/graph/wantBlocks'
import type { StepsSection, Target, WantDiagnostic } from '@/lib/ipc/types'
import { AddPaneState, addPaneState } from '@/lib/plan/addPaneState'
import { canQueue, isQueued } from '@/lib/plan/queueRows'
import { rowModel } from '@/lib/plan/rowModel'
import { RouteName } from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'
import { useTabsStore } from '@/stores/tabs'
import WantAnswer from './WantAnswer.vue'
import WantBar from './WantBar.vue'
import { sectionTitle } from './sectionTitle'

const props = defineProps<{
  sections: StepsSection[]
  queued: Set<number>
  canWrite: boolean
  busy: boolean
  /** B37: a want is named, so this pane answers it instead of suggesting. */
  wantActive: boolean
  /** The unlock view's own diagnostic, never recomputed here. */
  noCatalog: boolean
  blocks: WantBlock[]
  banner: WantDiagnostic | null
}>()
const emit = defineEmits<{
  add: [achievement: number]
  pick: [target: Target]
  clear: []
}>()
const { t } = useMessages()
const tabs = useTabsStore()

const state = computed(() =>
  addPaneState(props.wantActive, props.sections, props.noCatalog),
)

// This pane recommends; Unlock is the exhaustive list. It ends on the way there with the
// same filter already picked, so the two stop competing.
const seeAll: TabLocation = {
  name: RouteName.Unlock,
  query: { state: NodeState.Now },
}
</script>

<template>
  <!-- Where rows come from: either the app proposes, or you name what you are after. The
       queue beside it never changes content, so you can see what you decided while you
       decide what to add to it (spec §2, §3). -->
  <Card class="w-full min-w-0 flex-col gap-0 p-0">
    <!-- The same band the queue wears, for the same reason: two panels of one workbench read
         as a tool, where a banded card beside a bare column reads as a card and a leftover. -->
    <CardHeader>
      <CardTitle>{{ t('plan.addPane') }}</CardTitle>
    </CardHeader>
    <div class="flex flex-col gap-3 p-3">
      <WantBar @pick="emit('pick', $event)" @clear="emit('clear')" />

      <WantAnswer
        v-if="state === AddPaneState.Want"
        :blocks="blocks"
        :banner="banner"
        :can-write="canWrite"
        :busy="busy"
        @queue="emit('add', $event)"
      />

      <template v-else-if="state === AddPaneState.Sections">
        <!-- One block per reason. A section that would be empty never arrives: Rust decides
           "absent" once, so there is no heading over nothing to handle here. -->
        <section
          v-for="section in sections"
          :key="section.basis"
          class="flex flex-col gap-1"
        >
          <h2 class="text-label text-subtle-foreground">
            {{ t(sectionTitle[section.basis]) }}
          </h2>
          <!-- A bare list, not a card: the pane is the card now, and a card inside a card is
             two edges saying the same thing. -->
          <div class="-mx-3 border-y border-hairline bg-data">
            <div
              v-for="step in section.steps"
              :key="nodeNumber(step)"
              :class="[
                'border-b border-hairline last:border-b-0',
                isQueued(step, queued) && 'opacity-disabled',
              ]"
            >
              <!-- A queued row is not suggested (Rust leaves it out), so this only lasts the
                 moment between adding it and the suggestions coming back: dimmed, its button
                 refused, so the same click cannot land twice. -->
              <GoalRow
                :model="rowModel(step, t)"
                :node="step"
                :can-add="canWrite && canQueue(step, queued)"
                :busy="busy"
                @add="emit('add', nodeNumber(step))"
              />
            </div>
          </div>
        </section>
        <Button
          :variant="ButtonVariant.Ref"
          :size="ButtonSize.Inline"
          class="self-start"
          @click="tabs.go(seeAll, $event.ctrlKey)"
          >{{ t('goals.seeAll') }}</Button
        >
      </template>

      <Alert v-else-if="state === AddPaneState.NoCatalog">
        <InfoIcon />
        <AlertTitle>{{ t('goals.noCatalogTitle') }}</AlertTitle>
        <AlertDescription>{{ t('goals.noCatalog') }}</AlertDescription>
      </Alert>

      <!-- The fourth member. `v-else` is the exhaustive arm a template can have: every other
         state is named above it, so a fifth added to the enum lands here visibly rather than
         silently. -->
      <EmptyCategory v-else>{{ t('goals.nothingNow') }}</EmptyCategory>
    </div>
  </Card>
</template>
