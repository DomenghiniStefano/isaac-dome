<script setup lang="ts">
import type { CollapsibleRootEmits, CollapsibleRootProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { CollapsibleRoot, useForwardPropsEmits } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  CollapsibleRootProps & { class?: HTMLAttributes['class'] }
>()
const emits = defineEmits<CollapsibleRootEmits>()

const delegatedProps = reactiveOmit(props, 'class')
const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <!-- The configuration screens' brick (Chrome e Stati.dc.html, "Card comprimibile"). Parts,
       not a prop on Card: a prop can't turn a div into Reka's collapsible root. -->
  <CollapsibleRoot
    v-slot="slotProps"
    data-slot="card-collapsible"
    v-bind="forwarded"
    :class="cn('flex flex-col border border-border bg-sheet', props.class)"
  >
    <slot v-bind="slotProps" />
  </CollapsibleRoot>
</template>
