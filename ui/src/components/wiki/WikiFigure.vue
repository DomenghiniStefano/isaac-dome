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
  [WikiFigureSize.Row]: ArtSize.Thumb,
  [WikiFigureSize.Card]: ArtSize.Card,
  [WikiFigureSize.Hero]: ArtSize.Hero,
}

// The box a sprite or a portrait is set in. A thumbnail has none — it sits bare in whatever
// holds it — and the other three are the same frame at three sizes, so a page's figure is
// recognisably one thing from a list row to the band that opens the page.
const box: Record<WikiFigureSize, string | null> = {
  [WikiFigureSize.Thumb]: null,
  [WikiFigureSize.Row]: 'size-wiki-row-figure border-hairline tile-wash',
  [WikiFigureSize.Card]: 'size-wiki-figure border-hairline bg-data',
  [WikiFigureSize.Hero]: 'size-wiki-hero border-border tile-wash',
}

// What the picture is drawn at inside that box. A portrait fills the frame; a sprite keeps
// the game's whole multiple of 32px — except in a list row, whose 48px frame is smaller than
// that multiple already is at scale 200. There the sprite stays at its native 32, which is
// still a whole multiple and still crisp, rather than being cropped by the frame around it.
const portrait: Record<WikiFigureSize, string> = {
  [WikiFigureSize.Thumb]: 'size-8',
  [WikiFigureSize.Row]: 'size-wiki-row-figure',
  [WikiFigureSize.Card]: 'size-wiki-figure',
  [WikiFigureSize.Hero]: 'size-wiki-hero',
}
const sprite: Record<WikiFigureSize, string> = {
  [WikiFigureSize.Thumb]: 'size-8',
  [WikiFigureSize.Row]: 'size-8',
  [WikiFigureSize.Card]: 'size-sprite',
  [WikiFigureSize.Hero]: 'size-sprite',
}
</script>

<template>
  <!-- One figure per page: a missing one is the hatch placeholder, never a broken image
       and never another page's picture. A thumbnail sits bare in its row; every other size
       gets the frame, which is what makes the same picture read as the same object in a
       list, in a header and on the page's own band. -->
  <AchievementArt
    v-if="frame === Frame.Painting"
    :url="url"
    :size="artSize[size]"
  />
  <PixelSprite
    v-else-if="box[size] === null"
    :url="url"
    placeholder
    class="size-8 shrink-0"
  />
  <span
    v-else
    :class="cn('grid shrink-0 place-items-center border', box[size])"
  >
    <PixelSprite
      :url="url"
      placeholder
      :class="cn(frame === Frame.Sprite ? sprite[size] : portrait[size])"
    />
  </span>
</template>
