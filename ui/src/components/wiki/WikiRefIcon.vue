<script setup lang="ts">
import { computed } from 'vue'
import { assertNever } from '@/lib/assertNever'
import { cn } from '@/lib/cn'
import type { Target } from '@/lib/ipc/types'
import { RefArt, RefIconSize, refArt } from './refIcon'

const props = defineProps<{ src: string; target: Target; size: RefIconSize }>()

// A drawing keeps its 263 x 176 ratio on the mark paper, as `AchievementArt` draws it: squeezed
// into a square on the dark page it was dark strokes on dark, and unreadable. A sprite stays
// pixel art; at the name size it is the whole `--spacing-sprite`, a whole multiple of its 32px,
// and the drawing beside it is the table rows' thumbnail, the same 32 tall.
const frame = computed((): string => {
  const art = refArt(props.target)
  switch (art) {
    case RefArt.Drawing:
      return cn(
        'aspect-achievement bg-mark-paper',
        props.size === RefIconSize.Name ? 'w-achievement-thumb' : 'h-4',
      )
    case RefArt.Sprite:
      return props.size === RefIconSize.Name ? 'size-sprite' : 'size-4'
    default:
      return assertNever(art)
  }
})
</script>

<template>
  <span
    :class="
      cn(
        'mr-1 inline-grid place-items-center',
        size === RefIconSize.Name ? 'align-middle' : 'align-text-bottom',
        frame,
      )
    "
    ><img
      :src="src"
      alt=""
      :class="
        cn(
          'size-full object-contain',
          refArt(target) === RefArt.Sprite && 'pixelated',
        )
      "
  /></span>
</template>
