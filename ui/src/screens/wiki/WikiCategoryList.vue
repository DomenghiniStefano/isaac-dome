<script setup lang="ts">
import { useVirtualizer } from '@tanstack/vue-virtual'
import { computed, ref, watch } from 'vue'
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Card, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Skeleton } from '@/components/ui/skeleton'
import WikiFigure from '@/components/wiki/WikiFigure.vue'
import { WikiFigureSize } from '@/components/wiki/figureSize'
import { useMessages } from '@/i18n'
import type { WikiPageRef } from '@/lib/ipc/types'
import { pageLocation } from '@/lib/wiki/category'
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
import { wikiRowHeight } from './wikiLayout'

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

const scroller = ref<HTMLElement | null>(null)

// Up to some 900 pages in a category, drawn as many as fit plus a margin, the same rule as
// Unlock and the Collection.
const virtualizer = useVirtualizer(
  computed(() => ({
    count: pages.value.length,
    getScrollElement: () => scroller.value,
    estimateSize: () => wikiRowHeight,
    overscan: 8,
  })),
)
const visible = computed(() =>
  virtualizer.value.getVirtualItems().flatMap((entry) => {
    const page = pages.value[entry.index]
    return page ? [{ entry, page }] : []
  }),
)
const body = computed(() => ({
  '--unlock-total': `${virtualizer.value.getTotalSize()}px`,
}))
const rowStart = (start: number) => ({ '--row-start': `${start}px` })
</script>

<template>
  <div class="flex max-w-250 flex-col gap-4">
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
      <div
        v-if="pages.length > 0"
        ref="scroller"
        class="max-h-unlock-body overflow-auto"
      >
        <div :style="body" class="relative h-(--unlock-total)">
          <Button
            v-for="{ entry, page } in visible"
            :key="pageKey(page.target) ?? entry.index"
            :variant="ButtonVariant.Ghost"
            :size="ButtonSize.Row"
            :style="rowStart(entry.start)"
            class="absolute inset-x-0 top-0 h-row-wide translate-y-(--row-start) gap-3 border-0 border-b border-hairline px-3 py-0"
            @click="open(page, $event)"
          >
            <WikiFigure
              :target="page.target"
              :url="page.iconUrl"
              :size="WikiFigureSize.Thumb"
            />
            <span class="min-w-0 flex-1 truncate text-row text-foreground">{{
              page.title
            }}</span>
            <span
              v-if="pageId(page.target) !== null"
              class="shrink-0 text-micro text-faint-foreground tabular-nums"
              >{{ t('wiki.id') }} {{ pageId(page.target) }}</span
            >
          </Button>
        </div>
      </div>
      <div v-else class="flex flex-col items-start gap-3 p-4">
        <EmptyCategory>{{ t('wiki.noResults') }}</EmptyCategory>
        <Button :variant="ButtonVariant.Outline" @click="query = ''">{{
          t('wiki.resetFilters')
        }}</Button>
      </div>
    </Card>
    <Skeleton v-else class="h-150 w-full" />
  </div>
</template>
