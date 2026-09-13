<script setup lang="ts">
import { InfoIcon } from '@lucide/vue'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { useMessages } from '@/i18n'
import { WantBlockKind } from '@/lib/graph/wantBlocks'
import type { WantBlock } from '@/lib/graph/wantBlocks'
import { nodeSlot } from '@/lib/graph/unlockFilter'
import type { WantDiagnostic } from '@/lib/ipc/types'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import type { TabLocation } from '@/router/routeTable'
import GoalCard from './GoalCard.vue'

type Message = MessageKey<MessageSchema>

defineProps<{
  blocks: WantBlock[]
  banner: WantDiagnostic | null
  canWrite: boolean
  busy: boolean
}>()
const emit = defineEmits<{
  queue: [achievement: number]
  navigate: [location: TabLocation, newTab: boolean]
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

const bannerText: Record<WantDiagnostic['kind'], Message> = {
  noCatalog: 'want.diagnostics.noCatalog',
  noProfile: 'want.diagnostics.noProfile',
  nothingUnlocks: 'want.diagnostics.nothingUnlocks',
  notUnlockable: 'want.diagnostics.notUnlockable',
}
</script>

<template>
  <div class="flex flex-col gap-4">
    <Alert v-if="banner !== null">
      <InfoIcon />
      <AlertDescription>{{ t(bannerText[banner.kind]) }}</AlertDescription>
    </Alert>
    <!-- One block per way in. Two achievements granting the same thing is two blocks, each
         with its own button: neither is hidden and neither is chosen for you. -->
    <section
      v-for="(block, index) in blocks"
      :key="nodeSlot(block.node)"
      class="flex flex-col gap-2"
    >
      <h2 class="text-label text-subtle-foreground">
        {{ t(heading[block.kind]) }}
      </h2>
      <p v-if="blocks.length > 1" class="text-sm text-subtle-foreground">
        {{ t('want.wayOf', { index: index + 1, total: blocks.length }) }}
      </p>
      <!-- The steps in order, then the thing you asked for. The position is the meaning, so
           it is drawn and the list is never re-sorted. -->
      <ol class="flex flex-col gap-2">
        <li
          v-for="(step, at) in block.steps"
          :key="nodeSlot(step)"
          class="flex items-start gap-2"
        >
          <span class="pt-3 text-label text-subtle-foreground">{{
            at + 1
          }}</span>
          <GoalCard
            class="flex-1"
            :node="step"
            :queued="false"
            :can-add="false"
            :busy="busy"
            compact
            @navigate="(location, newTab) => emit('navigate', location, newTab)"
          />
        </li>
        <li class="flex items-start gap-2">
          <span class="pt-3 text-label text-subtle-foreground">{{
            block.steps.length + 1
          }}</span>
          <GoalCard
            class="flex-1"
            :node="block.node"
            :queued="!block.queueable"
            :can-add="false"
            :busy="busy"
            @navigate="(location, newTab) => emit('navigate', location, newTab)"
          />
        </li>
      </ol>
      <p v-if="block.unknown > 0" class="text-sm text-subtle-foreground">
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
