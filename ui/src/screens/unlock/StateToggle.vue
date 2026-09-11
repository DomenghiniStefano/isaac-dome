<script setup lang="ts">
import {
  ToggleGroup,
  ToggleGroupItem,
  ToggleGroupType,
} from '@/components/ui/toggle-group'
import { useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import { NodeState, stateOrder } from '@/lib/graph/nodeState'
import { stateText } from './facetLabels'

defineProps<{ counts: Record<NodeState, number>; picked: string[] }>()
const emit = defineEmits<{ update: [picked: string[]] }>()
const { t } = useMessages()

// A state is never colour alone: the square carries its colour, the name says it. Partial's
// square is the blocked colour with a dashed edge, like its badge.
const dot: Record<NodeState, string> = {
  [NodeState.Done]: 'bg-state-done',
  [NodeState.Now]: 'bg-state-now',
  [NodeState.Blocked]: 'bg-state-blocked',
  [NodeState.Partial]: 'border border-dashed border-state-blocked',
}

const onUpdate = (value: unknown) =>
  emit('update', Array.isArray(value) ? value.map(String) : [])
</script>

<template>
  <!-- The filter that matters more than the others (DESIGN-BRIEF.md §6): the four states, each
       with its count, any of them picked at once. -->
  <ToggleGroup
    :type="ToggleGroupType.Multiple"
    :model-value="picked"
    class="w-fit"
    @update:model-value="onUpdate"
  >
    <ToggleGroupItem
      v-for="state in stateOrder"
      :key="state"
      :value="state"
      class="gap-2"
    >
      <span aria-hidden="true" :class="cn('size-2 shrink-0', dot[state])" />
      {{ t(stateText[state]) }}
      <span class="tabular-nums">{{ counts[state] }}</span>
    </ToggleGroupItem>
  </ToggleGroup>
</template>
