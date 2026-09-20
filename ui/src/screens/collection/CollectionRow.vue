<script setup lang="ts">
import { computed } from 'vue'
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import FindHighlight from '@/components/find/FindHighlight.vue'
import { unlockKindText } from '@/components/graph/unlockKindText'
import PixelSprite from '@/components/sprite/PixelSprite.vue'
import WhyMenu from '@/components/graph/WhyMenu.vue'
import { Badge, BadgeVariant } from '@/components/ui/badge'
import { useMessages } from '@/i18n'
import { CollectionFacet } from '@/lib/collection/collectionFacets'
import { ItemState, itemState } from '@/lib/collection/itemState'
import { lockWhy } from '@/lib/graph/whyMenu'
import type { CollectionItem } from '@/lib/ipc/types'
import QualityPips from './QualityPips.vue'
import { collectionFacetValueLabel, itemStateText } from './collectionLabels'

const props = defineProps<{
  item: CollectionItem
  /** What the find bar is looking for; empty while it is closed (B67). */
  findQuery: string
  /** Whether this row is the match the bar is standing on. */
  findCurrent: boolean
}>()
const { t } = useMessages()

const state = computed(() => itemState(props.item))

const variant: Record<ItemState, BadgeVariant> = {
  [ItemState.InCollection]: BadgeVariant.Done,
  [ItemState.Available]: BadgeVariant.Now,
  [ItemState.Locked]: BadgeVariant.Blocked,
  [ItemState.Unknown]: BadgeVariant.Unknown,
}

const subtitle = computed(
  () =>
    `${t('collection.id')} ${props.item.id} · ${t(unlockKindText[props.item.kind])}`,
)
const firstPool = computed(() => props.item.pools[0] ?? null)
const morePools = computed(() => Math.max(0, props.item.pools.length - 1))
const origin = computed(() =>
  props.item.origin
    ? collectionFacetValueLabel(t, CollectionFacet.Origin, props.item.origin)
    : null,
)

// The achievement behind the item, as the badge's menu: one group, one entry, and the page
// that says how that achievement is earned.
const groups = computed(() => lockWhy(props.item.lock, t))
</script>

<template>
  <!-- One row of the Collection's grid: the six cells are the grid's children, so this component
       has no root of its own. -->
  <span class="flex justify-center">
    <PixelSprite :url="item.iconUrl" placeholder class="size-8 shrink-0" />
  </span>
  <span class="flex min-w-0 flex-col px-2">
    <span class="truncate text-row text-foreground"
      ><FindHighlight
        :text="item.name"
        :query="findQuery"
        :current="findCurrent"
    /></span>
    <span class="text-micro text-faint-foreground tabular-nums">{{
      subtitle
    }}</span>
  </span>
  <span class="px-2">
    <QualityPips :quality="item.quality" />
  </span>
  <span class="flex min-w-0 items-center gap-2 px-2">
    <template v-if="firstPool">
      <span class="truncate text-caption text-foreground">{{ firstPool }}</span>
      <span
        v-if="morePools > 0"
        class="shrink-0 text-label text-subtle-foreground tabular-nums"
        >+{{ morePools }}</span
      >
    </template>
    <EmptyValue v-else>{{ t('collection.poolNone') }}</EmptyValue>
  </span>
  <span class="truncate px-2 text-caption text-foreground-soft">{{
    origin ?? '—'
  }}</span>
  <span class="px-2">
    <WhyMenu :groups="groups" :label="t('collection.lockedBy')">
      <Badge
        :variant="variant[state]"
        :tabindex="groups.length > 0 ? 0 : undefined"
        >{{ t(itemStateText[state]) }}</Badge
      >
    </WhyMenu>
  </span>
</template>
