<script setup lang="ts">
import { computed } from 'vue'
import { Badge } from '@/components/ui/badge'
import QualityPips from '@/components/data-state/QualityPips.vue'
import WikiFigure from '@/components/wiki/WikiFigure.vue'
import WikiInline from '@/components/wiki/WikiInline.vue'
import { WikiFigureSize } from '@/components/wiki/figureSize'
import { dlcNames } from '@/lib/wiki/dlcNames'
import { useMessages } from '@/i18n'
import type { Entry, Target } from '@/lib/ipc/types'
import { Dlc } from '@/lib/ipc/types'
import type { WikiCategory } from '@/router/routeTable'
import { useWikiStore } from '@/stores/wiki'
import { summaryOf } from './heroSummary'
import { kindText, pageId } from '@/lib/wiki/wikiLabels'

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
  iconFor?: (target: Target) => string | null
  canOpen?: (target: Target) => boolean
}>()
const emit = defineEmits<{ navigate: [target: Target, newTab: boolean] }>()
const { t } = useMessages()
const wiki = useWikiStore()

// The editions the infobox states, in release order. An empty list is the wiki's "no
// restriction" and NOT "it exists nowhere" (the `dlc` field's own doc), so nothing is drawn
// rather than a badge saying none.
const editions = computed(() =>
  Object.values(Dlc)
    .filter((dlc) => props.entry?.dlc.includes(dlc) === true)
    .map((dlc) => ({ dlc, name: dlcNames[dlc] })),
)

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

// Quality is the item's alone: `-1..=4` in the catalog, and the pips draw `null` as a dash.
const quality = computed(() => {
  const box = props.entry?.infobox
  return box?.kind === 'item' ? box.quality : null
})

const id = computed(() => (props.target ? pageId(props.target) : null))
</script>

<template>
  <!-- The band that opens a page (card #57). It is the full width of the page box, because a
       band that stops short of the window reads as a card that happens to be wide; the
       screen around it hands it the gutter rather than the band taking it (`WikiPage.vue`).
       The grain sits above the wash and below everything else, and takes no clicks. -->
  <header class="relative border-b border-hairline hero-wash px-5.5 py-5">
    <span class="pointer-events-none absolute inset-0 hero-grain" />
    <div class="relative flex flex-col gap-4 @regular/page:flex-row">
      <WikiFigure
        v-if="target"
        :target="target"
        :url="icon"
        :size="WikiFigureSize.Hero"
      />
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
            :icon-for="iconFor"
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
          <Badge v-for="edition in editions" :key="edition.dlc">{{
            edition.name
          }}</Badge>
          <QualityPips v-if="quality !== null" :quality="quality" />
          <span
            v-if="id !== null"
            class="text-caption text-faint-foreground tabular-nums"
            >{{ t('wiki.id') }} {{ id }}</span
          >
          <span
            v-if="entry"
            class="text-caption text-faint-foreground tabular-nums"
            >{{ t('wiki.revision') }} {{ entry.revid }}</span
          >
          <span
            v-if="entry === null"
            class="text-caption text-faint-foreground tabular-nums"
            >{{ pageKey }}</span
          >
        </div>
      </div>
    </div>
  </header>
</template>
