<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <!-- A sweep between two flat surfaces on steps, not a pulse: the loading strip of
       Motion.dc.html. The consumer gives the size, known ahead of time (brief §10). -->
  <div
    data-slot="skeleton"
    :class="cn('animate-skeleton bg-secondary', props.class)"
  />
</template>
