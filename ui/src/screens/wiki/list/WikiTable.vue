<script setup lang="ts">
import { ArrowDownIcon, ArrowUpIcon } from '@lucide/vue'
import { computed } from 'vue'
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import QueueActionButton from '@/components/plan/QueueActionButton.vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { GridTable } from '@/components/ui/grid-table'
import EditionBadge from '@/components/wiki/EditionBadge.vue'
import ProgressBadge from '@/components/wiki/ProgressBadge.vue'
import WikiFigure from '@/components/wiki/WikiFigure.vue'
import { FigureSize } from '@/components/wiki/figureSize'
import { useMessages } from '@/i18n'
import type { WikiPageRef } from '@/lib/ipc/types'
import type { ScrollOffset } from '@/lib/scale/scrollOffset'
import { wikiColumns } from '@/lib/table/columns'
import { RowHeight } from '@/lib/table/gridColumn'
import { factColumns, shownValue } from '@/lib/wiki/factChips'
import { categoryHasId } from '@/lib/wiki/listFacets'
import type { WikiSortKey } from '@/lib/wiki/listFacets'
import { SortDirection } from '@/lib/wiki/listSort'
import type { WikiSortSpec } from '@/lib/wiki/listSort'
import { pageKey } from '@/lib/wiki/pageKey'
import { pageId } from '@/lib/wiki/wikiLabels'
import { pageQueueTarget } from '@/lib/wiki/wikiQueue'
import type { WikiCategory } from '@/router/routeTable'
import { useWikiStore } from '@/stores/wiki'

// The table: the picture, the id, the name, the edition, every fact column `factColumns` gives
// this category — the same table `FactChips` reads for the card grid's chips, one definition
// either view draws from — and the save's state. Sorting is the filter bar's own control
// (`listFacets.ts`'s `wikiSortOrder`); this only marks which column that sort is currently
// reading, with an arrow. A click on a row opens its page; the title is the button the
// keyboard reaches, since the row cannot be one while its Actions cell holds another.
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

const facts = computed(() => factColumns(props.category))
const columns = computed(() =>
  wikiColumns(facts.value, categoryHasId(props.category)),
)
const sortArrow = (key: WikiSortKey) =>
  key === props.sort.key
    ? props.sort.direction === SortDirection.Asc
      ? ArrowUpIcon
      : ArrowDownIcon
    : null

// What a cell shows: the column's word, translated, or its value as it is (`shownValue`).
const cellText = (page: WikiPageRef, key: WikiSortKey): string | null => {
  const column = facts.value.find((c) => c.key === key)
  const shown = column ? shownValue(column, page) : null
  if (shown === null) return null
  return shown.kind === 'message' ? t(shown.key) : shown.text
}

const open = (page: WikiPageRef, event: MouseEvent): void => {
  event.stopPropagation()
  emit('open', page, event)
}
</script>

<template>
  <GridTable
    :columns="columns"
    :rows="pages"
    :row-key="(page) => pageKey(page.target) ?? page.title"
    :row-height="RowHeight.Wiki"
    virtual
    clickable
    :offset="offset"
    @offset-change="emit('offsetChange', $event)"
    @row-click="(page, event) => emit('open', page, event)"
  >
    <template #head-edition>
      <span class="flex min-w-0 items-center gap-1">
        <span class="truncate">{{ t('wiki.list.sort.edition') }}</span>
        <component
          :is="sortArrow('edition')"
          v-if="sortArrow('edition')"
          class="size-3 shrink-0"
        />
      </span>
    </template>
    <template v-for="fact in facts" :key="fact.key" #[`head-fact-${fact.key}`]>
      <span class="flex min-w-0 items-center gap-1">
        <span class="truncate">{{ t(fact.label) }}</span>
        <component
          :is="sortArrow(fact.key)"
          v-if="sortArrow(fact.key)"
          class="size-3 shrink-0"
        />
      </span>
    </template>
    <template #cell-figure="{ row }">
      <WikiFigure
        :target="row.target"
        :url="row.iconUrl"
        :size="FigureSize.Row"
      />
    </template>
    <template #cell-id="{ row }">
      <span class="truncate text-faint-foreground tabular-nums">{{
        pageId(row.target)
      }}</span>
    </template>
    <template #cell-name="{ row }">
      <Button
        :variant="ButtonVariant.RefQuiet"
        :size="ButtonSize.Compact"
        class="max-w-full justify-start text-body"
        @click="open(row, $event)"
        ><span class="min-w-0 truncate">{{ row.title }}</span></Button
      >
    </template>
    <template #cell-edition="{ row }">
      <EditionBadge :dlc="row.dlc" />
    </template>
    <template
      v-for="fact in facts"
      :key="fact.key"
      #[`cell-fact-${fact.key}`]="{ row }"
    >
      <EmptyValue v-if="cellText(row, fact.key) === null">{{
        t('wiki.infobox.none')
      }}</EmptyValue>
      <span v-else class="truncate tabular-nums">{{
        cellText(row, fact.key)
      }}</span>
    </template>
    <template #cell-profile="{ row }">
      <ProgressBadge :progress="wiki.progressFor(row.target)" compact />
    </template>
    <template #actions="{ row }">
      <QueueActionButton
        :target="pageQueueTarget(row.target, wiki.progressFor(row.target))"
      />
    </template>
  </GridTable>
</template>
