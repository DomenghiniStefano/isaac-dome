<script setup lang="ts">
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import { Button, ButtonVariant } from '@/components/ui/button'
import { useMessages } from '@/i18n'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import type { EmptyList } from '@/lib/facets/emptyList'

type Message = MessageKey<MessageSchema>

// A list that came back with nothing: what `emptyList` says it is, and the button that undoes
// the filter when there is one to undo. The two are `emptyList`'s decision, not this frame's.
withDefaults(
  defineProps<{ empty: EmptyList<Message>; resetText?: Message }>(),
  { resetText: 'filters.reset' },
)
const emit = defineEmits<{ reset: [] }>()
const { t } = useMessages()
</script>

<template>
  <div class="flex flex-col items-start gap-3 p-4">
    <EmptyCategory>{{ t(empty.text) }}</EmptyCategory>
    <Button
      v-if="empty.reset"
      :variant="ButtonVariant.Outline"
      @click="emit('reset')"
      >{{ t(resetText) }}</Button
    >
  </div>
</template>
