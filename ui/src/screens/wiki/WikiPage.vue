<script setup lang="ts">
import { InfoIcon } from '@lucide/vue'
import { computed, watch } from 'vue'
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import { Badge } from '@/components/ui/badge'
import { Button, ButtonVariant } from '@/components/ui/button'
import { Skeleton } from '@/components/ui/skeleton'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import WikiFigure from '@/components/wiki/WikiFigure.vue'
import { WikiFigureSize } from '@/components/wiki/figureSize'
import { useMessages } from '@/i18n'
import type { Target } from '@/lib/ipc/types'
import { categoryOf, pageLocation } from '@/lib/wiki/category'
import { parsePageKey } from '@/lib/wiki/pageKey'
import { RouteName } from '@/router/routeTable'
import type { WikiCategory } from '@/router/routeTable'
import { useTabsStore } from '@/stores/tabs'
import { useWikiStore } from '@/stores/wiki'
import WikiInfobox from './WikiInfobox.vue'
import WikiSections from './WikiSections.vue'
import { kindText } from './wikiLabels'

const props = defineProps<{
  pageKey: string
  category: WikiCategory | null
}>()
const wiki = useWikiStore()
const tabs = useTabsStore()
const { t } = useMessages()

// The key is the tab's; a key that doesn't parse is a page the dataset doesn't know, the
// same state as a valid key with no entry (spec 3.5, Decision 1).
const target = computed(() => parsePageKey(props.pageKey))
watch(
  target,
  (value) => {
    if (value) void wiki.loadEntry(value)
  },
  { immediate: true },
)

// `undefined` while the page is being read; `null` when the dataset lacks it.
const entry = computed(() =>
  target.value === null ? null : wiki.entry(props.pageKey),
)
const unknown = computed(() => entry.value === null)
const category = computed(() =>
  target.value ? categoryOf(target.value) : props.category,
)
const title = computed(
  () =>
    entry.value?.title ??
    wiki.titleOf(props.pageKey) ??
    t('wiki.states.unknownTitle'),
)
const icon = computed(() => (target.value ? wiki.iconFor(target.value) : null))

// A reference replaces the page in this tab, or opens one beside it with Ctrl: the same
// action as opening a search result (DESIGN-BRIEF.md §4.2).
const onNavigate = (next: Target, newTab: boolean) => {
  const location = pageLocation(next)
  if (location === null) return
  if (newTab) tabs.open(location)
  else tabs.navigate(location)
}
const back = () => {
  if (category.value)
    tabs.navigate({ name: RouteName.Wiki, query: { category: category.value } })
}
</script>

<template>
  <div class="flex max-w-250 flex-col gap-5">
    <header class="flex items-start gap-4">
      <WikiFigure
        v-if="target"
        :target="target"
        :url="icon"
        :size="WikiFigureSize.Card"
      />
      <div class="flex min-w-0 flex-col gap-2">
        <h1 class="text-title text-foreground">{{ title }}</h1>
        <div class="flex flex-wrap items-center gap-2.5">
          <Badge v-if="category">{{ t(kindText[category]) }}</Badge>
          <span
            v-if="entry"
            class="text-caption text-subtle-foreground tabular-nums"
            >{{ t('wiki.revision') }} {{ entry.revid }}</span
          >
          <span
            v-if="unknown"
            class="text-caption text-faint-foreground tabular-nums"
            >{{ pageKey }}</span
          >
        </div>
        <Tooltip>
          <TooltipTrigger as-child>
            <span
              class="flex w-fit cursor-help items-center gap-1.5 text-caption text-subtle-foreground"
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
      </div>
    </header>
    <template v-if="unknown">
      <EmptyCategory
        >{{ t('wiki.states.unknown') }}
        {{ t('wiki.states.unknownHint') }}</EmptyCategory
      >
      <Button
        v-if="category"
        :variant="ButtonVariant.Outline"
        class="w-fit"
        @click="back"
        >{{ t('wiki.back') }}</Button
      >
    </template>
    <div v-else-if="entry === undefined" class="flex flex-col gap-4">
      <Skeleton class="h-24 w-full" />
      <Skeleton class="h-40 w-full" />
      <Skeleton class="h-40 w-full" />
    </div>
    <template v-else-if="entry">
      <WikiInfobox
        :entry="entry"
        :icon-for="wiki.iconFor"
        :can-open="wiki.hasPage"
        @navigate="onNavigate"
      />
      <WikiSections
        :sections="entry.sections"
        :icon-for="wiki.iconFor"
        :can-open="wiki.hasPage"
        @navigate="onNavigate"
      />
    </template>
  </div>
</template>
