<script setup lang="ts">
import type { DialogOverlayProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { DialogOverlay } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  DialogOverlayProps & { class?: HTMLAttributes['class'] }
>()

const delegatedProps = reactiveOmit(props, 'class')
</script>

<template>
  <!-- Solid dim, no blur: the alpha lives in the overlay token. -->
  <DialogOverlay
    data-slot="dialog-overlay"
    v-bind="delegatedProps"
    :class="cn('fixed inset-0 z-overlay bg-overlay', props.class)"
  >
    <slot />
  </DialogOverlay>
</template>
