<script setup lang="ts">
import { computed } from 'vue'
import AchievementArt from '@/components/graph/AchievementArt.vue'
import { ArtSize } from '@/components/graph/artSize'
import PixelSprite from '@/components/sprite/PixelSprite.vue'
import { assertNever } from '@/lib/assertNever'
import { cn } from '@/lib/cn'
import type { Target } from '@/lib/ipc/types'
import { categoryOf } from '@/lib/wiki/category'
import { wikiCategoryIcon } from '@/router/routeTable'
import type { WikiCategory } from '@/router/routeTable'
import { FigureBoxPx, SpriteNativePx, integerScale } from './figureScale'
import { FigureSize } from './figureSize'

const props = defineProps<{
  target: Target | null
  url: string | null
  size: FigureSize
  /**
   * The category to read the fallback icon from, when the caller already knows it precisely
   * — a landing tile or the hero's mosaic (card #90, decision 8), which draw a category's own
   * representative picture and not one page's. `categoryOf(target)` (the default below) is an
   * approximation for two kinds: an `entity` always reads as Bosses, so it cannot tell a
   * Monsters tile from a Bosses one, and an `article` reads as no category at all, so it
   * cannot resolve the four article-based tiles (cards & runes, pickups, stages, versions).
   * Decides the fallback icon only — the frame (sprite, painting or portrait) still comes from
   * `target` alone, unchanged for every caller that doesn't pass this.
   */
  category?: WikiCategory
}>()

// One figure component, everywhere a wiki picture is drawn (card #90, decision 2): the
// background, the centring and the scale are decided once, here, by what the game draws for
// that kind (DESIGN-BRIEF.md §8) — a 32px sprite scaled up, a painted achievement at its own
// ratio on the mark paper, or a portrait.
const Frame = {
  Sprite: 'sprite',
  Painting: 'painting',
  Portrait: 'portrait',
} as const
type Frame = (typeof Frame)[keyof typeof Frame]

// `target === null` only ever reaches here from a landing sample with no representative page
// at all (`CategorySample.target`, `wiki_samples.rs`): `url` is always null then too, so no
// frame is ever actually drawn — Portrait is picked for its box, and nothing else reads it.
const frame = computed((): Frame => {
  if (props.target === null) return Frame.Portrait
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
    case 'article':
      return Frame.Portrait
    default:
      return assertNever(props.target)
  }
})

// Without the game, or without a picture at all (a transformation, a stage, a version
// article), the figure falls back to a category icon — never a hole (Review Focus 2). The
// caller's own `category` wins when given (see its own doc comment for why); otherwise
// `categoryOf(target)`, which returns `null` for the four kinds it cannot resolve from the
// target alone (`lib/wiki/category.ts`'s own doc) or when there is no target at all — for
// those there is no category icon to fall back to, and the plain placeholder is drawn
// instead, same as everywhere else in the app.
const fallbackIcon = computed(() => {
  const category =
    props.category ?? (props.target ? categoryOf(props.target) : null)
  return category ? wikiCategoryIcon[category] : null
})
// Half of the box, at every size: proportional to the picture it stands in for, never a
// small glyph lost in a large frame (the owner, on a `Hero`-sized boss with no picture: the
// icon read as stuck in a corner, not centred and not sized like the thing it replaces).
const fallbackIconClass: Record<FigureSize, string> = {
  [FigureSize.Row]: 'size-8',
  [FigureSize.Card]: 'size-16',
  [FigureSize.Tile]: 'size-20',
  [FigureSize.Hero]: 'size-24',
}

const artSize: Record<FigureSize, ArtSize> = {
  [FigureSize.Row]: ArtSize.Row,
  [FigureSize.Card]: ArtSize.Card,
  [FigureSize.Tile]: ArtSize.Tile,
  [FigureSize.Hero]: ArtSize.Hero,
}

// The frame every size shares: centred on both axes, in a box lit the way a tile or a band
// already is (`tile-wash`) or flat like the data beside it (`bg-data`, the card grid's own
// picture). A portrait fills it; a sprite and a painting sit inside it at their own scale,
// letterboxed.
const box: Record<FigureSize, string> = {
  [FigureSize.Row]: 'size-figure-row border-hairline tile-wash',
  [FigureSize.Card]: 'size-figure-card border-hairline bg-data',
  [FigureSize.Tile]: 'size-figure-tile border-border tile-wash',
  [FigureSize.Hero]: 'size-figure-hero border-border tile-wash',
}
const portrait: Record<FigureSize, string> = {
  [FigureSize.Row]: 'size-figure-row',
  [FigureSize.Card]: 'size-figure-card',
  [FigureSize.Tile]: 'size-figure-tile',
  [FigureSize.Hero]: 'size-figure-hero',
}

// A pixel sprite is drawn at the largest *integer* multiple of its own 32px that still fits
// the box (`figureScale.ts`): the four boxes are whole multiples of 32 for exactly this
// reason, so this always fills the box edge to edge. The value is computed, so it travels as
// a CSS variable bound from the template, consumed by a `size-*` utility — never a
// hand-written pixel (`docs/frontend-conventions.md`, "Dynamic values: CSS variables, not
// inline pixels").
const spriteSizePx = computed(
  () => SpriteNativePx * integerScale(SpriteNativePx, FigureBoxPx[props.size]),
)
</script>

<template>
  <!-- One figure per page: a missing one is the category icon, never a broken image and
       never another page's picture. Every size gets the same frame, so a picture reads as
       the same object in a row, a card and a band. -->
  <span :class="cn('grid shrink-0 place-items-center border', box[size])">
    <AchievementArt
      v-if="frame === Frame.Painting"
      :url="url"
      :size="artSize[size]"
    >
      <template v-if="fallbackIcon" #fallback>
        <component
          :is="fallbackIcon"
          :class="cn(fallbackIconClass[size], 'text-foreground-soft')"
        />
      </template>
    </AchievementArt>
    <PixelSprite
      v-else-if="frame === Frame.Sprite"
      :url="url"
      placeholder
      :style="{ '--figure-sprite-size': `${spriteSizePx}px` }"
      class="size-(--figure-sprite-size)"
    >
      <template v-if="fallbackIcon" #fallback>
        <component
          :is="fallbackIcon"
          :class="cn(fallbackIconClass[size], 'text-foreground-soft')"
        />
      </template>
    </PixelSprite>
    <PixelSprite v-else :url="url" placeholder :class="portrait[size]">
      <template v-if="fallbackIcon" #fallback>
        <component
          :is="fallbackIcon"
          :class="cn(fallbackIconClass[size], 'text-foreground-soft')"
        />
      </template>
    </PixelSprite>
  </span>
</template>
