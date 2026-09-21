<script setup lang="ts">
import { ref, watch } from 'vue'
import { cn } from '@/lib/cn'
import { ArtSize } from './artSize'

const props = defineProps<{ url: string | null; size: ArtSize }>()

// A drawing that fails to load is drawn as one that isn't there: the placeholder, never a
// broken-image glyph. A new URL tries again.
const failed = ref(false)
watch(
  () => props.url,
  () => {
    failed.value = false
  },
)

const sizeClass: Record<ArtSize, string> = {
  [ArtSize.Thumb]: 'w-achievement-thumb',
  [ArtSize.Card]: 'w-achievement',
  [ArtSize.Hero]: 'w-achievement-hero',
}
</script>

<template>
  <!-- An achievement drawing is dark strokes on transparency: on the dark theme it needs the
       flat mark paper under it, the way the game shows it on a note. The space keeps the
       drawing's ratio, so its arrival doesn't move the row. -->
  <span
    :class="
      cn(
        'grid aspect-achievement shrink-0 place-items-center',
        sizeClass[size],
        url && !failed ? 'bg-mark-paper' : 'hatch-placeholder',
      )
    "
  >
    <img
      v-if="url && !failed"
      :src="url"
      alt=""
      class="size-full object-contain"
      @error="failed = true"
    />
  </span>
</template>
