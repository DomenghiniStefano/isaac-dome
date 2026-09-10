<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <div
    data-slot="empty-title"
    :class="cn('text-heading text-foreground', props.class)"
  >
    <slot />
  </div>
</template>
