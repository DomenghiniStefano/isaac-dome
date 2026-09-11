<script setup lang="ts">
defineProps<{ label: string; value: string }>()
</script>

<template>
  <div class="flex flex-col gap-0.75">
    <span class="text-label text-subtle-foreground">{{ label }}</span>
    <span class="text-row text-foreground">{{ value }}</span>
  </div>
</template>
