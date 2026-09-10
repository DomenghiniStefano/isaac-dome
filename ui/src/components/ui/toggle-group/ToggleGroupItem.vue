<script setup lang="ts">
import type { ToggleGroupItemProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { ToggleGroupItem, useForwardProps } from 'reka-ui'
import { computed, inject } from 'vue'
import { cn } from '@/lib/cn'
import {
  ToggleSize,
  toggleGroupItemVariants,
  toggleGroupSizeKey,
} from './variants'

const props = defineProps<
  ToggleGroupItemProps & {
    class?: HTMLAttributes['class']
    size?: ToggleSize
  }
>()

const groupSize = inject(toggleGroupSizeKey, undefined)
const size = computed(
  () => props.size ?? groupSize?.value ?? ToggleSize.Default,
)

const delegatedProps = reactiveOmit(props, 'class', 'size')
const forwardedProps = useForwardProps(delegatedProps)
</script>

<template>
  <ToggleGroupItem
    v-slot="slotProps"
    data-slot="toggle-group-item"
    :data-size="size"
    v-bind="forwardedProps"
    :class="cn(toggleGroupItemVariants({ size }), props.class)"
  >
    <slot v-bind="slotProps" />
  </ToggleGroupItem>
</template>
