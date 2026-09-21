<script setup lang="ts">
import { InfoIcon, TriangleAlertIcon } from '@lucide/vue'
import { computed } from 'vue'
import { TabOrigin } from '@/components/shell/tabs'
import { tabOriginIcon } from '@/components/shell/tabOriginIcon'
import {
  Alert,
  AlertDescription,
  AlertTitle,
  AlertVariant,
} from '@/components/ui/alert'
import { Button, ButtonVariant } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { i18n, useMessages } from '@/i18n'
import { formatCount } from '@/lib/profile/profileView'
import {
  RouteName,
  WikiCategory,
  wikiCategoryIcon,
  wikiCategoryTitle,
} from '@/router/routeTable'
import { useTabsStore } from '@/stores/tabs'
import { useWikiStore } from '@/stores/wiki'
import ProfileFact from '../profile/ProfileFact.vue'

const wiki = useWikiStore()
const tabs = useTabsStore()
const { t } = useMessages()

const info = computed(() => wiki.index?.info ?? null)
const loaded = computed(() =>
  info.value?.kind === 'loaded' ? info.value : null,
)

// The provenance in words: the snapshot's day, the patch the wiki knew, the totals.
const view = computed(() => {
  const locale = i18n.global.locale.value
  const value = loaded.value
  if (!value) return null
  return {
    snapshot: new Intl.DateTimeFormat(locale, {
      day: 'numeric',
      month: 'short',
      year: 'numeric',
    }).format(new Date(value.snapshotAt)),
    patch: value.lastKnownPatch
      ? `${value.lastKnownPatch.number} · ${value.lastKnownPatch.date}`
      : t('wiki.provenance.patchUnknown'),
    unresolved: formatCount(value.unresolved, locale),
    unknownTemplates: formatCount(value.unknownTemplates, locale),
    count: (category: WikiCategory) =>
      formatCount(value.counts[category], locale),
  }
})

const categories = Object.values(WikiCategory)

// A category card opens its list in the tab, or beside it with Ctrl, as a sidebar entry does.
const open = (category: WikiCategory, event: MouseEvent) => {
  const location = { name: RouteName.Wiki, query: { category } }
  if (event.ctrlKey) tabs.open(location)
  else tabs.navigate(location)
}
</script>

<template>
  <!-- The gutter is moved, not sold off: the shell puts `px-5.5` on the page box, and a band
       that reached the window's edge by growing past it opened a horizontal scrollbar under
       the whole screen (seen 2026-09-21). So the scrolling box takes the gutter back with
       `-mx-5.5` and hands it to its children instead — the band then simply is the full
       width, and nothing overflows anything. -->
  <div class="-mx-5.5 flex h-full flex-col overflow-y-auto pb-15">
    <!-- The landing opens on the same band its pages do, and the categories come straight
         under it: what somebody arriving here wants is a way in, not the provenance of the
         dataset — that stays, and it goes last. -->
    <header
      class="relative flex items-center gap-4 border-b border-hairline hero-wash px-5.5 py-5"
    >
      <span class="pointer-events-none absolute inset-0 hero-grain" />
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
    </header>
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
            <span class="text-caption text-subtle-foreground tabular-nums"
              >{{ view.count(category) }} {{ t('wiki.pages') }}</span
            >
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
      <div v-else class="flex flex-col gap-4 pt-5">
        <Skeleton class="h-24 w-full" />
        <Skeleton class="h-40 w-full" />
      </div>
    </div>
  </div>
</template>
