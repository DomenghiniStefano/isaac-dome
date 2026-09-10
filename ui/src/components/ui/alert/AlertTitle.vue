<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <div
    data-slot="alert-title"
    :class="cn('col-start-2 text-body text-foreground', props.class)"
  >
    <slot />
  </div>
</template>
