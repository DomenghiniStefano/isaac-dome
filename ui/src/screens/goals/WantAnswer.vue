<script setup lang="ts">
import type { Message } from '@/i18n/message'
import { InfoIcon } from '@lucide/vue'
import GoalRow from '@/components/plan/GoalRow.vue'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { useMessages } from '@/i18n'
import { rowModel } from '@/lib/plan/rowModel'
import { WantBlockKind } from '@/lib/graph/wantBlocks'
import type { WantBlock } from '@/lib/graph/wantBlocks'
import { nodeNumber } from '@/lib/graph/achievementNode'
import { WantDiagnostic } from '@/lib/ipc/types'

defineProps<{
  blocks: WantBlock[]
  banner: WantDiagnostic | null
  canWrite: boolean
  busy: boolean
}>()
const emit = defineEmits<{
  queue: [achievement: number]
}>()
const { t } = useMessages()

// A record and not a chain of `v-if`: a fifth kind then fails to compile instead of drawing
// a block with no heading.
const heading: Record<WantBlockKind, Message> = {
  [WantBlockKind.Chain]: 'want.chain',
  [WantBlockKind.AvailableNow]: 'want.availableNow',
  [WantBlockKind.Done]: 'want.done',
  [WantBlockKind.NoProfile]: 'want.noProfile',
}

const bannerText: Record<WantDiagnostic, Message> = {
  [WantDiagnostic.NoCatalog]: 'want.diagnostics.noCatalog',
  [WantDiagnostic.NoProfile]: 'want.diagnostics.noProfile',
  [WantDiagnostic.NothingUnlocks]: 'want.diagnostics.nothingUnlocks',
  [WantDiagnostic.NotUnlockable]: 'want.diagnostics.notUnlockable',
}
</script>

<template>
  <div class="flex flex-col gap-4">
    <Alert v-if="banner !== null">
      <InfoIcon />
      <AlertDescription>{{ t(bannerText[banner]) }}</AlertDescription>
    </Alert>
    <!-- One block per way in. Two achievements granting the same thing is two blocks, each
         with its own button: neither is hidden and neither is chosen for you. -->
    <section
      v-for="(block, index) in blocks"
      :key="nodeNumber(block.node)"
      class="flex flex-col gap-2"
    >
      <h2 class="text-label text-subtle-foreground">
        {{ t(heading[block.kind]) }}
      </h2>
      <p v-if="blocks.length > 1" class="text-body text-subtle-foreground">
        {{ t('want.wayOf', { index: index + 1, total: blocks.length }) }}
      </p>
      <!-- The steps in order, then the thing you asked for. The position is the meaning, so
           it is drawn and the list is never re-sorted. -->
      <ol class="flex flex-col gap-2">
        <li
          v-for="(step, at) in block.steps"
          :key="nodeNumber(step)"
          class="flex items-start gap-2"
        >
          <span class="pt-3 text-label text-subtle-foreground">{{
            at + 1
          }}</span>
          <!-- No `+` on a step: the one offer is the block's own "put it all in the Plan"
               below, because a chain half in the queue is not what you asked for. -->
          <Card class="flex-1 flex-col gap-0 p-0">
            <GoalRow :model="rowModel(step, t)" :node="step" :busy="busy" />
          </Card>
        </li>
        <li class="flex items-start gap-2">
          <span class="pt-3 text-label text-subtle-foreground">{{
            block.steps.length + 1
          }}</span>
          <Card class="flex-1 flex-col gap-0 p-0">
            <GoalRow
              :model="rowModel(block.node, t)"
              :node="block.node"
              :busy="busy"
            />
          </Card>
        </li>
      </ol>
      <p v-if="block.unknown > 0" class="text-body text-subtle-foreground">
        {{ t('want.unknown', { count: block.unknown }) }}
      </p>
      <Button
        v-if="block.queueable && canWrite && block.achievement !== null"
        :variant="ButtonVariant.Secondary"
        :size="ButtonSize.Compact"
        class="self-start"
        :disabled="busy"
        @click="emit('queue', block.achievement)"
        >{{ t('want.addAll') }}</Button
      >
    </section>
  </div>
</template>
