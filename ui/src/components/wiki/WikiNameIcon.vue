<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { assertNever } from '@/lib/assertNever'
import { cn } from '@/lib/cn'
import type { Target } from '@/lib/ipc/types'
import { RefArt, refArt } from './nameList'

const props = defineProps<{ src: string | null; target: Target }>()

// A picture that fails to load is drawn as one that isn't there: the placeholder, never a
// broken-image glyph. A new address tries again.
const failed = ref(false)
watch(
  () => props.src,
  () => {
    failed.value = false
  },
)

const art = computed(() => refArt(props.target))

// One size per kind of art, whether the picture is there or not, so the names beside a column of
// them stay in line. A drawing is the table rows' thumbnail, 48 x 32 on the mark paper: its dark
// strokes on the dark page were unreadable. A sprite is `--spacing-sprite`, a whole multiple of
// its own 32px, so the pixel art lands on whole pixels.
const frame = computed((): string => {
  switch (art.value) {
    case RefArt.Drawing:
      return 'aspect-achievement w-achievement-thumb'
    case RefArt.Sprite:
      return 'size-sprite'
    default:
      return assertNever(art.value)
  }
})
const shown = computed(() => props.src !== null && !failed.value)
</script>

<template>
  <span
    :class="
      cn(
        'grid shrink-0 place-items-center',
        frame,
        !shown && 'hatch-placeholder',
        shown && art === RefArt.Drawing && 'bg-mark-paper',
      )
    "
  >
    <img
      v-if="shown"
      :src="src ?? undefined"
      alt=""
      :class="
        cn('size-full object-contain', art === RefArt.Sprite && 'pixelated')
      "
      @error="failed = true"
    />
  </span>
</template>
