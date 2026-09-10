<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { useVModel } from '@vueuse/core'
import { cn } from '@/lib/cn'

const props = defineProps<{
  defaultValue?: string | number
  modelValue?: string | number
  class?: HTMLAttributes['class']
}>()

const emits = defineEmits<{
  (e: 'update:modelValue', payload: string | number): void
}>()

const modelValue = useVModel(props, 'modelValue', emits, {
  passive: true,
  defaultValue: props.defaultValue,
})
</script>

<template>
  <input
    v-model="modelValue"
    data-slot="input"
    :class="
      cn(
        'h-control w-full min-w-0 rounded-input border border-input bg-data px-3 text-body text-foreground placeholder:text-faint-foreground disabled:pointer-events-none disabled:border-secondary disabled:bg-muted disabled:text-faint-foreground aria-invalid:border-destructive',
        props.class,
      )
    "
  />
</template>
