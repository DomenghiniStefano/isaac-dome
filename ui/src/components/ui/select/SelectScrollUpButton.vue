<script setup lang="ts">
import type { SelectScrollUpButtonProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { ChevronUpIcon } from '@lucide/vue'
import { reactiveOmit } from '@vueuse/core'
import { SelectScrollUpButton, useForwardProps } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  SelectScrollUpButtonProps & { class?: HTMLAttributes['class'] }
>()

const delegatedProps = reactiveOmit(props, 'class')
const forwardedProps = useForwardProps(delegatedProps)
</script>

<template>
  <SelectScrollUpButton
    data-slot="select-scroll-up-button"
    v-bind="forwardedProps"
    :class="
      cn(
        'flex cursor-default items-center justify-center bg-popover py-1 text-muted-foreground [&_svg:not([class*=size-])]:size-3.5',
        props.class,
      )
    "
  >
    <slot>
      <ChevronUpIcon />
    </slot>
  </SelectScrollUpButton>
</template>
