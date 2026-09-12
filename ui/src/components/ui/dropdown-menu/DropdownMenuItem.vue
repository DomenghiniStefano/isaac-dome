<script setup lang="ts">
import type { DropdownMenuItemEmits, DropdownMenuItemProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { DropdownMenuItem, useForwardPropsEmits } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  DropdownMenuItemProps & { class?: HTMLAttributes['class'] }
>()
const emits = defineEmits<DropdownMenuItemEmits>()

const delegatedProps = reactiveOmit(props, 'class')
const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <!-- A disabled entry stays readable and stops taking the gesture: it says "we can't take
       you there", which is not the same as not being in the way. -->
  <DropdownMenuItem
    data-slot="dropdown-menu-item"
    v-bind="forwarded"
    :class="
      cn(
        'flex w-full cursor-default items-center gap-2 py-1.5 pr-6 pl-3 text-row text-foreground outline-none select-none data-[disabled]:pointer-events-none data-[disabled]:text-faint-foreground data-[highlighted]:bg-secondary',
        props.class,
      )
    "
  >
    <slot />
  </DropdownMenuItem>
</template>
