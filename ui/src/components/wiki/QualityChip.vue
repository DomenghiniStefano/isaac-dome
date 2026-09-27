<script setup lang="ts">
import { computed } from 'vue'
import { Chip } from '@/components/ui/chip'
import { useMessages } from '@/i18n'
import { toneOfQuality } from '@/lib/wiki/tone'

// An item's quality, coloured the way the catalog rates it (card #90, decision 5): grey
// through bronze, silver, gold, to the vivid highlight of a 4. `null` — a trinket, or any
// page that has no quality at all — draws nothing, the same silence `QualityPips` keeps for
// it; the two are not one component because a pip row and a coloured chip answer different
// questions (how it compares to the others here, vs. what it is).
const props = defineProps<{ quality: number | null }>()
const { t } = useMessages()

const tone = computed(() =>
  props.quality === null ? null : toneOfQuality(props.quality),
)
</script>

<template>
  <Chip v-if="tone && quality !== null" :tone="tone">{{
    t('wiki.qualityChip', { quality })
  }}</Chip>
</template>
