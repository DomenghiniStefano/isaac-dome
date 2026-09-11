<script setup lang="ts">
import { ListPlusIcon } from '@lucide/vue'
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import AchievementArt from '@/components/graph/AchievementArt.vue'
import { ArtSize } from '@/components/graph/artSize'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useMessages } from '@/i18n'
import { nodeSlot } from '@/lib/graph/unlockFilter'
import type { UnlockNode } from '@/lib/ipc/types'
import {
  canQueue,
  isQueued,
  knownText,
  proposalLabel,
} from '@/lib/plan/queueRows'

defineProps<{
  steps: UnlockNode[]
  queued: Set<number>
  canWrite: boolean
  busy: boolean
}>()
const emit = defineEmits<{ add: [achievement: number] }>()
const { t } = useMessages()

const iconOf = (node: UnlockNode): string | null =>
  node.achievement.kind === 'known' ? node.achievement.iconUrl : null
</script>

<template>
  <!-- Schermate.dc.html, the Plan's "Prossimi passi": the proposal beside your queue, never
       your queue itself. A narrow column names what a step unlocks: the achievement's text
       would be cut to "You …". -->
  <Card>
    <CardHeader>
      <CardTitle>{{ t('plan.aside.title') }}</CardTitle>
    </CardHeader>
    <CardContent class="flex flex-col gap-3">
      <p class="text-caption text-foreground-soft">
        {{ t('plan.aside.intro') }}
      </p>
      <div v-if="steps.length > 0" class="flex flex-col gap-2">
        <div
          v-for="step in steps"
          :key="nodeSlot(step)"
          class="flex items-center gap-2"
        >
          <AchievementArt :url="iconOf(step)" :size="ArtSize.Thumb" />
          <div class="flex min-w-0 flex-1 flex-col">
            <span
              class="truncate text-caption text-foreground"
              :title="knownText(step) ?? undefined"
              >{{ proposalLabel(step) }}</span
            >
            <span class="text-label text-subtle-foreground tabular-nums"
              >{{ t('plan.row.fanOut') }}: {{ step.graph.fanOut }}</span
            >
          </div>
          <span
            v-if="isQueued(step, queued)"
            class="shrink-0 text-label text-state-done-foreground"
            >{{ t('queue.inQueue') }}</span
          >
          <Button
            v-else-if="canWrite && canQueue(step, queued)"
            :variant="ButtonVariant.Outline"
            :size="ButtonSize.IconCompact"
            :aria-label="t('queue.add')"
            :disabled="busy"
            @click="emit('add', nodeSlot(step))"
          >
            <ListPlusIcon />
          </Button>
        </div>
      </div>
      <EmptyValue v-else>{{ t('plan.aside.empty') }}</EmptyValue>
    </CardContent>
  </Card>
</template>
