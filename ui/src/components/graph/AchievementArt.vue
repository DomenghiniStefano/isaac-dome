<script setup lang="ts">
import { ref, useSlots, watch } from 'vue'
import { cn } from '@/lib/cn'
import { ArtSize } from './artSize'

const props = defineProps<{ url: string | null; size: ArtSize }>()
const slots = useSlots()

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
  // `WikiFigure`'s four sizes: the painting is as wide as its own frame
  // (`--spacing-figure-*`), one token for both rather than a second scale of its own.
  [ArtSize.Row]: 'w-figure-row',
  [ArtSize.Card]: 'w-figure-card',
  [ArtSize.Tile]: 'w-figure-tile',
  [ArtSize.Hero]: 'w-figure-hero',
}
</script>

<template>
  <!-- An achievement drawing is dark strokes on transparency: on the dark theme it needs the
       flat mark paper under it, the way the game shows it on a note. The space keeps the
       drawing's ratio, so its arrival doesn't move the row. Without a picture, the hatch
       reads as "unknown" the way every other placeholder in the app does — unless the
       caller gave its own fallback (`WikiFigure`'s category icon), which draws on the same
       plain box instead: the hatch and a second glyph would say "unknown" twice. -->
  <span
    :class="
      cn(
        'grid aspect-achievement shrink-0 place-items-center',
        sizeClass[size],
        url && !failed
          ? 'bg-mark-paper'
          : slots.fallback === undefined && 'hatch-placeholder',
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
    <slot v-else name="fallback" />
  </span>
</template>
