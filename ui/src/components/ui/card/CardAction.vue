<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <div data-slot="card-action" :class="cn('text-label', props.class)">
    <slot />
  </div>
</template>
