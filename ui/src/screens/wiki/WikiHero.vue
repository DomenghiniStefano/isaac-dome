<script setup lang="ts">
import { computed } from 'vue'
import { Badge, stateBadgeVariant } from '@/components/ui/badge'
import EditionBadge from '@/components/wiki/EditionBadge.vue'
import FactChips from '@/components/wiki/FactChips.vue'
import WikiFigure from '@/components/wiki/WikiFigure.vue'
import WikiInline from '@/components/wiki/WikiInline.vue'
import { FigureSize } from '@/components/wiki/figureSize'
import { progressLines } from '@/lib/wiki/progressLines'
import { useMessages } from '@/i18n'
import type { Entry, Target } from '@/lib/ipc/types'
import { WikiCategory } from '@/router/routeTable'
import { useWikiStore } from '@/stores/wiki'
import { summaryOf } from './heroSummary'
import { kindText, pageId } from '@/lib/wiki/wikiLabels'
import HeroBand from '@/components/screen/HeroBand.vue'

// `entry` is `undefined` while the page is being read and `null` when the dataset lacks it:
// the band is drawn in all three states, because the title, the kind and the figure are the
// index's and not the page's. What the entry adds — the summary, the editions, the quote —
// simply isn't there yet, and the band doesn't collapse when it arrives.
const props = defineProps<{
  target: Target | null
  title: string
  icon: string | null
  category: WikiCategory | null
  entry: Entry | null | undefined
  pageKey: string
  canOpen?: (target: Target) => boolean
}>()
const emit = defineEmits<{ navigate: [target: Target, newTab: boolean] }>()
const { t } = useMessages()
const wiki = useWikiStore()

// The page's own facts (design decision 3), read off the index by the same key `iconFor`
// already uses: `null` for a target the index doesn't list, drawing no chip row at all.
const facts = computed(() =>
  props.target ? wiki.factsFor(props.target) : null,
)

// The save's state for this page (design decision 6), and the lines it earns
// (`progressLines`, shared with `ProgressBadge`'s compact row): `null` — no save chosen, or
// this page's kind carries no state — draws no profile block, never a guessed one.
const progress = computed(() =>
  props.target ? wiki.progressFor(props.target) : null,
)
const profileLines = computed(() =>
  progress.value === null ? [] : progressLines(progress.value),
)

const profileVariant = stateBadgeVariant

// The figure's backdrop: the page's own category accent (design decision 9). A class per
// category, the way `chip/variants.ts` reads a `Tone` — that map is a pill's full shape and
// not exported, and this ring is a different shape (a border and a surface, no text), so it
// is not the same fact read twice (CLAUDE.md, "one definition per concept"): what would be
// duplicated is the *tone*, already named once in `tone.ts`, not this class list.
const categoryAccentClass: Record<WikiCategory, string> = {
  [WikiCategory.Items]:
    'border-category-items-foreground bg-category-items-surface',
  [WikiCategory.Trinkets]:
    'border-category-trinkets-foreground bg-category-trinkets-surface',
  [WikiCategory.Achievements]:
    'border-category-achievements-foreground bg-category-achievements-surface',
  [WikiCategory.Bosses]:
    'border-category-bosses-foreground bg-category-bosses-surface',
  [WikiCategory.Challenges]:
    'border-category-challenges-foreground bg-category-challenges-surface',
  [WikiCategory.Characters]:
    'border-category-characters-foreground bg-category-characters-surface',
  [WikiCategory.Transformations]:
    'border-category-transformations-foreground bg-category-transformations-surface',
  [WikiCategory.Monsters]:
    'border-category-monsters-foreground bg-category-monsters-surface',
  [WikiCategory.CardsAndRunes]:
    'border-category-cards-and-runes-foreground bg-category-cards-and-runes-surface',
  [WikiCategory.Pickups]:
    'border-category-pickups-foreground bg-category-pickups-surface',
  [WikiCategory.Stages]:
    'border-category-stages-foreground bg-category-stages-surface',
  [WikiCategory.Versions]:
    'border-category-versions-foreground bg-category-versions-surface',
}

// The line under the title (`heroSummary.ts`): the entry's description, or on an achievement
// what it unlocks, else what it asks for.
const summary = computed(() =>
  props.entry
    ? summaryOf(props.entry, t('wiki.infobox.unlocks'), wiki.titleOf)
    : [],
)

// The one line of the game's own voice on the page: the pickup quote of an item or a
// trinket, the unlock paper's line of an achievement. Every other kind has none, and no
// other field stands in for it.
const quote = computed(() => {
  const box = props.entry?.infobox
  if (box === undefined) return null
  return box.kind === 'item' ||
    box.kind === 'trinket' ||
    box.kind === 'achievement'
    ? box.quote
    : null
})

const id = computed(() => (props.target ? pageId(props.target) : null))
</script>

<template>
  <!-- The band that opens a page. It is the full width of the page box, because a band that
       stops short of the window reads as a card that happens to be wide. -->
  <HeroBand>
    <div class="relative flex flex-col gap-4 @regular/page:flex-row">
      <!-- The figure's own backdrop is `WikiFigure`'s (paper for a painting, its surface for
           a sprite or a portrait); this ring around it is the page's category, at the hero's
           own scale (design decision 9) — the two washes are not the same thing and do not
           merge into one. -->
      <span
        v-if="target"
        :class="[
          'inline-grid shrink-0 place-items-center border p-2',
          category && categoryAccentClass[category],
        ]"
      >
        <WikiFigure :target="target" :url="icon" :size="FigureSize.Hero" />
      </span>
      <div class="flex min-w-0 flex-1 flex-col gap-2">
        <span
          v-if="category"
          class="text-label tracking-caps text-subtle-foreground uppercase"
          >{{ t(kindText[category]) }}</span
        >
        <h1 class="text-title text-foreground">{{ title }}</h1>
        <!-- `WikiInline` is a fragment — its root is the v-for — so it takes no class of its
             own: the paragraph around it is what sets the measure and the size. -->
        <p v-if="summary.length > 0" class="max-w-200 text-body">
          <WikiInline
            :inline="summary"
            :can-open="canOpen"
            @navigate="(next, newTab) => emit('navigate', next, newTab)"
          />
        </p>
        <!-- The quote is the game's line, not the wiki's prose: it is set apart by a rule
             down its side, the way the game prints it under the pickup. -->
        <p
          v-if="quote && quote.length > 0"
          class="border-l-2 border-band pl-2.5 text-body text-highlight italic"
        >
          <WikiInline :inline="quote" />
        </p>
        <div class="flex flex-wrap items-center gap-2.5 pt-0.5">
          <EditionBadge :dlc="entry?.dlc ?? []" />
          <span
            v-if="id !== null"
            class="text-caption text-faint-foreground tabular-nums"
            >{{ t('wiki.id', { id }) }}</span
          >
          <span
            v-if="entry"
            class="text-caption text-faint-foreground tabular-nums"
            >{{ t('wiki.revision', { revision: entry.revid }) }}</span
          >
          <span
            v-if="entry === null"
            class="text-caption text-faint-foreground tabular-nums"
            >{{ pageKey }}</span
          >
        </div>
        <!-- Every fact the page's own `PageFacts` carries (design decision 3), the same chips
             a card or a table row shows, so a reader who came straight to the page still gets
             them. -->
        <FactChips v-if="facts" :facts="facts" />
        <!-- The save's state for this page (design decision 6), larger than the compact row a
             list badge draws (`ProgressBadge`) and never shown without a save chosen. -->
        <div
          v-if="profileLines.length > 0"
          class="flex flex-wrap items-center gap-2"
        >
          <Badge
            v-for="line in profileLines"
            :key="line.key"
            :variant="profileVariant[line.variant]"
            class="px-3 py-1 text-row"
            >{{ t(line.label.key, line.label.params) }}</Badge
          >
        </div>
      </div>
    </div>
  </HeroBand>
</template>
