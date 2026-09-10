<script setup lang="ts">
import type { SelectLabelProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { SelectLabel } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  SelectLabelProps & { class?: HTMLAttributes['class'] }
>()

const delegatedProps = reactiveOmit(props, 'class')
</script>

<template>
  <SelectLabel
    data-slot="select-label"
    v-bind="delegatedProps"
    :class="
      cn('px-3 pt-1.5 pb-1 text-label text-subtle-foreground', props.class)
    "
  >
    <slot />
  </SelectLabel>
</template>
