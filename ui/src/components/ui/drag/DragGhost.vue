<script setup lang="ts">
import type { GhostBox } from '@/lib/drag/dragList'

defineProps<{ box: GhostBox }>()
</script>

<template>
  <!-- The lifted copy, above everything and out of the pointer's way: the original stays dimmed
       in its slot and the marker keeps naming the landing, so what the eye follows and what
       decides the drop are the same row twice.
       The skin is flat by decision (`assets/theme/shadow.css`: `--shadow-*: initial`), so the
       lift is a 2px primary border on an opaque sheet, never an invented shadow. Position and
       size are CSS variables bound here — convention 1's one exception to "no inline style". -->
  <Teleport to="body">
    <div
      class="pointer-events-none fixed top-(--drag-top) left-(--drag-left) z-50 h-(--drag-height) w-(--drag-width) overflow-hidden border-2 border-primary bg-sheet"
      :style="{
        '--drag-left': `${box.left}px`,
        '--drag-top': `${box.top}px`,
        '--drag-width': `${box.width}px`,
        '--drag-height': `${box.height}px`,
      }"
    >
      <slot />
    </div>
  </Teleport>
</template>
