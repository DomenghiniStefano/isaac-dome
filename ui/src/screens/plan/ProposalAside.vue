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
import { canQueue, isQueued, knownText } from '@/lib/plan/queueRows'

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
       your queue itself. -->
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
          <span class="min-w-0 flex-1 truncate text-caption text-foreground">{{
            knownText(step)
          }}</span>
          <span class="text-label text-state-now-foreground tabular-nums">{{
            step.graph.fanOut
          }}</span>
          <span
            v-if="isQueued(step, queued)"
            class="text-label text-state-done-foreground"
            >{{ t('queue.inQueue') }}</span
          >
          <Button
            v-else-if="canWrite && canQueue(step, queued)"
            :variant="ButtonVariant.Outline"
            :size="ButtonSize.Compact"
            :aria-label="t('queue.add')"
            :disabled="busy"
            @click="emit('add', nodeSlot(step))"
            ><ListPlusIcon />{{ t('queue.addShort') }}</Button
          >
        </div>
      </div>
      <EmptyValue v-else>{{ t('plan.aside.empty') }}</EmptyValue>
    </CardContent>
  </Card>
</template>
