<script setup lang="ts">
import { computed } from 'vue'
import WhyMenu from '@/components/graph/WhyMenu.vue'
import { Badge, BadgeVariant } from '@/components/ui/badge'
import { useMessages } from '@/i18n'
import { itemStateText } from '@/lib/collection/collectionLabels'
import { ItemState, itemState } from '@/lib/collection/itemState'
import { lockWhy } from '@/lib/graph/whyMenu'
import type { CollectionItem } from '@/lib/ipc/types'

const props = defineProps<{ item: CollectionItem }>()
const { t } = useMessages()

const state = computed(() => itemState(props.item))

const variant: Record<ItemState, BadgeVariant> = {
  [ItemState.InCollection]: BadgeVariant.Done,
  [ItemState.Available]: BadgeVariant.Now,
  [ItemState.Locked]: BadgeVariant.Blocked,
  [ItemState.Unknown]: BadgeVariant.Unknown,
}

// The achievement behind the item, as the badge's menu: one group, one entry, and the page
// that says how that achievement is earned.
const groups = computed(() => lockWhy(props.item.lock, t))
</script>

<template>
  <WhyMenu :groups="groups" :label="t('collection.lockedBy')">
    <Badge
      :variant="variant[state]"
      :tabindex="groups.length > 0 ? 0 : undefined"
      >{{ t(itemStateText[state]) }}</Badge
    >
  </WhyMenu>
</template>
