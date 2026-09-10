<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <tfoot
    data-slot="table-footer"
    :class="
      cn(
        'border-t border-border text-label text-subtle-foreground [&>tr]:border-b-0 [&>tr:hover]:bg-transparent',
        props.class,
      )
    "
  >
    <slot />
  </tfoot>
</template>
