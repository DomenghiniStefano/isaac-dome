<script setup lang="ts">
import { ref } from 'vue'
import QualityPips from '@/components/data-state/QualityPips.vue'
import QueueActionButton from '@/components/plan/QueueActionButton.vue'
import PixelSprite from '@/components/sprite/PixelSprite.vue'
import { GridTable } from '@/components/ui/grid-table'
import { useMessages } from '@/i18n'
import { CollectionFacet } from '@/lib/collection/collectionFacets'
import { collectionFacetValueLabel } from '@/lib/collection/collectionLabels'
import { itemQueueTarget } from '@/lib/collection/collectionQueue'
import type { CollectionItem } from '@/lib/ipc/types'
import { collectionColumns } from '@/lib/table/columns'
import CollectionNameCell from './CollectionNameCell.vue'
import CollectionPoolsCell from './CollectionPoolsCell.vue'
import CollectionStateCell from './CollectionStateCell.vue'

defineProps<{
  items: CollectionItem[]
  /** What the find bar is looking for, so a row can paint it. */
  findQuery: string
  /** The id of the match the bar is standing on. */
  findCurrent: string | null
}>()
const { t } = useMessages()

const origin = (item: CollectionItem): string =>
  item.origin
    ? collectionFacetValueLabel(t, CollectionFacet.Origin, item.origin)
    : '—'

// The find bar hands back an index; moving there is the virtualizer's job, reached through the
// table. Structural and not `InstanceType`: `GridTable` is generic, so it has no instance type
// to take — and the one thing wanted from it is the one call named here.
const table = ref<{ scrollToIndex: (index: number) => void } | null>(null)
defineExpose({
  scrollToIndex: (index: number) => table.value?.scrollToIndex(index),
})
</script>

<template>
  <!-- The screen scrolls as a page (`PageScroll`): the rows virtualize against it, and the
       columns' header pins to its top while the list goes by under it. -->
  <GridTable
    ref="table"
    :columns="collectionColumns"
    :rows="items"
    :row-key="(item) => item.id"
    virtual
  >
    <template #cell-sprite="{ row }">
      <PixelSprite :url="row.iconUrl" placeholder class="size-8 shrink-0" />
    </template>
    <template #cell-item="{ row }">
      <CollectionNameCell
        :item="row"
        :find-query="findQuery"
        :find-current="String(row.id) === findCurrent"
      />
    </template>
    <template #cell-quality="{ row }">
      <QualityPips :quality="row.quality" />
    </template>
    <template #cell-pools="{ row }">
      <CollectionPoolsCell :item="row" />
    </template>
    <template #cell-origin="{ row }">
      <span class="truncate text-caption text-foreground-soft">{{
        origin(row)
      }}</span>
    </template>
    <template #cell-state="{ row }">
      <CollectionStateCell :item="row" />
    </template>
    <template #actions="{ row }">
      <QueueActionButton :target="itemQueueTarget(row)" />
    </template>
  </GridTable>
</template>
