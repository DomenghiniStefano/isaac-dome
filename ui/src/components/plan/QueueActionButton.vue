<script setup lang="ts">
import { ListMinusIcon, ListPlusIcon } from '@lucide/vue'
import type { Component } from 'vue'
import { computed } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { useQueueOffer } from '@/composables/useQueueOffer'
import { useMessages } from '@/i18n'
import type { Message } from '@/i18n/message'
import { QueueAction, queueAction } from '@/lib/plan/queueAction'
import type { QueueTarget } from '@/lib/plan/queueAction'

// The Actions button of a list row: adds the row's achievement to the Plan's queue, or takes
// it out when you asked for it. Disabled says nothing about why — the label says what a click
// does, and a disabled one does nothing.
const props = defineProps<{ target: QueueTarget | null }>()
const { t } = useMessages()
const { queue, membership, canWrite } = useQueueOffer()

const action = computed(() =>
  queueAction(props.target, membership.value, canWrite.value),
)

const label: Record<QueueAction, Message> = {
  [QueueAction.Add]: 'queue.add',
  [QueueAction.Remove]: 'queue.remove',
  [QueueAction.Unavailable]: 'queue.add',
}
const icon: Record<QueueAction, Component> = {
  [QueueAction.Add]: ListPlusIcon,
  [QueueAction.Remove]: ListMinusIcon,
  [QueueAction.Unavailable]: ListPlusIcon,
}

const act = async (): Promise<void> => {
  const target = props.target
  if (target === null) return
  if (action.value === QueueAction.Add) await queue.add(target.achievement)
  if (action.value === QueueAction.Remove)
    await queue.remove(target.achievement)
}
</script>

<template>
  <Tooltip :disabled="action === QueueAction.Unavailable">
    <TooltipTrigger as-child>
      <Button
        :variant="ButtonVariant.Ghost"
        :size="ButtonSize.IconCompact"
        :aria-label="t(label[action])"
        :disabled="action === QueueAction.Unavailable || queue.busy"
        @click="act"
      >
        <component :is="icon[action]" />
      </Button>
    </TooltipTrigger>
    <TooltipContent>{{ t(label[action]) }}</TooltipContent>
  </Tooltip>
</template>
