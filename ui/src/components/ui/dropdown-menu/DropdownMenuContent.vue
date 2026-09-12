<script setup lang="ts">
import type {
  DropdownMenuContentEmits,
  DropdownMenuContentProps,
} from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import {
  DropdownMenuContent,
  DropdownMenuPortal,
  useForwardPropsEmits,
} from 'reka-ui'
import { cn } from '@/lib/cn'
import { Align } from '@/lib/constants/placement'

defineOptions({
  inheritAttrs: false,
})

const props = withDefaults(
  defineProps<DropdownMenuContentProps & { class?: HTMLAttributes['class'] }>(),
  {
    align: Align.Start,
    sideOffset: 4,
  },
)
const emits = defineEmits<DropdownMenuContentEmits>()

const delegatedProps = reactiveOmit(props, 'class')
const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <!-- A menu is as wide as its longest entry, bounded by what the window has left: a fixed
       width would either clip a boss's name or leave a column of air beside a short one. -->
  <DropdownMenuPortal>
    <DropdownMenuContent
      data-slot="dropdown-menu-content"
      v-bind="{ ...$attrs, ...forwarded }"
      :class="
        cn(
          'z-50 flex max-w-(--reka-dropdown-menu-content-available-width) min-w-48 animate-panel-rise flex-col border border-input bg-popover py-1 text-popover-foreground',
          props.class,
        )
      "
    >
      <slot />
    </DropdownMenuContent>
  </DropdownMenuPortal>
</template>
