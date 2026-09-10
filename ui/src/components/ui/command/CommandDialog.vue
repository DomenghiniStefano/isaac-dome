<script setup lang="ts">
import type { DialogRootEmits, DialogRootProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { useForwardPropsEmits } from 'reka-ui'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { cn } from '@/lib/cn'
import Command from './Command.vue'

// Title and description are required: a screen reader announces them, and a default
// would be an English string nobody translated.
const props = defineProps<
  DialogRootProps & {
    title: string
    description: string
    class?: HTMLAttributes['class']
  }
>()
const emits = defineEmits<DialogRootEmits>()

const delegatedProps = reactiveOmit(props, 'title', 'description', 'class')
const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <Dialog v-slot="slotProps" v-bind="forwarded">
    <DialogContent
      :show-close-button="false"
      :class="cn('top-1/3 max-w-lg translate-y-0', props.class)"
    >
      <DialogHeader class="sr-only">
        <DialogTitle>{{ title }}</DialogTitle>
        <DialogDescription>{{ description }}</DialogDescription>
      </DialogHeader>
      <Command class="border-0">
        <slot v-bind="slotProps" />
      </Command>
    </DialogContent>
  </Dialog>
</template>
