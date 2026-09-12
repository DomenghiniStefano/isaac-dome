<script setup lang="ts">
import { ListPlusIcon } from '@lucide/vue'
import { computed } from 'vue'
import AchievementArt from '@/components/graph/AchievementArt.vue'
import NodeStateBadge from '@/components/graph/NodeStateBadge.vue'
import { ArtSize } from '@/components/graph/artSize'
import { unlockKindText } from '@/components/graph/unlockKindText'
import { Badge, BadgeVariant } from '@/components/ui/badge'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { useMessages } from '@/i18n'
import { targetName } from '@/lib/graph/characterName'
import { targetKind } from '@/lib/graph/unlockFilter'
import type { UnlockNode } from '@/lib/ipc/types'

const props = defineProps<{
  rank: number
  node: UnlockNode
  queued: boolean
  canAdd: boolean
  busy: boolean
}>()
const emit = defineEmits<{ add: [] }>()
const { t } = useMessages()

const text = computed(() => {
  const a = props.node.achievement
  return a.kind === 'known'
    ? a.text
    : `${t('graph.unknownAchievement')} · ${t('graph.slot')} ${a.slot}`
})
const art = computed(() =>
  props.node.achievement.kind === 'known'
    ? props.node.achievement.iconUrl
    : null,
)
</script>

<template>
  <!-- Schermate.dc.html, "Prossimi passi": the rank, the drawing, what it unlocks, and how
       much it opens. Gold is allowed on the count: every step is unlockable now. Under the
       state, whether the step is already in the Plan's queue, or one click to put it there. -->
  <Card class="flex-row items-start gap-3 p-3">
    <span
      class="mt-1 w-4 shrink-0 text-label text-subtle-foreground tabular-nums"
      >{{ rank }}</span
    >
    <AchievementArt :url="art" :size="ArtSize.Card" />
    <div class="flex min-w-0 flex-1 flex-col items-start gap-2">
      <span class="text-body text-foreground">{{ text }}</span>
      <div v-if="node.unlocks.length > 0" class="flex flex-wrap gap-1.5">
        <Badge
          v-for="target in node.unlocks"
          :key="`${target.kind}-${target.id}`"
          :variant="BadgeVariant.Tag"
          >{{ targetName(t, target) }} ·
          {{ t(unlockKindText[targetKind(target)]) }}</Badge
        >
      </div>
      <NodeStateBadge :node="node" />
      <span v-if="queued" class="text-caption text-state-done-foreground">{{
        t('queue.inPlan')
      }}</span>
      <Button
        v-else-if="canAdd"
        :variant="ButtonVariant.Outline"
        :size="ButtonSize.Compact"
        :disabled="busy"
        @click="emit('add')"
        ><ListPlusIcon />{{ t('queue.add') }}</Button
      >
    </div>
    <div class="flex shrink-0 flex-col items-end">
      <span class="text-kpi text-state-now-foreground tabular-nums">{{
        node.graph.fanOut
      }}</span>
      <span class="text-label text-subtle-foreground">{{
        t('nextSteps.unlocks')
      }}</span>
    </div>
  </Card>
</template>
