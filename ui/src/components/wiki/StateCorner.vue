<script setup lang="ts">
import { computed } from 'vue'
import { Badge, stateBadgeVariant } from '@/components/ui/badge'
import { useMessages } from '@/i18n'
import type { CardState } from '@/lib/wiki/cardState'

// A list card's save state as a mark in its corner, not as one more pill among the facts: the
// state's icon in its colour (the badge draws it), every line of it on hover. Nothing without
// a state.
const props = defineProps<{ state: CardState | null }>()
const { t } = useMessages()

const variant = computed(() =>
  props.state ? stateBadgeVariant[props.state.tone] : null,
)
const text = computed(() =>
  props.state
    ? props.state.lines
        .map((line) => t(line.label.key, line.label.params))
        .join(' · ')
    : '',
)
</script>

<template>
  <Badge
    v-if="state && variant"
    :variant="variant"
    :title="text"
    :aria-label="text"
    class="size-7 justify-center rounded-full p-0"
  />
</template>
