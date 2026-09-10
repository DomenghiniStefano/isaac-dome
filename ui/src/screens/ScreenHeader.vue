<script setup lang="ts">
import type { Component } from 'vue'

defineProps<{ icon: Component; title: string; eyebrow?: string }>()
</script>

<template>
  <header class="flex flex-col gap-2">
    <span v-if="eyebrow" class="text-caption text-subtle-foreground">{{
      eyebrow
    }}</span>
    <div class="flex items-center gap-2.75">
      <component :is="icon" class="size-6 shrink-0 text-foreground-soft" />
      <h1 class="text-title text-foreground">{{ title }}</h1>
    </div>
    <p v-if="$slots.default" class="max-w-155 text-row text-foreground-soft">
      <slot />
    </p>
  </header>
</template>
