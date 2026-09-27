<script setup lang="ts">
import { computed } from 'vue'
import { Chip } from '@/components/ui/chip'
import { useMessages } from '@/i18n'
import type { Dlc } from '@/lib/ipc/types'
import { dlcNames } from '@/lib/wiki/dlcNames'
import { editionAdded, editionRemoved } from '@/lib/wiki/edition'
import { toneOfEdition } from '@/lib/wiki/tone'

// "Added in …" and "Removed in …", each in the edition's own colour (card #90, decision 9).
// `dlc` is the entry's own restriction list; `editionAdded`/`editionRemoved` already read
// the two null cases that mean "draw nothing" (no restriction, or present since Rebirth),
// so this component states no rule of its own about when a chip appears.
const props = defineProps<{ dlc: Dlc[] }>()
const { t } = useMessages()

const added = computed(() => editionAdded(props.dlc))
const removed = computed(() => editionRemoved(props.dlc))
</script>

<template>
  <span v-if="added || removed" class="flex flex-wrap items-center gap-2">
    <Chip v-if="added" :tone="toneOfEdition(added)">{{
      t('wiki.addedIn', { edition: dlcNames[added] })
    }}</Chip>
    <Chip v-if="removed" :tone="toneOfEdition(removed)">{{
      t('wiki.removedIn', { edition: dlcNames[removed] })
    }}</Chip>
  </span>
</template>
