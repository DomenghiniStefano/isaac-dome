<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <!-- The leather band. Its row opts out of the body's stripes and hover: the thead
       selector outranks the row's own classes. -->
  <thead
    data-slot="table-header"
    :class="
      cn(
        'text-band-foreground [&>tr]:border-b-0 [&>tr]:bg-band [&>tr:hover]:bg-band',
        props.class,
      )
    "
  >
    <slot />
  </thead>
</template>
