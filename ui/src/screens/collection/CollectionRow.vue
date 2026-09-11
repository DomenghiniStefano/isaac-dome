<script setup lang="ts">
import { computed } from 'vue'
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import { unlockKindText } from '@/components/graph/unlockKindText'
import PixelSprite from '@/components/sprite/PixelSprite.vue'
import { Badge, BadgeVariant } from '@/components/ui/badge'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { useMessages } from '@/i18n'
import { assertNever } from '@/lib/assertNever'
import { CollectionFacet } from '@/lib/collection/collectionFilter'
import { ItemState, itemState } from '@/lib/collection/itemState'
import type { CollectionItem } from '@/lib/ipc/types'
import QualityPips from './QualityPips.vue'
import { collectionFacetValueLabel, itemStateText } from './collectionLabels'

const props = defineProps<{ item: CollectionItem }>()
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

// The achievement behind the item, for the badge's tooltip: "si sblocca con «…»".
const lockText = computed((): string | null => {
  const lock = props.item.lock
  switch (lock.kind) {
    case 'free':
      return null
    case 'unlocked':
    case 'locked':
    case 'unknown': {
      const name =
        lock.text ?? `${t('collection.achievement')} ${lock.achievement}`
      return `${t('collection.lockedBy')} «${name}»`
    }
    default:
      return assertNever(lock)
  }
})
</script>

<template>
  <!-- One row of the Collection's grid: the six cells are the grid's children, so this component
       has no root of its own. -->
  <span class="flex justify-center">
    <PixelSprite :url="item.iconUrl" placeholder class="size-8 shrink-0" />
  </span>
  <span class="flex min-w-0 flex-col px-2">
    <span class="truncate text-row text-foreground">{{ item.name }}</span>
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
    <Tooltip :disabled="lockText === null">
      <TooltipTrigger as-child>
        <Badge
          :variant="variant[state]"
          :tabindex="lockText === null ? undefined : 0"
          >{{ t(itemStateText[state]) }}</Badge
        >
      </TooltipTrigger>
      <TooltipContent class="max-w-80 text-caption text-foreground">{{
        lockText
      }}</TooltipContent>
    </Tooltip>
  </span>
</template>
