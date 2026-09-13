<script setup lang="ts">
import { ListPlusIcon } from '@lucide/vue'
import { computed } from 'vue'
import AchievementArt from '@/components/graph/AchievementArt.vue'
import { ArtSize } from '@/components/graph/artSize'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { useMessages } from '@/i18n'
import { goalCard } from '@/lib/graph/goalCard'
import type { UnlockNode } from '@/lib/ipc/types'
import type { TabLocation } from '@/router/routeTable'

const props = defineProps<{
  node: UnlockNode
  queued: boolean
  canAdd: boolean
  busy: boolean
  // The row under a heading that already says these are in the Plan: it drops the drawing
  // and the queue line, which would repeat the heading on every row.
  compact?: boolean
}>()
const emit = defineEmits<{
  add: []
  navigate: [location: TabLocation, newTab: boolean]
}>()
const { t } = useMessages()

const card = computed(() => goalCard(props.node, t))
const opensText = computed(() =>
  card.value.fanOut > 0
    ? t('goals.opens', { count: card.value.fanOut })
    : t('goals.opensNothing'),
)

const open = (newTab: boolean) => {
  if (card.value.location) emit('navigate', card.value.location, newTab)
}
</script>

<template>
  <!-- What you get, how you get it, why it is worth it, and the one action (B32 §3). No state
       badge: every row of this page is unlockable now, so it would say nothing.
       The headline and the drawing are the links — not the whole card, which also holds a
       button: a card that is itself a link and contains one is a trap for the keyboard, and
       `Ref` is already the app's way of saying "this goes to a page". -->
  <Card class="flex-row items-start gap-3 p-3">
    <template v-if="!compact">
      <Button
        v-if="card.location"
        :variant="ButtonVariant.Ghost"
        :size="ButtonSize.IconCompact"
        class="h-auto w-auto p-0"
        @click="open($event.ctrlKey)"
      >
        <AchievementArt :url="card.art" :size="ArtSize.Card" />
      </Button>
      <AchievementArt v-else :url="card.art" :size="ArtSize.Card" />
    </template>

    <div class="flex min-w-0 flex-1 flex-col items-start gap-1.5">
      <Button
        v-if="card.location"
        :variant="ButtonVariant.Ref"
        :size="ButtonSize.Inline"
        class="text-row"
        @click="open($event.ctrlKey)"
        >{{ card.headline }}</Button
      >
      <span v-else class="text-row text-foreground">{{ card.headline }}</span>

      <!-- The game's own `unlock_condition`. Absent when the file states none: an invented
           sentence would read exactly like a quoted one. -->
      <span v-if="card.condition" class="text-caption text-subtle-foreground">{{
        card.condition
      }}</span>

      <span class="text-caption text-state-now-foreground">{{
        opensText
      }}</span>

      <span
        v-if="queued && !compact"
        class="text-caption text-state-done-foreground"
        >{{ t('queue.inPlan') }}</span
      >
      <Button
        v-else-if="canAdd && !compact"
        :variant="ButtonVariant.Outline"
        :size="ButtonSize.Compact"
        :disabled="busy"
        @click="emit('add')"
        ><ListPlusIcon />{{ t('queue.add') }}</Button
      >
    </div>
  </Card>
</template>
