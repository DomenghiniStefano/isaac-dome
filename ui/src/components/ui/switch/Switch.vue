<script setup lang="ts">
import type { SwitchRootEmits, SwitchRootProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { SwitchRoot, SwitchThumb, useForwardPropsEmits } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  SwitchRootProps & { class?: HTMLAttributes['class'] }
>()
const emits = defineEmits<SwitchRootEmits>()

const delegatedProps = reactiveOmit(props, 'class')
const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <!-- Square, 34x18 with a 12px thumb (Shadcn Kit.dc.html). Inner width 28px, so the thumb
       travels 16px: translate-x-4, in two steps. -->
  <SwitchRoot
    v-slot="slotProps"
    data-slot="switch"
    v-bind="forwarded"
    :class="
      cn(
        'group peer inline-flex h-4.5 w-8.5 shrink-0 cursor-pointer items-center border border-input bg-data p-0.5 disabled:cursor-not-allowed disabled:border-secondary disabled:bg-muted enabled:data-[state=checked]:border-selection-edge enabled:data-[state=checked]:bg-primary',
        props.class,
      )
    "
  >
    <SwitchThumb
      data-slot="switch-thumb"
      class="pointer-events-none block size-3 bg-faint-foreground transition-transform duration-tap ease-tap data-[state=checked]:translate-x-4 group-enabled:data-[state=checked]:bg-foreground"
    >
      <slot name="thumb" v-bind="slotProps" />
    </SwitchThumb>
  </SwitchRoot>
</template>
