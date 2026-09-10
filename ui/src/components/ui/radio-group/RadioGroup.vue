<script setup lang="ts">
import type { RadioGroupRootEmits, RadioGroupRootProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { RadioGroupRoot, useForwardPropsEmits } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  RadioGroupRootProps & { class?: HTMLAttributes['class'] }
>()
const emits = defineEmits<RadioGroupRootEmits>()

const delegatedProps = reactiveOmit(props, 'class')
const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <!-- A row, as the kit lays "Normale · Hard · Greed" out; a column is flex-col from the
       consumer. orientation only steers the arrow keys. -->
  <RadioGroupRoot
    v-slot="slotProps"
    data-slot="radio-group"
    v-bind="forwarded"
    :class="cn('flex flex-wrap items-center gap-3.5', props.class)"
  >
    <slot v-bind="slotProps" />
  </RadioGroupRoot>
</template>
