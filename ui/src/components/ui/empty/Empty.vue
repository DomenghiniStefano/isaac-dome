<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <div
    data-slot="empty"
    :class="
      cn(
        'flex w-full min-w-0 flex-col items-center justify-center gap-3 border border-dashed border-secondary-edge bg-data px-4.5 py-6.5 text-center',
        props.class,
      )
    "
  >
    <slot />
  </div>
</template>
