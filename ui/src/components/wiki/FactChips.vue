<script setup lang="ts">
import { computed } from 'vue'
import { Chip } from '@/components/ui/chip'
import { useMessages } from '@/i18n'
import type { PageFacts } from '@/lib/ipc/types'
import { factChips } from '@/lib/wiki/factChips'
import { useWikiStore } from '@/stores/wiki'

// Every meaningful fact a page's own `PageFacts` carries, as coloured chips:
// a list card, a table row's expanded state, and a page's hero all read the same function.
// `limit` caps how many show, for a card that has no room for every one of them — the full set
// stays reachable on the single page.
const props = defineProps<{ facts: PageFacts; limit?: number }>()
const { t } = useMessages()
const wiki = useWikiStore()

const chips = computed(() => {
  const all = factChips(props.facts, wiki.titleOf)
  return props.limit === undefined ? all : all.slice(0, props.limit)
})
</script>

<template>
  <div v-if="chips.length > 0" class="flex w-full min-w-0 flex-wrap gap-1.5">
    <Chip
      v-for="chip in chips"
      :key="chip.key"
      :tone="chip.tone"
      :title="t(chip.label.key, chip.label.params)"
      >{{ t(chip.label.key, chip.label.params) }}</Chip
    >
  </div>
</template>
