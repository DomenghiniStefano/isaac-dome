<script setup lang="ts">
import { CheckIcon } from '@lucide/vue'
import { computed, ref, watch } from 'vue'
import { assertNever } from '@/lib/assertNever'
import { cn } from '@/lib/cn'
import type { Target } from '@/lib/ipc/types'
import { RefArt, refArt } from './nameList'

// `done`: the profile has it, and the picture wears a tick in its corner — seen while scrolling
// down a list, before the badge beside the name is read.
const props = defineProps<{
  src: string | null
  target: Target
  done?: boolean
}>()

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
// them stay in line. A drawing is `--spacing-wiki-name-art` wide on the mark paper: its dark strokes
// on the dark page were unreadable. A sprite is `--spacing-sprite`, a whole multiple of
// its own 32px, so the pixel art lands on whole pixels.
const frame = computed((): string => {
  switch (art.value) {
    case RefArt.Drawing:
      return 'aspect-achievement w-wiki-name-art'
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
        'relative grid shrink-0 place-items-center',
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
    <span
      v-if="done"
      class="absolute -right-1 -bottom-1 grid size-4 place-items-center border border-state-done bg-state-done-surface text-state-done-foreground"
      ><CheckIcon class="size-3"
    /></span>
  </span>
</template>
