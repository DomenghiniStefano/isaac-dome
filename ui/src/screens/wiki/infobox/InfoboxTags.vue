<script setup lang="ts">
import { Chip } from '@/components/ui/chip'
import { useMessages } from '@/i18n'
import { Tone } from '@/lib/wiki/tone'
import InfoboxRow from '../InfoboxRow.vue'

// A collectible's tags, the last row of an item's card and of a trinket's. Absent when the page
// states none. A tag carries no tone of its own (the catalog's vocabulary is open, `Infobox`'s
// own doc comment), so every one draws the same neutral chip `factChips.ts` already gives it.
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
    <Chip v-for="tag in tags" :key="tag" :tone="Tone.QualityNone">{{
      tag
    }}</Chip>
  </InfoboxRow>
</template>
