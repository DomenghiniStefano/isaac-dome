<script setup lang="ts">
import type {
  DialogRootEmits,
  DialogRootProps,
  ListboxRootEmits,
} from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { useForwardPropsEmits } from 'reka-ui'
import { useTemplateRef } from 'vue'
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
    // Both belong to the Command inside: the palette never mounts one itself.
    filter?: boolean
    search?: string
    keepSearch?: boolean
  }
>()
// `highlight` and `highlightItem` are the Command's, relayed: the dialog is a wrapper, and a
// caller that mounts the palette must not have to reach past it to know which row is under
// the keyboard.
const emits = defineEmits<
  DialogRootEmits & {
    'update:search': [value: string]
    highlight: ListboxRootEmits['highlight']
  }
>()

const command = useTemplateRef('command')
defineExpose({
  highlightItem: (value: string) => command.value?.highlightItem(value),
})

const delegatedProps = reactiveOmit(
  props,
  'title',
  'description',
  'class',
  'filter',
  'search',
  'keepSearch',
)
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
      <Command
        ref="command"
        class="border-0"
        :filter="props.filter"
        :search="props.search"
        :keep-search="props.keepSearch"
        @update:search="emits('update:search', $event)"
        @highlight="emits('highlight', $event)"
      >
        <slot v-bind="slotProps" />
      </Command>
    </DialogContent>
  </Dialog>
</template>
