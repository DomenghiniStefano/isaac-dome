<script setup lang="ts">
import type { ToggleGroupRootEmits, ToggleGroupRootProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { ToggleGroupRoot, useForwardPropsEmits } from 'reka-ui'
import { provide, toRef } from 'vue'
import { cn } from '@/lib/cn'
import { ToggleSize, toggleGroupSizeKey } from './variants'

const props = withDefaults(
  defineProps<
    ToggleGroupRootProps & {
      class?: HTMLAttributes['class']
      size?: ToggleSize
    }
  >(),
  {
    size: ToggleSize.Default,
  },
)
const emits = defineEmits<ToggleGroupRootEmits>()

provide(toggleGroupSizeKey, toRef(props, 'size'))

const delegatedProps = reactiveOmit(props, 'class', 'size')
const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <ToggleGroupRoot
    v-slot="slotProps"
    data-slot="toggle-group"
    v-bind="forwarded"
    :class="
      cn(
        'flex w-fit items-center border border-secondary-edge bg-data',
        props.class,
      )
    "
  >
    <slot v-bind="slotProps" />
  </ToggleGroupRoot>
</template>
