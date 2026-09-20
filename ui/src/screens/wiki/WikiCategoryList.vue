<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Card, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Skeleton } from '@/components/ui/skeleton'
import { VirtualRows } from '@/components/ui/virtual'
import WikiFigure from '@/components/wiki/WikiFigure.vue'
import { WikiFigureSize } from '@/components/wiki/figureSize'
import { useMessages } from '@/i18n'
import type { WikiPageRef } from '@/lib/ipc/types'
import { rowWidePx } from '@/lib/scale/rows'
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
import ScreenHeader from '../ScreenHeader.vue'
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
  <div class="flex h-full flex-col gap-4 overflow-y-auto pt-5 pb-15">
    <ScreenHeader
      :icon="wikiCategoryIcon[category]"
      :title="t(wikiCategoryTitle[category])"
      :eyebrow="`${total} ${t('wiki.pages')}`"
    />
    <p v-if="noCatalog" class="text-caption text-subtle-foreground">
      {{ t('wiki.noCatalog') }}
    </p>
    <Card v-if="wiki.index">
      <CardHeader class="flex-wrap">
        <CardTitle class="tabular-nums"
          >{{ pages.length }} / {{ total }} {{ t('wiki.pages') }}</CardTitle
        >
        <Input
          :model-value="query"
          :placeholder="t('wiki.search')"
          class="w-search"
          @update:model-value="query = String($event)"
        />
      </CardHeader>
      <VirtualRows
        v-if="pages.length > 0"
        v-slot="{ visible }"
        :rows="pages"
        :row-px="rowWidePx"
      >
        <Button
          v-for="{ index, style, row: page } in visible"
          :key="pageKey(page.target) ?? index"
          :variant="ButtonVariant.Ghost"
          :size="ButtonSize.Row"
          :style="style"
          class="absolute inset-x-0 top-0 h-row-wide translate-y-(--row-start) gap-3 border-0 border-b border-hairline px-3 py-0"
          @click="open(page, $event)"
        >
          <WikiFigure
            :target="page.target"
            :url="page.iconUrl"
            :size="WikiFigureSize.Thumb"
          />
          <span
            class="min-w-0 flex-1 truncate text-left text-row text-foreground"
            >{{ page.title }}</span
          >
          <span
            v-if="pageId(page.target) !== null"
            class="shrink-0 text-micro text-faint-foreground tabular-nums"
            >{{ t('wiki.id') }} {{ pageId(page.target) }}</span
          >
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
</template>
