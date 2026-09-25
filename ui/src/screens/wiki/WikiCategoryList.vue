<script setup lang="ts">
import { ChevronRightIcon } from '@lucide/vue'
import ListEmptyState from '@/components/data-state/ListEmptyState.vue'
import { computed } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Skeleton } from '@/components/ui/skeleton'
import { VirtualRows } from '@/components/ui/virtual'
import WikiFigure from '@/components/wiki/WikiFigure.vue'
import { WikiFigureSize } from '@/components/wiki/figureSize'
import { useTabView } from '@/composables/useTabView'
import { useMessages } from '@/i18n'
import type { WikiPageRef } from '@/lib/ipc/types'
import { rowWikiPx } from '@/lib/scale/rows'
import { emptyList, queryTyped } from '@/lib/facets/emptyList'
import { filterPages } from '@/lib/wiki/listFilter'
import type { ScrollOffset } from '@/lib/scale/scrollOffset'
import { pageKey } from '@/lib/wiki/pageKey'
import {
  WikiCategory,
  wikiCategoryIcon,
  wikiCategoryTitle,
} from '@/router/routeTable'
import { useTabsStore } from '@/stores/tabs'
import { useWikiStore } from '@/stores/wiki'
import { wikiView } from './tabView'
import { pageId } from '@/lib/wiki/wikiLabels'
import HeroBand from '@/components/screen/HeroBand.vue'

const props = defineProps<{ category: WikiCategory }>()
const wiki = useWikiStore()
const tabs = useTabsStore()
const { t } = useMessages()

// The filter and the position are the entry's reading (`tabView.ts`): they come back after a tab
// switch, a back, a tear-off. A tab that moves to another category is a new entry, so it starts
// clean without anything here having to clear it.
const { reading, update } = useTabView(wikiView)
const query = computed(() => reading.value.query)
const setQuery = (value: string) => update({ query: value })
const setOffset = (offset: ScrollOffset) => update({ offset })

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

const open = (page: WikiPageRef, event: MouseEvent) =>
  tabs.openPage(page.target, event.ctrlKey)
</script>

<template>
  <!-- The gutter is the children's, so the band can be the full width without overflowing
       anything (`WikiLanding.vue` says what that cost when it was done the other way round). -->
  <div class="flex h-full min-h-0 flex-col overflow-hidden pb-5">
    <!-- The same band a page opens with (`WikiHero.vue`), at the size a list deserves: the
         category is the subject here, so it carries the icon, the count, and the filter. -->
    <HeroBand class="flex flex-wrap items-center gap-4 py-4">
      <span
        class="relative grid size-wiki-row-figure shrink-0 place-items-center border border-border tile-wash"
      >
        <component
          :is="wikiCategoryIcon[category]"
          class="size-6 text-foreground-soft"
        />
      </span>
      <div class="relative flex min-w-0 flex-1 flex-col gap-1">
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
        @update:model-value="setQuery(String($event))"
      />
    </HeroBand>
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
          :offset="reading.offset"
          @offset-change="setOffset"
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
        <ListEmptyState
          v-else
          :empty="empty"
          :reset-text="'wiki.resetFilters'"
          @reset="setQuery('')"
        />
      </Card>
      <Skeleton v-else class="h-150 w-full" />
    </div>
  </div>
</template>
