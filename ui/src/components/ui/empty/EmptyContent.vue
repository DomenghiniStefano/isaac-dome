<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <div
    data-slot="empty-content"
    :class="cn('mt-1 flex flex-col items-center gap-2', props.class)"
  >
    <slot />
  </div>
</template>
