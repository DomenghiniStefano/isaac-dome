<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <div
    data-slot="empty-media"
    :class="
      cn(
        'flex shrink-0 items-center justify-center text-faint-foreground [&_svg:not([class*=size-])]:size-8.5',
        props.class,
      )
    "
  >
    <slot />
  </div>
</template>
