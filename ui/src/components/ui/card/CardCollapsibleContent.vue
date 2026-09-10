<script setup lang="ts">
import type { CollapsibleContentProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { CollapsibleContent } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  CollapsibleContentProps & { class?: HTMLAttributes['class'] }
>()

const delegatedProps = reactiveOmit(props, 'class')
</script>

<template>
  <CollapsibleContent
    data-slot="card-collapsible-content"
    v-bind="delegatedProps"
    :class="
      cn(
        'animate-panel-open px-3 py-2.75 text-caption text-foreground-soft',
        props.class,
      )
    "
  >
    <slot />
  </CollapsibleContent>
</template>
