<script setup lang="ts">
import { ChevronRightIcon } from '@lucide/vue'
import { computed, ref, watch } from 'vue'
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Skeleton } from '@/components/ui/skeleton'
import { VirtualRows } from '@/components/ui/virtual'
import WikiFigure from '@/components/wiki/WikiFigure.vue'
import { WikiFigureSize } from '@/components/wiki/figureSize'
import { useMessages } from '@/i18n'
import type { WikiPageRef } from '@/lib/ipc/types'
import { rowWikiPx } from '@/lib/scale/rows'
import { pageLocation } from '@/lib/wiki/category'
import { emptyList, queryTyped } from '@/lib/facets/emptyList'
import { filterPages } from '@/lib/wiki/listFilter'
import { pageKey } from '@/lib/wiki/pageKey'
import {
  WikiCategory,
  wikiCategoryIcon,
  wikiCategoryTitle,
} from '@/router/routeTable'
import { useTabsStore } from '@/stores/tabs'
import { useWikiStore } from '@/stores/wiki'
import { pageId } from './wikiLabels'

const props = defineProps<{ category: WikiCategory }>()
const wiki = useWikiStore()
const tabs = useTabsStore()
const { t } = useMessages()

// The filter belongs to this screen and to this category: a tab that moves to another
// category starts clean.
const query = ref('')
watch(
  () => props.category,
  () => {
    query.value = ''
  },
)

const all = computed(() => wiki.index?.pages ?? [])
const total = computed(() => filterPages(all.value, props.category, '').length)
const pages = computed(() =>
  filterPages(all.value, props.category, query.value),
)
// An empty list is not always a search that failed: a category with nothing in it says so,
// and offers no button to clear a search nobody typed.
const empty = computed(() =>
  emptyList(total.value, queryTyped(query.value), {
    empty: 'wiki.emptyCategory',
    noResults: 'wiki.noResults',
  }),
)
// No picture on any page is the game's absence, not 900 pages without art.
const noCatalog = computed(
  () => all.value.length > 0 && all.value.every((p) => p.iconUrl === null),
)

const open = (page: WikiPageRef, event: MouseEvent) => {
  const location = pageLocation(page.target)
  if (location === null) return
  if (event.ctrlKey) tabs.open(location)
  else tabs.navigate(location)
}
</script>

<template>
  <!-- The shell's gutter is taken back by the scrolling box and handed to its children, so
       the band can be the full width without overflowing anything (`WikiLanding.vue` says
       what that cost when it was done the other way round). -->
  <div class="-mx-5.5 flex h-full min-h-0 flex-col overflow-hidden pb-5">
    <!-- The same band a page opens with (`WikiHero.vue`), at the size a list deserves: the
         category is the subject here, so it carries the icon, the count, and the filter. -->
    <header
      class="relative flex flex-wrap items-center gap-4 border-b border-hairline hero-wash px-5.5 py-4"
    >
      <span class="pointer-events-none absolute inset-0 hero-grain" />
      <span
        class="relative grid size-wiki-row-figure shrink-0 place-items-center border border-border tile-wash"
      >
        <component
          :is="wikiCategoryIcon[category]"
          class="size-6 text-foreground-soft"
        />
      </span>
      <div class="relative flex min-w-0 flex-1 flex-col gap-0.75">
        <h1 class="text-title text-foreground">
          {{ t(wikiCategoryTitle[category]) }}
        </h1>
        <span class="text-caption text-subtle-foreground tabular-nums"
          >{{ pages.length }} / {{ total }} {{ t('wiki.pages') }}</span
        >
      </div>
      <Input
        :model-value="query"
        :placeholder="t('wiki.search')"
        class="relative w-search"
        @update:model-value="query = String($event)"
      />
    </header>
    <div class="flex min-h-0 flex-1 flex-col gap-3 px-5.5 pt-4">
      <p v-if="noCatalog" class="text-caption text-subtle-foreground">
        {{ t('wiki.noCatalog') }}
      </p>
      <Card v-if="wiki.index" class="min-h-0 flex-1">
        <VirtualRows
          v-if="pages.length > 0"
          v-slot="{ visible }"
          :rows="pages"
          :row-px="rowWikiPx"
        >
          <!-- The banding is the row's position, so a list of 900 keeps a place to rest the
               eye; the hover wins over it, or the row under the pointer would be the only
               one that changes nothing. -->
          <Button
            v-for="{ index, style, row: page } in visible"
            :key="pageKey(page.target) ?? index"
            :variant="ButtonVariant.Ghost"
            :size="ButtonSize.Row"
            :style="style"
            :class="[
              'absolute inset-x-0 top-0 h-row-wiki translate-y-(--row-start) gap-3 border-0 px-3 py-0',
              index % 2 === 1 ? 'bg-row-alt' : 'bg-transparent',
            ]"
            @click="open(page, $event)"
          >
            <WikiFigure
              :target="page.target"
              :url="page.iconUrl"
              :size="WikiFigureSize.Row"
            />
            <span
              class="min-w-0 flex-1 truncate text-left text-body text-foreground"
              >{{ page.title }}</span
            >
            <span
              v-if="pageId(page.target) !== null"
              class="shrink-0 text-micro text-faint-foreground tabular-nums"
              >{{ t('wiki.id') }} {{ pageId(page.target) }}</span
            >
            <ChevronRightIcon class="shrink-0 text-faint-foreground" />
          </Button>
        </VirtualRows>
        <div v-else class="flex flex-col items-start gap-3 p-4">
          <EmptyCategory>{{ t(empty.text) }}</EmptyCategory>
          <Button
            v-if="empty.reset"
            :variant="ButtonVariant.Outline"
            @click="query = ''"
            >{{ t('wiki.resetFilters') }}</Button
          >
        </div>
      </Card>
      <Skeleton v-else class="h-150 w-full" />
    </div>
  </div>
</template>
