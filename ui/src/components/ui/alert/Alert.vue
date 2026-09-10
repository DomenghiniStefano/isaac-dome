<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'
import type { AlertVariant } from './variants'
import { alertVariants } from './variants'

const props = defineProps<{
  class?: HTMLAttributes['class']
  variant?: AlertVariant
}>()
</script>

<template>
  <div
    data-slot="alert"
    role="alert"
    :class="cn(alertVariants({ variant }), props.class)"
  >
    <slot />
  </div>
</template>
