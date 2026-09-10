<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <th
    data-slot="table-head"
    :class="
      cn(
        'h-7 px-2 text-left align-middle text-label whitespace-nowrap',
        props.class,
      )
    "
  >
    <slot />
  </th>
</template>
