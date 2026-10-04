<script setup lang="ts">
import { computed } from 'vue'
import FindHighlight from '@/components/find/FindHighlight.vue'
import { useMessages } from '@/i18n'
import { unlockKindText } from '@/lib/graph/unlockKindText'
import type { CollectionItem } from '@/lib/ipc/types'

// An item's name, painted where the find bar matches it, and its id and kind beneath.
const props = defineProps<{
  item: CollectionItem
  /** What the find bar is looking for; empty while it is closed. */
  findQuery: string
  /** Whether this row is the match the bar is standing on. */
  findCurrent: boolean
}>()
const { t } = useMessages()

const subtitle = computed(
  () =>
    `${t('collection.id')} ${props.item.id} · ${t(unlockKindText[props.item.kind])}`,
)
</script>

<template>
  <span class="flex min-w-0 flex-col">
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
</template>
