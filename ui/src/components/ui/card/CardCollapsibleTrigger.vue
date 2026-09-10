<script setup lang="ts">
import type { CollapsibleTriggerProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { ChevronRightIcon } from '@lucide/vue'
import { reactiveOmit } from '@vueuse/core'
import { CollapsibleTrigger } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  CollapsibleTriggerProps & { class?: HTMLAttributes['class'] }
>()

const delegatedProps = reactiveOmit(props, 'class')
</script>

<template>
  <!-- The whole header is the trigger. The chevron's turn is the only motion; the summary
       stays on the right, so a closed card still says something. -->
  <CollapsibleTrigger
    data-slot="card-collapsible-trigger"
    v-bind="delegatedProps"
    :class="
      cn(
        'group flex w-full cursor-pointer items-center gap-2.25 bg-muted px-3 py-2.25 text-left text-caption text-foreground-soft data-[state=open]:border-b data-[state=open]:border-secondary data-[state=open]:text-highlight',
        props.class,
      )
    "
  >
    <ChevronRightIcon
      class="size-3 shrink-0 text-subtle-foreground transition-transform duration-panel ease-panel group-data-[state=open]:rotate-90 group-data-[state=open]:text-highlight"
    />
    <span class="flex-1"><slot /></span>
    <span class="text-label text-subtle-foreground"
      ><slot name="summary"
    /></span>
  </CollapsibleTrigger>
</template>
