<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'
import { FieldOrientation, fieldVariants } from './variants'

const props = withDefaults(
  defineProps<{
    class?: HTMLAttributes['class']
    orientation?: FieldOrientation
  }>(),
  {
    orientation: FieldOrientation.Vertical,
  },
)
</script>

<template>
  <div
    role="group"
    data-slot="field"
    :data-orientation="orientation"
    :class="cn(fieldVariants({ orientation }), props.class)"
  >
    <slot />
  </div>
</template>
