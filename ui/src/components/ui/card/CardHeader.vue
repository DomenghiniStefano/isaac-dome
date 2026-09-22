<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <div
    data-slot="card-header"
    :class="
      cn(
        'flex items-center justify-between gap-2 bg-band band-wash px-3 py-2 text-band-foreground',
        props.class,
      )
    "
  >
    <slot />
  </div>
</template>
