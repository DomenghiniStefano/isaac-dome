<script setup lang="ts">
import { vScrollMemory } from '@/directives/scrollMemory'
import { InfoIcon, TriangleAlertIcon } from '@lucide/vue'
import { computed } from 'vue'
import {
  Alert,
  AlertDescription,
  AlertTitle,
  AlertVariant,
} from '@/components/ui/alert'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import ScreenSkeleton from '@/components/data-state/ScreenSkeleton.vue'
import { SkeletonBlock } from '@/components/data-state/skeletonBlock'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { useFormat } from '@/composables/useFormat'
import { useMessages } from '@/i18n'
import { RouteName, WikiCategory } from '@/router/routeTable'
import { useTabsStore } from '@/stores/tabs'
import { useWikiStore } from '@/stores/wiki'
import type { CategorySample } from '@/lib/ipc/types'
import ProfileFact from '@/components/data-state/ProfileFact.vue'
import HeroBand from '@/components/screen/HeroBand.vue'
import { TabOrigin } from '@/lib/shell/tabs'
import { tabOriginIcon } from '@/lib/shell/tabOriginIcon'
import { categoryProgress, overallProgress } from '@/lib/wiki/progress'
import WikiLandingHero from './landing/WikiLandingHero.vue'
import WikiLandingTile from './landing/WikiLandingTile.vue'

const wiki = useWikiStore()
const tabs = useTabsStore()
const { t } = useMessages()
const fmt = useFormat()

const info = computed(() => wiki.index?.info ?? null)
const loaded = computed(() =>
  info.value?.kind === 'loaded' ? info.value : null,
)

// The provenance in words: the snapshot's day, the patch the wiki knew, the totals.
const view = computed(() => {
  const value = loaded.value
  if (!value) return null
  return {
    snapshot: fmt.date(new Date(value.snapshotAt)),
    patch: value.lastKnownPatch
      ? `${value.lastKnownPatch.number} · ${value.lastKnownPatch.date}`
      : t('wiki.provenance.patchUnknown'),
    unresolved: fmt.count(value.unresolved),
    unknownTemplates: fmt.count(value.unknownTemplates),
    count: (category: WikiCategory) => value.counts[category],
  }
})

const categories = Object.values(WikiCategory)

// The ground truth for "how many pages the wiki has" (design decision 8b's hero totals): the
// index's own array, never a hand-summed total over `WikiCounts` — some of its fields are
// subsets of others (`cardsAndRunes`, `pickups`, `stages`, `versions` are all `articles`),
// so adding them would count a page more than once.
const totalPages = computed(() => wiki.index?.pages.length ?? 0)

const overall = computed(() =>
  wiki.index ? overallProgress(wiki.index.pages, wiki.progressFor) : null,
)

const categoryProgressOf = (category: WikiCategory) =>
  wiki.index
    ? categoryProgress(wiki.index.pages, category, wiki.progressFor)
    : null

// A tile's own picture, when the game gives one (design decision 5): a representative row
// the backend chose deliberately (`ipc::category_sample`), never "whichever page happens to
// be first". `undefined` before the index has loaded; `WikiFigure` reads a sample with no
// `iconUrl` — no game, or the kind has no picture at all (transformations, stages, version
// articles) — as its cue to fall back to the category's plain icon.
const sampleOf = (category: WikiCategory): CategorySample | undefined =>
  wiki.index?.samples.find((s) => s.category === category)

// A category tile opens its list in the tab, or beside it with Ctrl, as a sidebar entry does.
const open = (category: WikiCategory, event: MouseEvent) =>
  tabs.go({ name: RouteName.Wiki, query: { category } }, event.ctrlKey)
</script>

<template>
  <!-- The gutter is the children's, not this box's: the band is the full width of the page
       and the sections under it carry `px-5.5`. A band that reached the window's edge by
       growing past a gutter opened a horizontal scrollbar under the whole screen, which is
       why the shell's page box pads nothing. -->
  <div
    v-scroll-memory="'page'"
    class="flex h-full flex-col overflow-y-auto pb-15"
  >
    <!-- The wiki failed entirely (a corrupted embedded dataset): the plain band still names
         the screen, and everything else is the alert below explaining why there is nothing
         to open. The enriched hero (mosaic, totals, the overall bar) only draws once there
         is a dataset to draw it from. -->
    <template v-if="info?.kind === 'missing'">
      <HeroBand class="flex items-center gap-4">
        <component
          :is="tabOriginIcon[TabOrigin.Wiki]"
          class="relative size-8 shrink-0 text-foreground-soft"
        />
        <div class="relative flex min-w-0 flex-col gap-1.5">
          <h1 class="text-title text-foreground">{{ t('routes.wiki') }}</h1>
          <p class="max-w-200 text-body text-subtle-foreground">
            {{ t('wiki.intro') }}
          </p>
        </div>
      </HeroBand>
      <div class="flex flex-col px-5.5">
        <Alert :variant="AlertVariant.Destructive" class="mt-5">
          <TriangleAlertIcon />
          <AlertTitle>{{ t('wiki.states.missingTitle') }}</AlertTitle>
          <AlertDescription>{{ t('wiki.states.missing') }}</AlertDescription>
        </Alert>
      </div>
    </template>
    <template v-else-if="loaded && view">
      <!-- The landing opens on a hero (design decision 8b): the mosaic, the title, the
           totals and, with a save, the overall progress. The categories come straight
           under it — what somebody arriving here wants is a way in — and the provenance
           of the dataset goes last. -->
      <WikiLandingHero
        :samples="wiki.index?.samples ?? []"
        :total-pages="totalPages"
        :snapshot="view.snapshot"
        :patch="view.patch"
        :overall="overall"
      />
      <div class="flex flex-col px-5.5">
        <Alert v-if="loaded.gameNewerThanSnapshot === true" class="mt-5">
          <InfoIcon />
          <AlertDescription>{{
            t('wiki.provenance.newerGame')
          }}</AlertDescription>
        </Alert>
        <h2
          class="pt-5 pb-3 text-control tracking-caps text-highlight uppercase"
        >
          {{ t('wiki.categories') }}
        </h2>
        <div class="grid grid-cols-2 gap-3 @regular/page:grid-cols-4">
          <WikiLandingTile
            v-for="category in categories"
            :key="category"
            :category="category"
            :sample="sampleOf(category)"
            :page-count="view.count(category)"
            :progress="categoryProgressOf(category)"
            @open="open(category, $event)"
          />
        </div>
        <Card class="mt-6">
          <CardHeader>
            <CardTitle>{{ t('wiki.provenance.title') }}</CardTitle>
            <Tooltip>
              <TooltipTrigger as-child>
                <span
                  class="flex cursor-help items-center gap-1.5 text-caption text-subtle-foreground"
                  tabindex="0"
                  ><InfoIcon class="size-3" />{{
                    t('wiki.provenance.license')
                  }}</span
                >
              </TooltipTrigger>
              <TooltipContent class="max-w-80 text-caption text-foreground">{{
                t('wiki.provenance.licenseLong')
              }}</TooltipContent>
            </Tooltip>
          </CardHeader>
          <CardContent class="grid grid-cols-2 gap-4 @regular/page:grid-cols-4">
            <ProfileFact
              :label="t('wiki.provenance.snapshot')"
              :value="view.snapshot"
            />
            <ProfileFact
              :label="t('wiki.provenance.patch')"
              :value="view.patch"
            />
            <ProfileFact
              :label="t('wiki.provenance.unresolved')"
              :value="view.unresolved"
            />
            <ProfileFact
              :label="t('wiki.provenance.unknownTemplates')"
              :value="view.unknownTemplates"
            />
          </CardContent>
        </Card>
      </div>
    </template>
    <ScreenSkeleton
      v-else
      untitled
      class="px-5.5 pt-5"
      :blocks="[SkeletonBlock.SummaryCard, SkeletonBlock.Card]"
    />
  </div>
</template>
