<script setup lang="ts">
import type { DialogContentEmits, DialogContentProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { XIcon } from '@lucide/vue'
import { reactiveOmit } from '@vueuse/core'
import {
  DialogClose,
  DialogContent,
  DialogPortal,
  useForwardPropsEmits,
} from 'reka-ui'
import { useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import DialogOverlay from './DialogOverlay.vue'

defineOptions({
  inheritAttrs: false,
})

const props = withDefaults(
  defineProps<
    DialogContentProps & {
      class?: HTMLAttributes['class']
      showCloseButton?: boolean
    }
  >(),
  {
    showCloseButton: true,
  },
)
const emits = defineEmits<DialogContentEmits>()

const delegatedProps = reactiveOmit(props, 'class', 'showCloseButton')
const forwarded = useForwardPropsEmits(delegatedProps, emits)

const { t } = useMessages()
</script>

<template>
  <!-- The sheet rises in five steps (Motion.dc.html, --m-sheet). The close control sits on
       the header band, so its colour is the band's text. -->
  <DialogPortal>
    <DialogOverlay />
    <DialogContent
      data-slot="dialog-content"
      v-bind="{ ...$attrs, ...forwarded }"
      :class="
        cn(
          'fixed top-1/2 left-1/2 z-50 flex w-full max-w-md -translate-x-1/2 -translate-y-1/2 animate-sheet-rise flex-col border border-input bg-sheet text-foreground',
          props.class,
        )
      "
    >
      <slot />
      <DialogClose
        v-if="showCloseButton"
        data-slot="dialog-close"
        class="absolute top-1.5 right-2 grid size-5 cursor-pointer place-items-center text-band-foreground [&_svg]:size-3.5"
      >
        <XIcon />
        <span class="sr-only">{{ t('ui.close') }}</span>
      </DialogClose>
    </DialogContent>
  </DialogPortal>
</template>
