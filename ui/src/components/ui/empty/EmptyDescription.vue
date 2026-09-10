<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <p
    data-slot="empty-description"
    :class="cn('text-row text-muted-foreground', props.class)"
  >
    <slot />
  </p>
</template>
