<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <kbd
    data-slot="kbd"
    :class="
      cn(
        'pointer-events-none inline-flex h-5 min-w-5 items-center justify-center gap-1 border border-secondary-edge px-1.5 text-label text-foreground select-none [&_svg:not([class*=size-])]:size-3',
        props.class,
      )
    "
  >
    <slot />
  </kbd>
</template>
