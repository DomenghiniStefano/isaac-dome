<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

// The band a screen opens on: Completion, Goals, the wiki's landing, its lists and its pages.
// The light comes from the corner the picture sits in and falls back into the page, and the
// grain stands in for the shadow the skin does not have; it sits above the wash and below
// everything else, and takes no clicks. The band is the full width of the page box — the
// screen hands it the gutter rather than the band taking it, which is the one thing
// `WikiLanding` paid for the hard way.
//
// What goes inside is the screen's. A child must be `relative` to sit above the grain.
const props = defineProps<{ class?: HTMLAttributes['class'] }>()
</script>

<template>
  <header
    :class="
      cn('relative border-b border-hairline hero-wash px-5.5 py-5', props.class)
    "
  >
    <span class="pointer-events-none absolute inset-0 hero-grain" />
    <slot />
  </header>
</template>
