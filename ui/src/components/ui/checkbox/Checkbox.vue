<script setup lang="ts">
import type { CheckboxRootEmits, CheckboxRootProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { CheckIcon } from '@lucide/vue'
import { reactiveOmit } from '@vueuse/core'
import { CheckboxIndicator, CheckboxRoot, useForwardPropsEmits } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  CheckboxRootProps & { class?: HTMLAttributes['class'] }
>()
const emits = defineEmits<CheckboxRootEmits>()

const delegatedProps = reactiveOmit(props, 'class')
const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <!-- Checked and indeterminate share the red fill and the lit edge; the indicator's own
       data-state picks the tick or the bar, in CSS, with no string compared in script. -->
  <CheckboxRoot
    v-slot="slotProps"
    data-slot="checkbox"
    v-bind="forwarded"
    :class="
      cn(
        'peer relative flex size-4 shrink-0 cursor-pointer items-center justify-center border border-input bg-data text-foreground disabled:cursor-not-allowed disabled:border-secondary disabled:bg-muted data-[state=checked]:border-selection-edge data-[state=checked]:bg-primary data-[state=indeterminate]:border-selection-edge data-[state=indeterminate]:bg-primary',
        props.class,
      )
    "
  >
    <CheckboxIndicator
      data-slot="checkbox-indicator"
      class="group grid place-content-center"
    >
      <slot v-bind="slotProps">
        <CheckIcon
          class="size-3 animate-tap-in group-data-[state=indeterminate]:hidden"
        />
        <span
          class="hidden h-0.5 w-2.5 bg-foreground group-data-[state=indeterminate]:block"
        />
      </slot>
    </CheckboxIndicator>
  </CheckboxRoot>
</template>
