<script setup lang="ts">
import type { TabsContentProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { TabsContent } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  TabsContentProps & { class?: HTMLAttributes['class'] }
>()

const delegatedProps = reactiveOmit(props, 'class')
</script>

<template>
  <!-- The panel takes the sheet's edge, never a radius, and has no top edge of its own:
       the list's underline is it. -->
  <TabsContent
    data-slot="tabs-content"
    v-bind="delegatedProps"
    :class="
      cn(
        'border border-t-0 border-border bg-data p-3 text-body text-foreground-soft',
        props.class,
      )
    "
  >
    <slot />
  </TabsContent>
</template>
