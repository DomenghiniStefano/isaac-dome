<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <div
    data-slot="card"
    :class="
      cn(
        // `bg-card` stays under the wash: a gradient is an image, and a surface that draws
        // one still needs a colour for the moment before it paints and for anything that
        // reads the background rather than the picture on it.
        'flex flex-col border border-border border-t-hairline-lit bg-card card-wash text-card-foreground',
        props.class,
      )
    "
  >
    <slot />
  </div>
</template>
