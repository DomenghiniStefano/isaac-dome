<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { uniq } from 'lodash-es'
import { computed } from 'vue'
import { cn } from '@/lib/cn'

type FieldErrorEntry = string | { message: string | undefined } | undefined

const props = defineProps<{
  class?: HTMLAttributes['class']
  errors?: FieldErrorEntry[]
}>()

const messageOf = (entry: FieldErrorEntry): string | undefined =>
  typeof entry === 'string' ? entry : entry?.message

// The distinct messages, in order; the field's red edge says where, this says what.
const messages = computed(() =>
  uniq((props.errors ?? []).map(messageOf).filter((m): m is string => !!m)),
)
</script>

<template>
  <div
    v-if="$slots.default || messages.length"
    role="alert"
    data-slot="field-error"
    :class="cn('text-caption text-foreground', props.class)"
  >
    <slot v-if="$slots.default" />
    <template v-else-if="messages.length === 1">{{ messages[0] }}</template>
    <ul v-else class="ml-4 flex list-disc flex-col gap-1">
      <li v-for="message in messages" :key="message">{{ message }}</li>
    </ul>
  </div>
</template>
