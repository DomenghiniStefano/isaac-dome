<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <div data-slot="card-content" :class="cn('p-3 text-body', props.class)">
    <slot />
  </div>
</template>
