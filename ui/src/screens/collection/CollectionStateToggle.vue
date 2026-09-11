<script setup lang="ts">
import {
  ToggleGroup,
  ToggleGroupItem,
  ToggleGroupType,
} from '@/components/ui/toggle-group'
import { useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import { ItemState, itemStateOrder } from '@/lib/collection/itemState'
import { itemStateText } from './collectionLabels'

defineProps<{ counts: Record<ItemState, number>; picked: string[] }>()
const emit = defineEmits<{ update: [picked: string[]] }>()
const { t } = useMessages()

// A state is never colour alone: the square carries its colour, the name says it. Unreadable
// wears the unknown hatch, as its badge does.
const dot: Record<ItemState, string> = {
  [ItemState.InCollection]: 'bg-state-done',
  [ItemState.Available]: 'bg-state-now',
  [ItemState.Locked]: 'bg-state-blocked',
  [ItemState.Unknown]:
    'hatch-unknown border border-dashed border-state-unknown',
}

const onUpdate = (value: unknown) =>
  emit('update', Array.isArray(value) ? value.map(String) : [])
</script>

<template>
  <!-- The Collection's first filter: the four states, each with its count, any of them picked. -->
  <ToggleGroup
    :type="ToggleGroupType.Multiple"
    :model-value="picked"
    class="w-fit"
    @update:model-value="onUpdate"
  >
    <ToggleGroupItem
      v-for="state in itemStateOrder"
      :key="state"
      :value="state"
      class="gap-2"
    >
      <span aria-hidden="true" :class="cn('size-2 shrink-0', dot[state])" />
      {{ t(itemStateText[state]) }}
      <span class="tabular-nums">{{ counts[state] }}</span>
    </ToggleGroupItem>
  </ToggleGroup>
</template>
