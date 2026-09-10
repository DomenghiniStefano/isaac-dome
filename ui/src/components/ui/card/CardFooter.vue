<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <div
    data-slot="card-footer"
    :class="
      cn(
        'flex items-center gap-2 border-t border-hairline px-3 py-2.5',
        props.class,
      )
    "
  >
    <slot />
  </div>
</template>
