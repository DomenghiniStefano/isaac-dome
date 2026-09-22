<script setup lang="ts">
// Three boxes, each its own `page` container, so a compact layout can be judged without
// resizing the window — the thing container queries buy and media queries could not (spec
// 3.13a §11). The widths straddle all three thresholds: 500 is below compact (844), 900 is past
// it and below regular (1004), and 1350 is past wide (1324).
const widths = [500, 900, 1350]
</script>

<template>
  <div class="flex items-start gap-4 overflow-x-auto">
    <div
      v-for="width in widths"
      :key="width"
      :style="{ '--kit-width': `${width}px` }"
      class="@container/page flex h-80 w-(--kit-width) shrink-0 flex-col border border-border"
    >
      <slot />
    </div>
  </div>
</template>
