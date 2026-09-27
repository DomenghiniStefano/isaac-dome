<script setup lang="ts">
import { useMessages } from '@/i18n'
import InfoboxRow from '../InfoboxRow.vue'
import InfoboxTags from './InfoboxTags.vue'
import type { InfoboxOf } from './links'

// No game-sourced pools row: `itempools.xml`'s pools are collectibles only, never a
// trinket's, by the game's own construction (`ipc::item_pools`'s doc comment has the
// measurement) — a trinket has no game data to join here. `obtainedFrom` is the wiki's own
// text (7 of 188 trinkets write it) and needs no game at all.
defineProps<{ infobox: InfoboxOf<'trinket'> }>()
const { t } = useMessages()
</script>

<template>
  <dl class="flex flex-col gap-2">
    <InfoboxRow
      v-if="infobox.obtainedFrom.length > 0"
      :label="t('wiki.infobox.obtainedFrom')"
      :inline="infobox.obtainedFrom"
    />
    <InfoboxTags :tags="infobox.tags" />
  </dl>
</template>
