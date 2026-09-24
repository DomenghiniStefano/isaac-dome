<script setup lang="ts">
import type { Component } from 'vue'
import { HelpTip } from '@/components/ui/tooltip'

// The screen says what it is in its title; what it is *for* is one hover away. The slot is the
// same one the screens have always passed — the paragraph moved into the tooltip here, so none
// of the fourteen callers changed. A screen that passes nothing (Placeholder) gets no mark.
defineProps<{ icon: Component; title: string; eyebrow?: string }>()
</script>

<template>
  <header class="flex flex-col gap-2">
    <span v-if="eyebrow" class="text-caption text-subtle-foreground">{{
      eyebrow
    }}</span>
    <div class="flex items-center gap-3">
      <component :is="icon" class="size-6 shrink-0 text-foreground-soft" />
      <h1 class="text-title text-foreground">{{ title }}</h1>
      <HelpTip v-if="$slots.default"><slot /></HelpTip>
    </div>
  </header>
</template>
