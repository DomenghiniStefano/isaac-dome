<script setup lang="ts">
import {
  ToggleGroup,
  ToggleGroupItem,
  ToggleGroupType,
} from '@/components/ui/toggle-group'
import { useMessages } from '@/i18n'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import { cn } from '@/lib/cn'

// The filter that matters more than the others (DESIGN-BRIEF.md §6): a screen's states, each
// with its count, any of them picked at once.
//
// What the states are, what they are called and what colour each square carries belong to the
// screen and arrive as tables. A state is never colour alone: the square carries the colour,
// the name says it.
defineProps<{
  order: string[]
  counts: Record<string, number>
  picked: string[]
  dot: Record<string, string>
  text: Record<string, MessageKey<MessageSchema>>
}>()
const emit = defineEmits<{ update: [picked: string[]] }>()
const { t } = useMessages()

const onUpdate = (value: unknown) =>
  emit('update', Array.isArray(value) ? value.map(String) : [])
</script>

<template>
  <ToggleGroup
    :type="ToggleGroupType.Multiple"
    :model-value="picked"
    class="w-fit"
    @update:model-value="onUpdate"
  >
    <ToggleGroupItem
      v-for="state in order"
      :key="state"
      :value="state"
      class="gap-2"
    >
      <span aria-hidden="true" :class="cn('size-2 shrink-0', dot[state])" />
      {{ t(text[state]) }}
      <span class="tabular-nums">{{ counts[state] }}</span>
    </ToggleGroupItem>
  </ToggleGroup>
</template>
