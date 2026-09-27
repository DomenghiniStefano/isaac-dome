<script setup lang="ts">
import { computed } from 'vue'
import { Chip } from '@/components/ui/chip'
import { useMessages } from '@/i18n'
import type { Dlc } from '@/lib/ipc/types'
import { dlcNames } from '@/lib/wiki/dlcNames'
import { editionAdded, editionRemoved } from '@/lib/wiki/edition'
import { toneOfEdition } from '@/lib/wiki/tone'

// "Added in …" and "Removed in …", each in the edition's own colour.
// `dlc` is the entry's own restriction list; `editionAdded`/`editionRemoved` already read
// the two null cases that mean "draw nothing" (no restriction, or present since Rebirth),
// so this component states no rule of its own about when a chip appears. `compact` is a
// card's form: the edition's name alone, whole where "Added in Repentance+" would be cut, the
// sentence on hover; a removed edition's name is struck through.
const props = defineProps<{ dlc: Dlc[]; compact?: boolean }>()
const { t } = useMessages()

const added = computed(() => editionAdded(props.dlc))
const removed = computed(() => editionRemoved(props.dlc))
const addedText = computed(() =>
  added.value ? t('wiki.addedIn', { edition: dlcNames[added.value] }) : '',
)
const removedText = computed(() =>
  removed.value
    ? t('wiki.removedIn', { edition: dlcNames[removed.value] })
    : '',
)
</script>

<template>
  <span
    v-if="added || removed"
    class="flex max-w-full min-w-0 flex-wrap items-center gap-2"
  >
    <Chip v-if="added" :tone="toneOfEdition(added)" :title="addedText">{{
      compact ? dlcNames[added] : addedText
    }}</Chip>
    <Chip
      v-if="removed"
      :tone="toneOfEdition(removed)"
      :title="removedText"
      :class="compact ? 'line-through' : undefined"
      >{{ compact ? dlcNames[removed] : removedText }}</Chip
    >
  </span>
</template>
