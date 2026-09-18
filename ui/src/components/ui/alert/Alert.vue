<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'
import type { AlertLive, AlertVariant } from './variants'
import { AlertLive as Live, alertVariants } from './variants'

const props = withDefaults(
  defineProps<{
    class?: HTMLAttributes['class']
    variant?: AlertVariant
    live?: AlertLive
  }>(),
  { class: undefined, variant: undefined, live: Live.Polite },
)
</script>

<template>
  <div
    data-slot="alert"
    :role="props.live"
    :class="cn(alertVariants({ variant }), props.class)"
  >
    <slot />
  </div>
</template>
