<script setup lang="ts">
import type { RadioGroupItemProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { RadioGroupIndicator, RadioGroupItem, useForwardProps } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  RadioGroupItemProps & { class?: HTMLAttributes['class'] }
>()

const delegatedProps = reactiveOmit(props, 'class')
const forwardedProps = useForwardProps(delegatedProps)
</script>

<template>
  <!-- 16px, a round edge on the data surface; chosen, the edge lights and an 8px dot
       appears (Shadcn Kit.dc.html, "Checkbox · Radio · Switch · Toggle"). The dot is
       currentColor, so disabled keeps it, faint, like Checkbox keeps its tick. -->
  <RadioGroupItem
    data-slot="radio-group-item"
    v-bind="forwardedProps"
    :class="
      cn(
        'peer relative flex size-4 shrink-0 cursor-pointer items-center justify-center rounded-full border border-input bg-data text-selection-mark disabled:cursor-not-allowed disabled:border-secondary disabled:bg-muted disabled:text-faint-foreground enabled:data-[state=checked]:border-selection-edge',
        props.class,
      )
    "
  >
    <RadioGroupIndicator
      data-slot="radio-group-indicator"
      class="flex items-center justify-center"
    >
      <slot>
        <span class="size-2 rounded-full bg-current" />
      </slot>
    </RadioGroupIndicator>
  </RadioGroupItem>
</template>
