<script setup lang="ts">
import { Badge } from '@/components/ui/badge'
import { useMessages } from '@/i18n'
import InfoboxRow from '../InfoboxRow.vue'

// A collectible's tags, the last row of an item's card and of a trinket's. Absent when the page
// states none.
defineProps<{ tags: string[] }>()
const { t } = useMessages()
</script>

<template>
  <InfoboxRow
    v-if="tags.length > 0"
    :label="t('wiki.infobox.tags')"
    class="gap-1.5 pt-1"
    value-class="flex flex-wrap gap-1.5"
  >
    <Badge v-for="tag in tags" :key="tag">{{ tag }}</Badge>
  </InfoboxRow>
</template>
