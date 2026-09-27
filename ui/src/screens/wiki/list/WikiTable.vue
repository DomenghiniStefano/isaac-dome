<script setup lang="ts">
import { ArrowDownIcon, ArrowUpIcon } from '@lucide/vue'
import { computed } from 'vue'
import EditionBadge from '@/components/wiki/EditionBadge.vue'
import ProgressBadge from '@/components/wiki/ProgressBadge.vue'
import WikiFigure from '@/components/wiki/WikiFigure.vue'
import { FigureSize } from '@/components/wiki/figureSize'
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { VirtualRows } from '@/components/ui/virtual'
import { useMessages } from '@/i18n'
import type { WikiPageRef } from '@/lib/ipc/types'
import { factColumns } from '@/lib/wiki/factChips'
import { categoryHasId } from '@/lib/wiki/listFacets'
import type { WikiSortKey } from '@/lib/wiki/listFacets'
import { SortDirection } from '@/lib/wiki/listSort'
import type { WikiSortSpec } from '@/lib/wiki/listSort'
import { rowWikiPx } from '@/lib/scale/rows'
import type { ScrollOffset } from '@/lib/scale/scrollOffset'
import { pageId } from '@/lib/wiki/wikiLabels'
import { pageKey } from '@/lib/wiki/pageKey'
import type { WikiCategory } from '@/router/routeTable'
import { useWikiStore } from '@/stores/wiki'

// The table: the picture, the name, the id, the edition, every fact
// column `factColumns` gives this category — the same table `FactChips` reads for the card
// grid's chips, one definition either view draws from — and the save's state. Sorting is the
// filter bar's own control (`listFacets.ts`'s `wikiSortOrder`); this only marks which column
// that sort is currently reading, with an arrow.
const props = defineProps<{
  pages: WikiPageRef[]
  category: WikiCategory
  sort: WikiSortSpec
  offset: ScrollOffset | null
}>()
const emit = defineEmits<{
  offsetChange: [ScrollOffset]
  open: [page: WikiPageRef, event: MouseEvent]
}>()
const { t } = useMessages()
const wiki = useWikiStore()

const columns = computed(() => factColumns(props.category))
const hasId = computed(() => categoryHasId(props.category))
const sortArrow = (key: WikiSortKey) =>
  key === props.sort.key
    ? props.sort.direction === SortDirection.Asc
      ? ArrowUpIcon
      : ArrowDownIcon
    : null

const cellText = (page: WikiPageRef, key: WikiSortKey): string | null => {
  const column = columns.value.find((c) => c.key === key)
  const value = column?.value(page)
  return value === null || value === undefined ? null : String(value)
}
</script>

<template>
  <div class="flex flex-col">
    <div
      class="sticky top-0 z-raised-header flex items-center gap-3 border-b border-hairline bg-band px-3 py-1.5 text-label text-band-foreground"
    >
      <span class="w-figure-row shrink-0" />
      <span
        v-if="hasId"
        class="w-wiki-table-id shrink-0 truncate @max-compact/page:hidden"
        >{{ t('wiki.list.sort.id') }}</span
      >
      <span class="min-w-0 flex-1 truncate">{{
        t('wiki.list.sort.name')
      }}</span>
      <span
        class="flex w-wiki-table-edition shrink-0 items-center gap-1 truncate @max-compact/page:hidden"
      >
        {{ t('wiki.list.sort.edition') }}
        <component
          :is="sortArrow('edition')"
          v-if="sortArrow('edition')"
          class="size-3"
        />
      </span>
      <span
        v-for="(column, at) in columns"
        :key="column.key"
        :class="[
          'flex w-wiki-table-fact shrink-0 items-center gap-1 truncate',
          at > 0 && '@max-compact/page:hidden',
        ]"
      >
        {{ t(column.label) }}
        <component
          :is="sortArrow(column.key)"
          v-if="sortArrow(column.key)"
          class="size-3"
        />
      </span>
      <span class="w-wiki-table-fact shrink-0 truncate">{{
        t('wiki.list.facet.profile')
      }}</span>
    </div>
    <VirtualRows
      v-if="pages.length > 0"
      v-slot="{ visible }"
      :rows="pages"
      :row-px="rowWikiPx"
      :offset="offset"
      @offset-change="emit('offsetChange', $event)"
    >
      <Button
        v-for="{ index, style, row: page } in visible"
        :key="pageKey(page.target) ?? index"
        :variant="ButtonVariant.Ghost"
        :size="ButtonSize.Row"
        :style="style"
        :class="[
          'absolute inset-x-0 top-0 h-row-wiki translate-y-(--row-start) items-center gap-3 border-0 px-3 py-0',
          index % 2 === 1 ? 'bg-row-alt' : 'bg-transparent',
        ]"
        @click="emit('open', page, $event)"
      >
        <WikiFigure
          :target="page.target"
          :url="page.iconUrl"
          :size="FigureSize.Row"
        />
        <span
          v-if="hasId"
          class="w-wiki-table-id shrink-0 truncate text-faint-foreground tabular-nums @max-compact/page:hidden"
          >{{ pageId(page.target) }}</span
        >
        <span
          class="min-w-0 flex-1 truncate text-left text-body text-foreground"
          >{{ page.title }}</span
        >
        <span
          class="w-wiki-table-edition shrink-0 truncate @max-compact/page:hidden"
        >
          <EditionBadge :dlc="page.dlc" />
        </span>
        <span
          v-for="(column, at) in columns"
          :key="column.key"
          :class="[
            'w-wiki-table-fact shrink-0 truncate text-left',
            at > 0 && '@max-compact/page:hidden',
          ]"
        >
          <EmptyValue v-if="cellText(page, column.key) === null">{{
            t('wiki.infobox.none')
          }}</EmptyValue>
          <span v-else class="tabular-nums">{{
            cellText(page, column.key)
          }}</span>
        </span>
        <span class="w-wiki-table-fact shrink-0 truncate">
          <ProgressBadge :progress="wiki.progressFor(page.target)" compact />
        </span>
      </Button>
    </VirtualRows>
  </div>
</template>
