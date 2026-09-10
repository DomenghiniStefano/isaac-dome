<script setup lang="ts">
import type { SelectTriggerProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { ChevronDownIcon } from '@lucide/vue'
import { reactiveOmit } from '@vueuse/core'
import { SelectIcon, SelectTrigger, useForwardProps } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  SelectTriggerProps & { class?: HTMLAttributes['class'] }
>()

const delegatedProps = reactiveOmit(props, 'class')
const forwardedProps = useForwardProps(delegatedProps)
</script>

<template>
  <!-- One control height for every field. The chevron turns in three steps when the list
       opens: the trigger's own data-state drives it. -->
  <SelectTrigger
    data-slot="select-trigger"
    v-bind="forwardedProps"
    :class="
      cn(
        'group flex h-control w-fit min-w-40 cursor-pointer items-center justify-between gap-2 rounded-input border border-input bg-data px-3 text-body whitespace-nowrap text-foreground hover:bg-row-hover disabled:cursor-not-allowed disabled:border-secondary disabled:bg-muted data-[placeholder]:text-faint-foreground *:data-[slot=select-value]:line-clamp-1 [&_svg]:pointer-events-none [&_svg]:shrink-0',
        props.class,
      )
    "
  >
    <slot />
    <SelectIcon as-child>
      <ChevronDownIcon
        class="size-3.5 text-muted-foreground transition-transform duration-panel ease-panel group-data-[state=open]:rotate-180"
      />
    </SelectIcon>
  </SelectTrigger>
</template>
