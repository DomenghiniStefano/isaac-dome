<script setup lang="ts">
import type { SelectItemProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { CheckIcon } from '@lucide/vue'
import { reactiveOmit } from '@vueuse/core'
import {
  SelectItem,
  SelectItemIndicator,
  SelectItemText,
  useForwardProps,
} from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  SelectItemProps & { class?: HTMLAttributes['class'] }
>()

const delegatedProps = reactiveOmit(props, 'class')
const forwardedProps = useForwardProps(delegatedProps)
</script>

<template>
  <!-- The tick is foreground, not the done green the kit draws: selection is not a state
       of the data. -->
  <SelectItem
    data-slot="select-item"
    v-bind="forwardedProps"
    :class="
      cn(
        'relative flex w-full cursor-default items-center gap-2 py-2 pr-8 pl-3 text-row text-foreground outline-none select-none data-[disabled]:pointer-events-none data-[disabled]:text-faint-foreground data-[highlighted]:bg-secondary [&_svg:not([class*=size-])]:size-3.5',
        props.class,
      )
    "
  >
    <span
      class="pointer-events-none absolute right-3 flex size-3.5 items-center justify-center"
    >
      <SelectItemIndicator>
        <slot name="indicator-icon">
          <CheckIcon />
        </slot>
      </SelectItemIndicator>
    </span>
    <SelectItemText>
      <slot />
    </SelectItemText>
  </SelectItem>
</template>
