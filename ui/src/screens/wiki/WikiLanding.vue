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
import ScreenHeader from '../ScreenHeader.vue'
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
  <div class="flex max-w-250 flex-col gap-4">
    <ScreenHeader
      :icon="tabOriginIcon[TabOrigin.Wiki]"
      :title="t('routes.wiki')"
      >{{ t('wiki.intro') }}</ScreenHeader
    >
    <Alert v-if="info?.kind === 'missing'" :variant="AlertVariant.Destructive">
      <TriangleAlertIcon />
      <AlertTitle>{{ t('wiki.states.missingTitle') }}</AlertTitle>
      <AlertDescription>{{ t('wiki.states.missing') }}</AlertDescription>
    </Alert>
    <template v-else-if="loaded && view">
      <Alert v-if="loaded.gameNewerThanSnapshot === true">
        <InfoIcon />
        <AlertDescription>{{
          t('wiki.provenance.newerGame')
        }}</AlertDescription>
      </Alert>
      <Card>
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
        <CardContent class="grid grid-cols-4 gap-4">
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
      <h2 class="text-control text-highlight">{{ t('wiki.categories') }}</h2>
      <div class="grid grid-cols-3 gap-3">
        <Button
          v-for="category in categories"
          :key="category"
          :variant="ButtonVariant.Outline"
          class="h-auto flex-col items-start gap-1 p-3 [&_svg]:size-5"
          @click="open(category, $event)"
        >
          <component :is="wikiCategoryIcon[category]" />
          <span class="text-control">{{ t(wikiCategoryTitle[category]) }}</span>
          <span class="text-caption text-subtle-foreground tabular-nums"
            >{{ view.count(category) }} {{ t('wiki.pages') }}</span
          >
        </Button>
      </div>
    </template>
    <div v-else class="flex flex-col gap-4">
      <Skeleton class="h-24 w-full" />
      <Skeleton class="h-40 w-full" />
    </div>
  </div>
</template>
