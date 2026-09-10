<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <div
    data-slot="dialog-footer"
    :class="
      cn('flex justify-end gap-2 border-t border-border px-3 py-3', props.class)
    "
  >
    <slot />
  </div>
</template>
