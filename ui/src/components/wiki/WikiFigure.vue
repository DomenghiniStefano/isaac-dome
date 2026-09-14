<script setup lang="ts">
import { computed } from 'vue'
import AchievementArt from '@/components/graph/AchievementArt.vue'
import { ArtSize } from '@/components/graph/artSize'
import PixelSprite from '@/components/sprite/PixelSprite.vue'
import { assertNever } from '@/lib/assertNever'
import { cn } from '@/lib/cn'
import type { Target } from '@/lib/ipc/types'
import { WikiFigureSize } from './figureSize'

const props = defineProps<{
  target: Target
  url: string | null
  size: WikiFigureSize
}>()

// How a page's figure is framed, by what the game draws for that kind (DESIGN-BRIEF.md
// §8): a 32px sprite scaled up, a painted achievement at its own ratio, a portrait.
const Frame = {
  Sprite: 'sprite',
  Painting: 'painting',
  Portrait: 'portrait',
} as const
type Frame = (typeof Frame)[keyof typeof Frame]

const frame = computed((): Frame => {
  switch (props.target.kind) {
    case 'item':
    case 'trinket':
      return Frame.Sprite
    case 'achievement':
    case 'challenge':
      return Frame.Painting
    case 'entity':
    case 'character':
    case 'stage':
    case 'room':
    case 'concept':
    case 'transformation':
      return Frame.Portrait
    default:
      return assertNever(props.target)
  }
})

const artSize: Record<WikiFigureSize, ArtSize> = {
  [WikiFigureSize.Thumb]: ArtSize.Thumb,
  [WikiFigureSize.Card]: ArtSize.Card,
}
</script>

<template>
  <!-- One figure per page: a missing one is the hatch placeholder, never a broken image
       and never another page's picture. A thumbnail sits bare in its row; the card's
       figure gets the data frame the export draws behind it. -->
  <AchievementArt
    v-if="frame === Frame.Painting"
    :url="url"
    :size="artSize[size]"
  />
  <PixelSprite
    v-else-if="size === WikiFigureSize.Thumb"
    :url="url"
    placeholder
    class="size-8 shrink-0"
  />
  <span
    v-else
    class="grid size-wiki-figure shrink-0 place-items-center border border-hairline bg-data"
  >
    <PixelSprite
      :url="url"
      placeholder
      :class="cn(frame === Frame.Sprite ? 'size-sprite' : 'size-wiki-figure')"
    />
  </span>
</template>
