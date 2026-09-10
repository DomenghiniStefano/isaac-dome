<script setup lang="ts">
defineProps<{ title: string }>()
</script>

<template>
  <section class="flex flex-col gap-3 border border-border bg-sheet p-4">
    <h2 class="text-control text-highlight">{{ title }}</h2>
    <slot />
  </section>
</template>
