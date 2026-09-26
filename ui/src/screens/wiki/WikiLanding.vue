<script setup lang="ts">
import { vScrollMemory } from '@/directives/scrollMemory'
import { InfoIcon, TriangleAlertIcon } from '@lucide/vue'
import { computed } from 'vue'
import { TabOrigin } from '@/lib/shell/tabs'
import { tabOriginIcon } from '@/lib/shell/tabOriginIcon'
import {
  Alert,
  AlertDescription,
  AlertTitle,
  AlertVariant,
} from '@/components/ui/alert'
import { Button, ButtonVariant } from '@/components/ui/button'
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
import {
  RouteName,
  WikiCategory,
  wikiCategoryIcon,
  wikiCategoryTitle,
} from '@/router/routeTable'
import { useTabsStore } from '@/stores/tabs'
import { useWikiStore } from '@/stores/wiki'
import ProfileFact from '@/components/data-state/ProfileFact.vue'
import HeroBand from '@/components/screen/HeroBand.vue'

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
    count: (category: WikiCategory) => fmt.count(value.counts[category]),
  }
})

const categories = Object.values(WikiCategory)

// A category card opens its list in the tab, or beside it with Ctrl, as a sidebar entry does.
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
    <!-- The landing opens on the same band its pages do, and the categories come straight
         under it: what somebody arriving here wants is a way in, not the provenance of the
         dataset — that stays, and it goes last. -->
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
      <Alert
        v-if="info?.kind === 'missing'"
        :variant="AlertVariant.Destructive"
        class="mt-5"
      >
        <TriangleAlertIcon />
        <AlertTitle>{{ t('wiki.states.missingTitle') }}</AlertTitle>
        <AlertDescription>{{ t('wiki.states.missing') }}</AlertDescription>
      </Alert>
      <template v-else-if="loaded && view">
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
          <Button
            v-for="category in categories"
            :key="category"
            :variant="ButtonVariant.Outline"
            class="h-wiki-tile flex-col items-start justify-end gap-1 border-border tile-wash p-3 hover:border-input [&_svg]:size-7"
            @click="open(category, $event)"
          >
            <component
              :is="wikiCategoryIcon[category]"
              class="mb-auto text-foreground-soft"
            />
            <span class="text-heading text-foreground">{{
              t(wikiCategoryTitle[category])
            }}</span>
            <span class="text-caption text-subtle-foreground tabular-nums">{{
              t('wiki.pages', { n: view.count(category) })
            }}</span>
          </Button>
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
      </template>
      <ScreenSkeleton
        v-else
        untitled
        class="pt-5"
        :blocks="[SkeletonBlock.SummaryCard, SkeletonBlock.Card]"
      />
    </div>
  </div>
</template>
