<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <!-- The leather band; the right padding leaves room for the close control. -->
  <div
    data-slot="dialog-header"
    :class="
      cn(
        'flex items-center gap-2 bg-band py-2 pr-9 pl-3 text-band-foreground',
        props.class,
      )
    "
  >
    <slot />
  </div>
</template>
