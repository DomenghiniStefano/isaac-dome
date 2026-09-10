<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <div
    data-slot="popover-title"
    :class="
      cn('bg-band px-3 py-1.5 text-control text-band-foreground', props.class)
    "
  >
    <slot />
  </div>
</template>
